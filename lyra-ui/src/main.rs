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
      init_event_listeners(cc.egui_ctx.clone(), toggle_hk_id);
      egui_extras::install_image_loaders(&cc.egui_ctx);
      // Note the tray must be built in this thread or it doesn't work (non-linux)
      #[cfg(not(target_os = "linux"))]
      _tray_ref
        .borrow_mut()
        .replace(mk_system_tray().build().unwrap());
      Box::new(bld.build())
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
  let plugins = PluginManager::init(config.clone())?;
  Ok(LyraUiBuilder {
    config: config.clone(),
    plugins: plugins.clone(),
    launcher: Launcher::new(config, plugins),
    clipboard: Clipboard::new()?,
  })
}

fn get_shortname<'a>(v: &'a Value) -> Option<&'a str> {
  v.as_object()
    .and_then(|m| m.get("shortname"))
    .and_then(|v| v.as_str())
}

fn get_required_args(v: &Value) -> Option<usize> {
  v.as_object()
    .and_then(|m| m.get("required_args"))
    .and_then(|v| v.as_u64())
    .map(|v| v as usize)
}

fn is_default_search(shortname: &str) -> bool {
  shortname.is_empty()
}

fn is_bookmark(v: &Value) -> bool {
  get_required_args(v).filter(|n| *n == 0).is_some()
}

fn extract_args<'a>(v: &'a Value, input: &'a str) -> Option<(Vec<&'a str>, usize)> {
  if get_shortname(v)
    .filter(|sn| is_default_search(sn))
    .and(Some(input).filter(|i| !i.is_empty()))
    .is_some()
  {
    return Some((vec![input], 0));
  }
  get_required_args(v).map(|rq| {
    (
      input.trim().splitn(rq + 1, ' ').skip(1).collect::<Vec<_>>(),
      rq,
    )
  })
}

// TODO: This TemplatingState concept is best moved to the WebQ plugin where the templating
// code lives; and exposed as some pure functions
#[derive(Debug, PartialEq, Eq)]
enum TemplatingState {
  NotStarted,
  Started,
  Complete,
}

impl TemplatingState {
  fn templating(&self) -> bool {
    *self != TemplatingState::NotStarted
  }

  fn is_complete(&self) -> bool {
    *self == TemplatingState::Complete
  }

  fn compute(&mut self, selected: Option<&(String, Value)>, input: &str) -> bool {
    match self {
      TemplatingState::NotStarted => {
        if self.is_templating_started(selected, input) {
          *self = TemplatingState::Started;
          return true;
        } else if self.is_templating_complete(selected, input) {
          *self = TemplatingState::Complete;
        }
      }
      TemplatingState::Started => {
        if !self.is_templating_started(selected, input) {
          *self = TemplatingState::NotStarted;
        } else if self.is_templating_complete(selected, input) {
          *self = TemplatingState::Complete;
        }
      }
      TemplatingState::Complete => {
        if !self.is_templating_complete(selected, input) {
          *self = TemplatingState::Started;
        }
      }
    };
    false
  }

  fn is_templating_started(&self, selected: Option<&(String, Value)>, input: &str) -> bool {
    let Some((_, pv)) = selected else {
      return false;
    };
    if is_bookmark(pv) {
      return false;
    }
    let Some(sn) = get_shortname(pv) else {
      return false;
    };
    !is_default_search(sn) && input.starts_with(sn) && input.contains(" ")
  }

  fn is_templating_complete(&self, selected: Option<&(String, Value)>, input: &str) -> bool {
    let Some((_, pv)) = selected else {
      return false;
    };
    if is_bookmark(pv) {
      return false;
    }
    extract_args(pv, input)
      .filter(|(args, required)| args.len() == *required)
      .is_some()
  }
}

struct LyraUi {
  config: Arc<Config>,
  plugins: PluginManager,
  launcher: Launcher,
  clipboard: Clipboard,
  input: String,
  options: Vec<(String, Value)>,
  selected: usize,
  templating: TemplatingState,
}

struct LyraUiBuilder {
  pub config: Arc<Config>,
  pub plugins: PluginManager,
  pub launcher: Launcher,
  pub clipboard: Clipboard,
}
impl LyraUiBuilder {
  fn build(self) -> LyraUi {
    LyraUi {
      config: self.config,
      plugins: self.plugins,
      launcher: self.launcher,
      clipboard: self.clipboard,
      input: "".into(),
      options: Vec::new(),
      selected: 0,
      templating: TemplatingState::NotStarted,
    }
  }
}

impl LyraUi {
  fn reset_state(&mut self) {
    self.input = "".into();
    self.options = Vec::new();
    self.selected = 0;
    self.templating = TemplatingState::NotStarted;
  }

  fn update_template_state(&mut self) {
    // TODO: This is buggy ->
    //   - The selected option might not be the one that's actually matching
    //     the template prefix typed into input
    //   - Once templating starts we need to be filtering to only exact matches
    //     and not "starts_with" matches
    //   - ^^ Notice I said matches and not singular match. Fix that too.
    if self
      .templating
      .compute(self.options.get(self.selected), &self.input)
    {
      self.selected = 0;
      self.options = vec![self.options[self.selected].clone()];
    }
    println!("{:?} -> {:?}", self.templating, self.input);
  }
}

impl eframe::App for LyraUi {
  fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
    // Fill the window with nothing so transparent still takes effect
    egui::Rgba::TRANSPARENT.to_array()
  }

  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Window does not play well auto-hiding on focus loss on Linux, so we'll
    // leave it as open until manually closed
    #[cfg(not(target_os = "linux"))]
    if ctx.input(|is| is.events.iter().any(|e| *e == Event::WindowFocused(false))) {
      close_window(ctx, false);
      return;
    }

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
      close_window(ctx, false);
      return;
    }
    if ctx.input(is_nav_down) {
      self.selected = (self.selected + 1).min(self.options.len().checked_sub(1).unwrap_or(0));
      self.update_template_state();
    }
    if ctx.input(is_nav_up) {
      self.selected = self.selected.checked_sub(1).unwrap_or(0);
      self.update_template_state();
    }

    if ctx.input(|i| i.key_released(Key::Enter)) {
      if let Some((pv, opt)) = self.options.get(self.selected) {
        let value = match pv.as_str() {
          "calc" => opt.get("Ok").unwrap().clone(),
          "apps" => opt.clone(),
          "webq" if self.templating.is_complete() || is_bookmark(opt) => {
            let (args, _) = extract_args(opt, &self.input).unwrap();
            opt
              .as_object()
              .map(|m| {
                let mut m = m.clone();
                m.insert(
                  "args".into(),
                  Value::Array(args.iter().map(|v| Value::String(v.to_string())).collect()),
                );
                Value::Object(m)
              })
              .unwrap()
          }
          "webq" => {
            // TODO: Would be nice to enter templating mode on the selected
            //       item, which will require updating the input to be the prefix
            //       + a space & then updating template state
            return;
          }
          _ => return,
        };
        if let Err(e) = launcher::submit(
          &mut self.clipboard,
          &self.launcher,
          pv.clone(),
          value,
          || close_window(ctx, false),
        ) {
          error!("{:?}", e);
        } else {
          self.reset_state();
        }
      }
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

          if res.changed() {
            self.update_template_state();
            if !self.templating.templating() {
              self.options = launcher::search(&self.launcher, &self.input);
              self.selected = 0;
              self.update_template_state();
            }
          }

          // TODO: Extract all styles & sizes/paddings to object on app so they can be set from
          //       once place as "constants"
          // TODO: Eventually can defer UI behavior to each plugin tbh
          // TODO: Find a better interface than Value; this might be a good next step.
          //   - You could use the `Any` trait and store the plugin structs + downcast https://ysantos.com/blog/downcast-rust
          //     - Either downcasting at each usage (ew)
          //     - Or at handing back to the pluginManager (this is very possible; the data is disjoint and you only interact with it by itself).
          //       The tricky part is how templating manipulates input/selected/options. You need a hook for that for any plugin to mess with (eg)
          //       when any app state is updated, check with each plugin if it has state to update && store that on the LyraUi/replace it.
          //   - You could implement each behavior as a trait and create a "super trait" representing something implementing each of them
          //     then create a blanket impl on each trait w/ a no-op/default behavior. Then each plugin can opt into behavior by implementing the
          //     trait and at each of these needed callsites invoke the beahvior on the option.
          //     - This may have edge cases like when one case needs more data than another
          //     - Nor does it fully grasp the templating issue.
          //   - Perhaps its best to ditch the enabled/disabled plugins concept and just have them all enabled always; then define their core
          //     data structures in the central plugin so it can be made to an enum
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
    .and_then(parse_image_data)
    .ok_or(anyhow!("TODO: Non-PNG support"))
    .and_then(|(s, ext)| convert::decode_bytes(&s).map(|b| (b, ext)));

  ui.horizontal(|ui| {
    if let Ok((img, ext)) = icon {
      ui.add(
        Image::from_bytes(format!("bytes://{}.{}", label.to_string(), ext), img)
          .maintain_aspect_ratio(true)
          .shrink_to_fit(),
      );
    }
    ui.label(mk_text(label));
  });
}

// TODO: Eventually do away with this as we'll want to just natively
// integrate, but config is currently all data url based. Ideally just use
// PNG since they seem to render cleaner
fn parse_image_data(s: &str) -> Option<(String, String)> {
  let prefixes = vec![
    ("image/svg+xml", "svg"),
    ("image/png", "png"),
    ("image/vnd.microsoft.icon", "ico"),
    ("image/jpeg", "jpg"),
  ];
  prefixes.iter().find_map(|(pf, ext)| {
    s.strip_prefix(&format!("data:{};base64,", pf))
      .map(|s| (s.to_string(), ext.to_string()))
  })
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

    // BUG: (Linux) -> https://github.com/tauri-apps/tray-icon/issues/104
    // tl;dr - Tray Click events don't work and actually segfault.
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
  // BUG: Linux -> Sometimes (not always) the app doesn't want to revive. It's
  // either the global hotkey has died, or something wrong here. Need more debug
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
