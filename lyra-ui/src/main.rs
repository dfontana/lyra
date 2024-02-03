mod config;
// mod convert;
mod launcher;
mod logs;
// mod page;
mod plugin_manager;
use eframe::egui;
use egui::{Event, IconData, ViewportBuilder};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::sync::Arc;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  TrayIconBuilder,
};

use config::{Config, Placement, Styles};
use launcher::Launcher;
// use page::{MainData, Page, SettingsData};
use plugin_manager::PluginManager;
use tracing::{error, info};

fn mk_viewport(cfg: Arc<Config>, icon: IconData) -> ViewportBuilder {
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
    .with_icon(icon)
    .with_min_inner_size([400.0, 100.0])
    .with_inner_size([400.0, 100.0]);

  match window_placement {
    Placement::XY(x, y) => {
      bld = bld.with_position([x, y]);
    }
  }
  bld
}

fn _mk_settings() {
  // TODO: Need to make the settings page
}

fn main() -> Result<(), eframe::Error> {
  if let Err(err) = logs::init_logs() {
    error!("Failed to start logger: {}", err);
    // TODO fix this return instead
    panic!("TODO FIX THIS RETURN INSTEAD");
  }

  let config = match Config::get_or_init_config() {
    Ok(c) => Arc::new(c),
    Err(err) => {
      info!("Failed to initialize config: {}", err);
      // TODO fix this return instead
      panic!("TODO FIX THIS RETURN INSTEAD");
    }
  };

  let plugin_manager = match PluginManager::init(config.clone()) {
    Ok(pm) => pm,
    Err(err) => {
      info!("Failed to initialize plugins: {}", err);
      // TODO fix this return instead
      panic!("TODO FIX THIS RETURN INSTEAD");
    }
  };

  Launcher::new(config.clone(), plugin_manager);

  // TODO#37 move this into the config so folks can customize the trigger
  let manager = GlobalHotKeyManager::new().unwrap();
  let hotkey: HotKey = "CmdOrCtrl+Space".parse().unwrap();
  let toggle_hk_id = hotkey.id();
  let _ = manager.register(hotkey);

  let hk_receiver = GlobalHotKeyEvent::receiver();
  let mu_receiver = MenuEvent::receiver();

  // TODO: You'll want to include_bytes this so it's in the final binary
  let tray_icon = load_icon(std::path::Path::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/icons/app-icon.png"
  )));
  let dock_icon = load_icon_data(std::path::Path::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/icons/app-icon-alt.png"
  )));

  let mut _tray_icon = Rc::new(RefCell::new(None));
  let tray_c = _tray_icon.clone();
  let menu_bar = Menu::new();
  let _ = menu_bar.append_items(&[
    &MenuItem::with_id("settings", "Settings", true, None),
    &PredefinedMenuItem::quit(None),
  ]);
  let tray_bld = TrayIconBuilder::new()
    .with_menu(Box::new(menu_bar))
    .with_tooltip("Lyra")
    .with_icon(tray_icon);

  let options = eframe::NativeOptions {
    viewport: mk_viewport(config.clone(), dock_icon),
    ..Default::default()
  };
  eframe::run_native(
    "My egui App",
    options,
    Box::new(move |cc| {
      {
        tray_c.borrow_mut().replace(tray_bld.build().unwrap());
      }
      let frame = cc.egui_ctx.clone();
      {
        std::thread::spawn(move || loop {
          if let Ok(event) = hk_receiver.try_recv() {
            match (event.id(), event.state()) {
              (id, HotKeyState::Released) if id == toggle_hk_id => {
                let vis = !frame.input(|is| is.viewport().focused).unwrap_or(false);
                close_window(&frame, vis);
              }
              _ => {}
            }
          }

          if let Ok(event) = mu_receiver.try_recv() {
            println!("menu event: {:?}", event);
          }
          std::thread::sleep(Duration::from_millis(100));
        });
      }

      Box::<LyraUi>::default()
    }),
  )
}

#[derive(Default)]
struct LyraUi {}

impl eframe::App for LyraUi {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    if ctx.input(|is| is.events.iter().any(|e| *e == Event::WindowFocused(false))) {
      close_window(ctx, false);
      return;
    }

    // TODO Start building the ui! Need to pull in adjacent packages
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("My egui Application");
      // ui.horizontal(|ui| {
      //   let name_label = ui.label("Your name: ");
      //   ui.text_edit_singleline(&mut self.name)
      //     .labelled_by(name_label.id);
      // });
      // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
      // if ui.button("Click each year").clicked() {
      //   self.age += 1;
      // }
      // ui.label(format!("Hello '{}', age {}", self.name, self.age));
    });
  }
}

fn close_window(ctx: &egui::Context, vis: bool) {
  // TODO: Ideally this hides the application (meaning invisible AND yields focus)
  //       Right now it just goes invisible and does not yield focus.
  ctx.send_viewport_cmd(egui::ViewportCommand::Visible(vis));
  if vis {
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
  }
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
  let (icon_rgba, icon_width, icon_height) = {
    let image = image::open(path)
      .expect("Failed to open icon path")
      .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    (rgba, width, height)
  };
  tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn load_icon_data(path: &std::path::Path) -> IconData {
  // TODO: This icon is bigger in the doc than others, need to look into what size it should be
  let image = image::open(path)
    .expect("Failed to open icon path")
    //.rescale(...) to change the size or pick a diff source image
    .into_rgba8();
  IconData {
    width: image.width(),
    height: image.height(),
    rgba: image.into_raw(),
  }
}
