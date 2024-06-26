mod cacher;
mod config;
mod icon_ui;
mod logs;
mod powerbar;
mod settings;

// Plugins
mod apps;
mod calc;
mod plugin;
mod plugin_manager;
mod template;
mod webq;

use anyhow::anyhow;
use egui::{IconData, ViewportBuilder, ViewportId};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use nucleo_matcher::{Config as NucleoConfig, Matcher};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use plugin::AppState;
use plugin_manager::PluginManager;
use powerbar::{LyraPowerbar, LyraPowerbarImpl};
use settings::LyraSettings;
use std::sync::Arc;
use std::time::Duration;
#[cfg(not(target_os = "linux"))]
use std::{cell::RefCell, rc::Rc};
use tray_icon::menu::MenuId;
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  TrayIconBuilder,
};
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use config::{Config, Placement, Styles};

const SETTINGS_MENU_ID: &str = "settings";

fn main() -> anyhow::Result<()> {
  let bld = setup_app()?;
  // Note this must be registered on the main thread or it doesn't work.
  let manager = GlobalHotKeyManager::new().unwrap();
  let hotkey_toggle: HotKey = bld.config.get().hotkey.parse().unwrap();
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
      event_loop_builder: Some(Box::new(|_evlp| {
        #[cfg(target_os = "macos")]
        _evlp
          .with_activation_policy(ActivationPolicy::Accessory)
          .with_activate_ignoring_other_apps(true);
      })),
      ..Default::default()
    },
    Box::new(move |cc| {
      let app = bld.build();
      init_event_listeners(
        cc.egui_ctx.clone(),
        app.powerbar.clone(),
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
  let config = Config::get_or_init_config().map(Arc::new)?;
  // TODO: maybe replace the PluginManager with just an enum
  let plugins = PluginManager::init(&config)?;

  Ok(LyraUiBuilder {
    config: config.clone(),
    plugins: plugins.clone(),
  })
}

struct LyraUi {
  powerbar: LyraPowerbar,
  settings: LyraSettings,
}

struct LyraUiBuilder {
  pub config: Arc<Config>,
  pub plugins: PluginManager,
}

impl LyraUiBuilder {
  fn build(self) -> LyraUi {
    let mut cfg = NucleoConfig::DEFAULT;
    cfg.ignore_case = true;
    cfg.prefer_prefix = true;
    LyraUi {
      powerbar: LyraPowerbar::new(LyraPowerbarImpl {
        state: AppState::default(),
        plugins: self.plugins,
        matcher: RwLock::new(Matcher::new(cfg)),
        config: self.config.clone(),
      }),
      settings: LyraSettings::new(self.config.clone()),
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

fn init_event_listeners(
  ctx: egui::Context,
  powerbar: LyraPowerbar,
  settings_vis: Arc<RwLock<bool>>,
  toggle_hk_id: u32,
) {
  // Not events must be actioned on in their own thread and not update
  // otherwise they will only be seen/reacted to when the UI is focused
  let hk_receiver = GlobalHotKeyEvent::receiver();
  let mu_receiver = MenuEvent::receiver();
  std::thread::spawn(move || loop {
    if let Ok(event) = hk_receiver.try_recv() {
      match (event.id(), event.state()) {
        (id, HotKeyState::Released) if id == toggle_hk_id => {
          let vis = !ctx.input(|is| is.viewport().focused).unwrap_or(false);
          powerbar.close(&ctx, vis);
        }
        _ => {}
      }
    }

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
    window_placement,
    window_size: size,
    ..
  } = cfg.get().styles;

  let mut bld = egui::ViewportBuilder::default()
    .with_resizable(false)
    .with_always_on_top()
    .with_decorations(false)
    .with_fullscreen(false)
    .with_transparent(true)
    .with_active(true)
    .with_visible(true)
    .with_min_inner_size(size)
    .with_inner_size(size);
  if cfg!(not(target_os = "linux")) {
    // TODO: This causes a SegFault when clicking the tray icon; it's not really
    // clear why but it has to do with eframe dependency. For now just disable it..
    bld = bld.with_icon(APP_ICON_ALT.clone());
  }
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
