use egui::{
  Align, Color32, Event, EventFilter, FontId, InputState, Key, Modifiers, TextBuffer, TextEdit,
  ViewportId,
};
use lyra_plugin::{AppState, OkAction, PluginManager};
use tracing::error;

use crate::launcher::{self, Launcher};

pub struct LyraPowerbar {
  pub state: AppState,
  pub plugins: PluginManager,
  pub launcher: Launcher,
}

impl LyraPowerbar {
  fn reset_state(&mut self) {
    self.state = AppState::default();
  }

  fn check_plugins_for_state_updates(&mut self) {
    if let Some(st) = self
      .plugins
      .iter()
      .find_map(|p| p.derive_state(&self.state))
    {
      self.state = st;
    }
  }
}
impl LyraPowerbar {
  pub fn update(&mut self, ctx: &egui::Context) {
    // Window does not play well auto-hiding on focus loss on Linux, so we'll
    // leave it as open until manually closed
    #[cfg(not(target_os = "linux"))]
    if ctx.input(|is| is.events.iter().any(|e| *e == Event::WindowFocused(false))) {
      close_powerbar(ctx, false);
      return;
    }

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
      close_powerbar(ctx, false);
      return;
    }

    if ctx.input(is_nav_down) {
      self.state.selected =
        (self.state.selected + 1).min(self.state.options.len().checked_sub(1).unwrap_or(0));
      self.check_plugins_for_state_updates();
    }

    if ctx.input(is_nav_up) {
      self.state.selected = self.state.selected.checked_sub(1).unwrap_or(0);
      self.check_plugins_for_state_updates();
    }

    if ctx.input(|i| i.key_released(Key::Enter)) {
      if let Some(opt) = self.state.selected() {
        match self.plugins.try_launch(opt) {
          Ok(OkAction { close_win: true }) => {
            // TODO: This should reset the size of the UI, so when it re-appears it's not 2+ lines tall
            close_powerbar(ctx, false);
            self.reset_state();
          }
          Ok(_) => self.reset_state(),
          Err(e) => error!("{:?}", e),
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
          let res = mk_text_edit(&mut self.state.input).show(ui).response;
          res.request_focus();

          if res.changed() {
            self.check_plugins_for_state_updates();
            if self
              .state
              .selected()
              .filter(|pv| pv.blocks_search(&self.state))
              .is_none()
            {
              self.state.options = launcher::search(&self.launcher, &self.state.input);
              self.state.selected = 0;
              self.check_plugins_for_state_updates();
            }
          }

          // TODO: Extract all styles & sizes/paddings to object on app so they can be set from
          //       once place as "constants"
          for (idx, pv) in self.state.options.iter().enumerate() {
            let mut fm = egui::Frame::none().inner_margin(4.0).rounding(2.0);
            if idx == self.state.selected {
              fm = fm.fill(Color32::from_hex("#54e6ae").unwrap());
            }
            fm.show(ui, |ui| {
              if idx == self.state.selected {
                ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
              }
              pv.render(ui, &self.state);
              ui.set_width(ui.available_width());
            });
          }

          if res.changed() {
            let height = ui.min_rect().height() + (padding * 2.0);
            let width = ui.min_rect().width() + (padding * 2.0);
            ctx.send_viewport_cmd_to(
              ViewportId::ROOT,
              egui::ViewportCommand::InnerSize([width, height].into()),
            );
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

pub fn close_powerbar(ctx: &egui::Context, vis: bool) {
  // TODO: So both TAO & Winit are issuing an orderOut command on MacOS
  // (https://developer.apple.com/documentation/appkit/nswindow/1419660-orderout)
  // but for some reason the previous application does not take focus. Tauri also
  // suffers from this so it's not a regression with using EGUI but instead
  // something else entirely that apps like Alfred & Raycast don't experience.
  // Will need more research.
  // BUG: Linux -> Sometimes (not always) the app doesn't want to revive. It's
  // either the global hotkey has died, or something wrong here. Need more debug
  ctx.send_viewport_cmd_to(ViewportId::ROOT, egui::ViewportCommand::Visible(vis));
  if vis {
    ctx.send_viewport_cmd_to(ViewportId::ROOT, egui::ViewportCommand::Focus);
  }
}
