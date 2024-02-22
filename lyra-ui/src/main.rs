mod config;
mod launcher;
mod logs;
mod powerbar;
mod settings;

use anyhow::anyhow;
use egui::{IconData, ViewportBuilder, ViewportId};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use lyra_plugin::{AppState, PluginManager};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use powerbar::{close_powerbar, LyraPowerbar};
use settings::LyraSettings;
use std::sync::Arc;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};
use tray_icon::menu::MenuId;
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  TrayIconBuilder,
};
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use config::{Config, Placement, Styles};
use launcher::Launcher;

const SETTINGS_MENU_ID: &str = "settings";

fn main() -> anyhow::Result<()> {
  let bld = setup_app()?;
  // TODO#37 move this into the config so folks can customize the trigger
  // Note this must be registered on the main thread or it doesn't work.
  let manager = GlobalHotKeyManager::new().unwrap();
  let hotkey_toggle: HotKey = "CmdOrCtrl+Space".parse().unwrap();
  let toggle_hk_id = hotkey_toggle.id();
  let _ = manager.register(hotkey_toggle);

  // Note must hold a Rc<RefCell<>> on for the Tray to stay alive/active
  // On linux, this requires using GTK and not winit, which forces this
  #[cfg(target_os = "linux")]
  std::thread::spawn(|| {
    gtk::init().unwrap();
    let _tray_handle = mk_system_tray().build().unwrap();
    gtk::main();
  });
  #[cfg(not(target_os = "linux"))]
  let mut _tray_handle = Rc::new(RefCell::new(None));
  #[cfg(not(target_os = "linux"))]
  let _tray_ref = _tray_handle.clone();

  eframe::run_native(
    "Lyra",
    eframe::NativeOptions {
      viewport: mk_viewport(bld.config.clone()),
      event_loop_builder: Some(Box::new(|evlp| {
        #[cfg(target_os = "macos")]
        evlp
          .with_activation_policy(ActivationPolicy::Accessory)
          .with_activate_ignoring_other_apps(true);
      })),
      ..Default::default()
    },
    Box::new(move |cc| {
      let app = bld.build();
      init_event_listeners(
        cc.egui_ctx.clone(),
        app.settings.visible.clone(),
        toggle_hk_id,
      );
      egui_extras::install_image_loaders(&cc.egui_ctx);
      // Note the tray must be built in this thread or it doesn't work (non-linux)
      #[cfg(not(target_os = "linux"))]
      _tray_ref
        .borrow_mut()
        .replace(mk_system_tray().build().unwrap());
      Box::new(app)
    }),
  )
  .map_err(|err| anyhow!("{}", err))
}

fn setup_app() -> Result<LyraUiBuilder, anyhow::Error> {
  logs::init_logs()?;
  // TODO: Now that we're under one app, can we just put all the plugin
  //       configs writing to dedicated tables in the TOML?
  // TODO: The default config needs more things setup (like plugins, & prefixes in calc)
  //       Alternatively need to display a message when no plugins are active, but better
  //       to have defaults.
  let config = Config::get_or_init_config().map(Arc::new)?;
  let plugins = PluginManager::init(&config.get().plugins, &config.conf_dir, &config.cache_dir)?;

  Ok(LyraUiBuilder {
    config: config.clone(),
    plugins: plugins.clone(),
    // TODO: Don't really need a launcher when I have the plugins. Perhaps remove?
    // TODO: And maybe ditch the plugin structure/fold it into main src
    launcher: Launcher::new(config, plugins),
  })
}

struct LyraUi {
  powerbar: LyraPowerbar,
  settings: LyraSettings,
}

struct LyraUiBuilder {
  pub config: Arc<Config>,
  pub plugins: PluginManager,
  pub launcher: Launcher,
}

impl LyraUiBuilder {
  fn build(self) -> LyraUi {
    LyraUi {
      powerbar: LyraPowerbar {
        state: AppState::default(),
        plugins: self.plugins,
        launcher: self.launcher,
      },
      settings: LyraSettings::default(),
    }
  }
}

impl eframe::App for LyraUi {
  fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
    // Fill the window with nothing so transparent still takes effect
    egui::Rgba::TRANSPARENT.to_array()
  }

  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    self.settings.update(ctx);
    self.powerbar.update(ctx);
  }
}

fn init_event_listeners(ctx: egui::Context, settings_vis: Arc<RwLock<bool>>, toggle_hk_id: u32) {
  // Not events must be actioned on in their own thread and not update
  // otherwise they will only be seen/reacted to when the UI is focused
  let hk_receiver = GlobalHotKeyEvent::receiver();
  let mu_receiver = MenuEvent::receiver();
  std::thread::spawn(move || loop {
    if let Ok(event) = hk_receiver.try_recv() {
      match (event.id(), event.state()) {
        (id, HotKeyState::Released) if id == toggle_hk_id => {
          let vis = !ctx.input(|is| is.viewport().focused).unwrap_or(false);
          close_powerbar(&ctx, vis);
        }
        _ => {}
      }
    }

    // BUG: (Linux) -> https://github.com/tauri-apps/tray-icon/issues/104
    // tl;dr - Tray Click events don't work and actually segfault.
    if let Ok(event) = mu_receiver.try_recv() {
      if *event.id() == MenuId::new(SETTINGS_MENU_ID) {
        *settings_vis.write() = true;
        ctx.send_viewport_cmd_to(ViewportId::ROOT, egui::ViewportCommand::Visible(true));
      }
    }
    std::thread::sleep(Duration::from_millis(100));
  });
}

fn mk_system_tray() -> TrayIconBuilder {
  let menu_bar = Menu::new();
  let _ = menu_bar.append_items(&[
    &MenuItem::with_id(SETTINGS_MENU_ID, "Settings", true, None),
    &PredefinedMenuItem::quit(None),
  ]);
  TrayIconBuilder::new()
    .with_menu(Box::new(menu_bar))
    .with_tooltip("Lyra")
    .with_icon(APP_ICON.clone())
}

fn mk_viewport(cfg: Arc<Config>) -> ViewportBuilder {
  let Styles {
    window_placement, ..
  } = cfg.get().styles;

  let mut bld = egui::ViewportBuilder::default()
    .with_resizable(false)
    .with_always_on_top()
    .with_decorations(false)
    .with_fullscreen(false)
    .with_transparent(true)
    .with_active(true)
    .with_visible(true)
    .with_icon(APP_ICON_ALT.clone())
    // TODO: Pull from config
    .with_min_inner_size([600.0, 32.0])
    .with_inner_size([600.0, 32.0]);
  match window_placement {
    Placement::XY(x, y) => {
      bld = bld.with_position([x, y]);
    }
  }
  bld
}

static APP_ICON_ALT: Lazy<IconData> = Lazy::new(|| {
  load_image(
    include_bytes!("../icons/app-icon-alt.png"),
    |width, height, rgba| IconData {
      width,
      height,
      rgba,
    },
  )
});

static APP_ICON: Lazy<tray_icon::Icon> = Lazy::new(|| {
  load_image(include_bytes!("../icons/app-icon.png"), |w, h, v| {
    tray_icon::Icon::from_rgba(v, w, h).unwrap()
  })
});

fn load_image<T>(bytes: &[u8], bld: impl FnOnce(u32, u32, Vec<u8>) -> T) -> T {
  let image = image::load_from_memory(bytes)
    .expect("Failed to parse image")
    .into_rgba8();
  bld(image.width(), image.height(), image.into_raw())
}
