mod config;
mod convert;
mod launcher;
mod logs;
mod plugin_manager;

use anyhow::anyhow;
use eframe::egui;
use egui::{
  Align, Color32, Event, EventFilter, FontId, IconData, Image, Key, Modifiers, TextBuffer,
  TextEdit, ViewportBuilder,
};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};
use tray_icon::TrayIcon;
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  TrayIconBuilder,
};

use config::{Config, Placement, Styles};
use launcher::Launcher;
use plugin_manager::PluginManager;

// TODO Move these into styles & pass around
const ROW_HEIGHT: f32 = 32.0;
const APP_WIDTH: f32 = 600.0;

fn main() -> anyhow::Result<()> {
  let bld = setup_app()?;
  // TODO#37 move this into the config so folks can customize the trigger
  // Note this must be registered on the main thread or it doesn't work.
  let manager = GlobalHotKeyManager::new().unwrap();
  let hotkey_toggle: HotKey = "CmdOrCtrl+Space".parse().unwrap();
  let toggle_hk_id = hotkey_toggle.id();
  let _ = manager.register(hotkey_toggle);

  eframe::run_native(
    "Lyra",
    eframe::NativeOptions {
      viewport: mk_viewport(bld.config.clone()),
      ..Default::default()
    },
    Box::new(move |cc| {
      init_event_listeners(cc.egui_ctx.clone(), toggle_hk_id);
      egui_extras::install_image_loaders(&cc.egui_ctx);
      // Note the tray must be built in this thread or it doesn't work
      Box::new(bld.build(mk_system_tray().build().unwrap()))
    }),
  )
  .map_err(|err| anyhow!("{}", err))
}

fn setup_app() -> Result<LyraUiBuilder, anyhow::Error> {
  logs::init_logs()?;
  let config = Config::get_or_init_config().map(Arc::new)?;
  let plugins = PluginManager::init(config.clone())?;
  Ok(LyraUiBuilder {
    config: config.clone(),
    plugins: plugins.clone(),
    launcher: Launcher::new(config, plugins),
  })
}

struct LyraUi {
  // Note must hold a Rc<RefCell<>> on for the Tray to stay alive/active
  _system_tray: Rc<RefCell<TrayIcon>>,
  config: Arc<Config>,
  plugins: PluginManager,
  launcher: Launcher,
  input: String,
  options: Vec<(String, Value)>,
  selected: usize,
}

struct LyraUiBuilder {
  pub config: Arc<Config>,
  pub plugins: PluginManager,
  pub launcher: Launcher,
}
impl LyraUiBuilder {
  fn build(self, system_tray: TrayIcon) -> LyraUi {
    LyraUi {
      _system_tray: Rc::new(RefCell::new(system_tray)),
      config: self.config,
      plugins: self.plugins,
      launcher: self.launcher,
      input: "".into(),
      options: Vec::new(),
      selected: 0,
    }
  }
}

impl eframe::App for LyraUi {
  fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
    // Fill the window with nothing so transparent still takes effect
    egui::Rgba::TRANSPARENT.to_array()
  }

  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    if ctx.input(|is| is.events.iter().any(|e| *e == Event::WindowFocused(false))) {
      close_window(ctx, false);
      return;
    }

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
      close_window(ctx, false);
      return;
    }

    let window_decor = egui::Frame {
      fill: Color32::TRANSPARENT,
      rounding: 5.0.into(),
      stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
      outer_margin: 0.5.into(), // so the stroke is within the bounds
      ..Default::default()
    };

    egui::CentralPanel::default()
      .frame(window_decor)
      .show(ctx, |ui| {
        let padding = 4.0;
        let rect = ui.max_rect().shrink(padding);
        let mut ui = ui.child_ui(rect, *ui.layout());

        ui.vertical_centered(|ui| {
          // TODO: Only render frame on the selected option and invert the colors. Distinguish this one
          //       differently
          let res = mk_text_edit(&mut self.input, false, true).show(ui).response;
          res.request_focus();

          // Navigation (TODO: this needs cleaning up, yuck)
          if ui.input(|i| {
            i.key_released(Key::ArrowDown)
              || (!i.modifiers.matches_exact(Modifiers::SHIFT) && i.key_released(Key::Tab))
          }) {
            self.selected = (self.selected + 1).min(self.options.len().checked_sub(1).unwrap_or(0));
          }
          if ui.input(|i| {
            i.key_released(Key::ArrowUp)
              || (i.modifiers.matches_exact(Modifiers::SHIFT) && i.key_released(Key::Tab))
          }) {
            self.selected = self.selected.checked_sub(1).unwrap_or(0);
          }

          if res.changed() {
            // TODO: Perform calc if calc'ing
            // TODO: Perform templating if templating
            self.options = launcher::search(&self.launcher, &self.input);
            self.selected = 0;
          }

          for (idx, (plugin_name, opt)) in self.options.iter().enumerate() {
            // TODO: Handle the other AppLaunch/Launcher response shapes. Maybe find a better interface than Value.
            let obj = opt.as_object();

            let label = obj
              .filter(|m| m.contains_key("label"))
              .and_then(|m| m.get("label"))
              .map(|l| l.to_string())
              .map(|s| s.trim_matches('"').to_owned())
              .unwrap_or("Unlabelled Result".into());

            let icon = obj
              .filter(|m| m.contains_key("icon"))
              .and_then(|m| m.get("icon"))
              .and_then(|v| v.as_str())
              .and_then(|s| {
                s.strip_prefix("data:image/png;base64,")
                  .map(|s| s.to_owned())
              })
              .ok_or(anyhow!("TODO: Non-PNG support"))
              .and_then(|s| convert::decode_bytes(&s));

            ui.horizontal(|ui| {
              // TODO: Can we render the entire row's background instead of just text edit frame?
              //       Can then just render frame on the input
              if let Ok(img) = icon {
                ui.add(
                  Image::from_bytes(format!("bytes://{}.png", label.to_string()), img)
                    .maintain_aspect_ratio(true)
                    .shrink_to_fit(),
                );
              }
              mk_text_edit(&mut label.to_string(), idx == self.selected, true).show(ui);
            });
          }

          if res.changed() {
            // TODO: Can compute/layout this better by using actual occupied rect
            //       and having the input row be a parent to these
            let height = ui.min_rect().height() + (padding * 2.0);
            let width = ui.min_rect().width() + (padding * 2.0);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([width, height].into()));
          }
        });
      });
  }
}

fn mk_text_edit<'t>(text: &'t mut dyn TextBuffer, selected: bool, interactive: bool) -> TextEdit {
  TextEdit::singleline(text)
    .desired_width(f32::INFINITY)
    .margin((0.0, 2.0).into())
    // TODO: User entered FontDefinition with fallback
    .font(FontId::new(16.0, egui::FontFamily::Monospace))
    .text_color(Color32::WHITE)
    .clip_text(true)
    .cursor_at_end(true)
    .vertical_align(Align::Center)
    .frame(selected)
    .interactive(interactive)
    .event_filter(EventFilter {
      tab: false,
      horizontal_arrows: true,
      vertical_arrows: false,
      ..Default::default()
    })
}

fn init_event_listeners(ctx: egui::Context, toggle_hk_id: u32) {
  // Not events must be actioned on in their own thread and not update
  // otherwise they will only be seen/reacted to when the UI is focused
  let hk_receiver = GlobalHotKeyEvent::receiver();
  let mu_receiver = MenuEvent::receiver();
  std::thread::spawn(move || loop {
    if let Ok(event) = hk_receiver.try_recv() {
      match (event.id(), event.state()) {
        (id, HotKeyState::Released) if id == toggle_hk_id => {
          let vis = !ctx.input(|is| is.viewport().focused).unwrap_or(false);
          close_window(&ctx, vis);
        }
        _ => {}
      }
    }

    if let Ok(event) = mu_receiver.try_recv() {
      // TODO: Open the settings frame, which will need to be built out & added into update loop.
      println!("menu event: {:?}", event);
    }
    std::thread::sleep(Duration::from_millis(100));
  });
}

fn close_window(ctx: &egui::Context, vis: bool) {
  // TODO: Ideally this hides the application (meaning invisible AND yields focus)
  //       Right now it just goes invisible and does not yield focus.
  ctx.send_viewport_cmd(egui::ViewportCommand::Visible(vis));
  if vis {
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
  }
}

fn mk_system_tray() -> TrayIconBuilder {
  // TODO: You'll want to include_bytes this so it's in the final binary
  let tray_icon = load_icon(std::path::Path::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/icons/app-icon.png"
  )));

  let menu_bar = Menu::new();
  let _ = menu_bar.append_items(&[
    &MenuItem::with_id("settings", "Settings", true, None),
    &PredefinedMenuItem::quit(None),
  ]);
  TrayIconBuilder::new()
    .with_menu(Box::new(menu_bar))
    .with_tooltip("Lyra")
    .with_icon(tray_icon)
}

fn mk_viewport(cfg: Arc<Config>) -> ViewportBuilder {
  let Styles {
    window_placement, ..
  } = cfg.get().styles;

  // TODO: include_bytes! style pls
  let icon = load_icon_data(std::path::Path::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/icons/app-icon-alt.png"
  )));
  let mut bld = egui::ViewportBuilder::default()
    .with_resizable(false)
    .with_always_on_top()
    .with_decorations(false)
    .with_fullscreen(false)
    .with_transparent(true)
    .with_active(true)
    .with_visible(true)
    .with_icon(icon)
    .with_min_inner_size([APP_WIDTH, ROW_HEIGHT])
    .with_inner_size([APP_WIDTH, ROW_HEIGHT]);
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
