mod config;
mod convert;
mod launcher;
mod logs;
mod plugin_manager;

use anyhow::anyhow;
use arboard::Clipboard;
use egui::{
  Align, Color32, Event, EventFilter, FontId, IconData, Image, InputState, Key, Modifiers,
  RichText, TextBuffer, TextEdit, Ui, ViewportBuilder,
};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};
use tracing::error;
use tray_icon::TrayIcon;
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  TrayIconBuilder,
};
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use config::{Config, Placement, Styles};
use launcher::Launcher;
use plugin_manager::PluginManager;

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
      event_loop_builder: Some(Box::new(|evlp| {
        #[cfg(target_os = "macos")]
        evlp
          .with_activation_policy(ActivationPolicy::Accessory)
          .with_activate_ignoring_other_apps(true);
      })),
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
    clipboard: Clipboard::new()?,
  })
}

struct LyraUi {
  // Note must hold a Rc<RefCell<>> on for the Tray to stay alive/active
  _system_tray: Rc<RefCell<TrayIcon>>,
  config: Arc<Config>,
  plugins: PluginManager,
  launcher: Launcher,
  clipboard: Clipboard,
  input: String,
  options: Vec<(String, Value)>,
  selected: usize,
}

struct LyraUiBuilder {
  pub config: Arc<Config>,
  pub plugins: PluginManager,
  pub launcher: Launcher,
  pub clipboard: Clipboard,
}
impl LyraUiBuilder {
  fn build(self, system_tray: TrayIcon) -> LyraUi {
    LyraUi {
      _system_tray: Rc::new(RefCell::new(system_tray)),
      config: self.config,
      plugins: self.plugins,
      launcher: self.launcher,
      clipboard: self.clipboard,
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
      fill: Color32::WHITE,
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
        ui.visuals_mut().override_text_color = Some(Color32::DARK_GRAY);
        ui.style_mut().override_font_id = Some(FontId::new(16.0, egui::FontFamily::Monospace));

        ui.vertical_centered(|ui| {
          let res = mk_text_edit(&mut self.input).show(ui).response;
          res.request_focus();

          // Navigation
          if ui.input(is_nav_down) {
            self.selected = (self.selected + 1).min(self.options.len().checked_sub(1).unwrap_or(0));
          }
          if ui.input(is_nav_up) {
            self.selected = self.selected.checked_sub(1).unwrap_or(0);
          }
          if ui.input(|i| i.key_released(Key::Enter)) {
            if let Some((pv, opt)) = self.options.get(self.selected) {
              let value = match pv.as_str() {
                "calc" => opt.get("Ok").unwrap(),
                "apps" => opt,
                // TODO: Need to impl templating extraction
                _ => return,
              };
              if let Err(e) = launcher::submit(
                &mut self.clipboard,
                &self.launcher,
                pv.clone(),
                value.clone(),
                || close_window(ctx, false),
              ) {
                error!("{:?}", e);
              }
            }
          }

          if res.changed() {
            // TODO: Perform templating if templating
            self.options = launcher::search(&self.launcher, &self.input);
            self.selected = 0;
          }

          // TODO: Extract all styles & sizes/paddings to object on app so they can be set from
          //       once place as "constants"
          // TODO: Eventually can defer UI behavior to each plugin tbh
          // TODO: Find a better interface than Value.
          for (idx, (plugin_name, opt)) in self.options.iter().enumerate() {
            let mut fm = egui::Frame::none().inner_margin(4.0).rounding(2.0);
            if idx == self.selected {
              fm = fm.fill(Color32::from_hex("#54e6ae").unwrap());
            }
            fm.show(ui, |ui| {
              if idx == self.selected {
                ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
              }
              match plugin_name.as_str() {
                "calc" => mk_calc(ui, opt, &self.input),
                "apps" => mk_app_res(ui, opt),
                // TODO: Can likely find a better thing to render here
                "webq" => mk_app_res(ui, opt),
                unk => {
                  error!("Unknown plugin: {}", unk);
                }
              };
              ui.set_width(ui.available_width());
            });
          }

          if res.changed() {
            let height = ui.min_rect().height() + (padding * 2.0);
            let width = ui.min_rect().width() + (padding * 2.0);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([width, height].into()));
          }
        });
      });
  }
}

fn is_nav_down(i: &InputState) -> bool {
  i.key_released(Key::ArrowDown)
    || (!i.modifiers.matches_exact(Modifiers::SHIFT) && i.key_released(Key::Tab))
}

fn is_nav_up(i: &InputState) -> bool {
  i.key_released(Key::ArrowUp)
    || (i.modifiers.matches_exact(Modifiers::SHIFT) && i.key_released(Key::Tab))
}

fn mk_calc(ui: &mut Ui, opt: &Value, inp: &str) {
  ui.horizontal(|ui| {
    let ok_result = opt
      .as_object()
      .and_then(|m| m.get("Ok"))
      .map(|v| v.to_string())
      .map(|s| s.trim_matches('"').to_owned());

    if let Some(v) = ok_result {
      ui.label(mk_text(v));
      return;
    }

    let err_result = opt
      .as_object()
      .and_then(|m| m.get("Err"))
      .and_then(|m| m.as_object());
    let err_msg = err_result
      .and_then(|m| m.get("message"))
      .map(|v| v.to_string())
      .map(|s| s.trim_matches('"').to_owned());
    let err_start = err_result
      .and_then(|m| m.get("start"))
      .and_then(|s| s.as_u64())
      .map(|v| v as usize);
    let err_end = err_result
      .and_then(|m| m.get("end"))
      .and_then(|s| s.as_u64())
      .map(|v| v as usize);

    match (err_start, err_end, err_msg) {
      (Some(s), Some(e), _) if s != 0 && e != 0 => {
        ui.label(mk_text(&inp[1..s]));
        ui.label(mk_text(&inp[s..e + 1]).color(Color32::RED));
        ui.label(mk_text(&inp[e + 1..]));
      }
      (_, _, Some(msg)) => {
        ui.label(mk_text(msg));
      }
      _ => return,
    }
  });
}

fn mk_app_res(ui: &mut Ui, opt: &Value) {
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
    if let Ok(img) = icon {
      ui.add(
        Image::from_bytes(format!("bytes://{}.png", label.to_string()), img)
          .maintain_aspect_ratio(true)
          .shrink_to_fit(),
      );
    }
    ui.label(mk_text(label));
  });
}

fn mk_text_edit<'t>(text: &'t mut dyn TextBuffer) -> TextEdit {
  TextEdit::singleline(text)
    .desired_width(f32::INFINITY)
    .margin((0.0, 2.0).into())
    .clip_text(true)
    .cursor_at_end(true)
    .vertical_align(Align::Center)
    .frame(false)
    .interactive(true)
    .event_filter(EventFilter {
      tab: false,
      horizontal_arrows: true,
      vertical_arrows: false,
      ..Default::default()
    })
}

fn mk_text(text: impl Into<String>) -> RichText {
  RichText::new(text)
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
  // TODO: So both TAO & Winit are issuing an orderOut command on MacOS
  // (https://developer.apple.com/documentation/appkit/nswindow/1419660-orderout)
  // but for some reason the previous application does not take focus. Tauri also
  // suffers from this so it's not a regression with using EGUI but instead
  // something else entirely that apps like Alfred & Raycast don't experience.
  // Will need more research.
  ctx.send_viewport_cmd(egui::ViewportCommand::Visible(vis));
  if vis {
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
  }
}

fn mk_system_tray() -> TrayIconBuilder {
  let menu_bar = Menu::new();
  let _ = menu_bar.append_items(&[
    &MenuItem::with_id("settings", "Settings", true, None),
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

fn _mk_settings() {
  // TODO: Need to make the settings page
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
