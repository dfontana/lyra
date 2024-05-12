use crate::{
  plugin::{AppState, OkAction, PluginV},
  plugin_manager::PluginManager,
};
use egui::{
  Align, Event, EventFilter, FontId, InputState, Key, Modifiers, TextBuffer, TextEdit, ViewportId,
};
use nucleo_matcher::{
  pattern::{CaseMatching, Pattern},
  Matcher,
};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::error;

use crate::config::{Config, Styles};

#[derive(Clone)]
pub struct LyraPowerbar(Arc<RwLock<LyraPowerbarImpl>>);

impl LyraPowerbar {
  pub fn new(inner: LyraPowerbarImpl) -> LyraPowerbar {
    LyraPowerbar(Arc::new(RwLock::new(inner)))
  }

  pub fn update(&mut self, ctx: &eframe::egui::Context) {
    self.0.write().update(ctx)
  }

  pub fn close(&self, ctx: &eframe::egui::Context, vis: bool) {
    self.0.write().close(ctx, vis)
  }
}

pub struct LyraPowerbarImpl {
  pub state: AppState,
  pub plugins: PluginManager,
  pub config: Arc<Config>,
  pub matcher: RwLock<Matcher>,
}

impl LyraPowerbarImpl {
  pub fn get_options(&self, search: &str) -> Vec<PluginV> {
    if search.is_empty() {
      // Special case, empty string == nothing back instead of everything
      return Vec::new();
    }

    Pattern::parse(search, CaseMatching::Ignore)
      .match_list(
        self
          .plugins
          .filter_to(&search)
          .iter()
          .flat_map(|pl| pl.options(search)),
        &mut *self.matcher.write(),
      )
      .into_iter()
      .take(self.config.get().result_count)
      .map(|(v, _)| v)
      .chain(
        self
          .plugins
          .always_present(search)
          .iter()
          .flat_map(|pl| pl.static_items()),
      )
      .map(|sk| sk.value)
      .collect()
  }

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

  pub fn close(&mut self, ctx: &eframe::egui::Context, vis: bool) {
    // TODO: So both TAO & Winit are issuing an orderOut command on MacOS
    // (https://developer.apple.com/documentation/appkit/nswindow/1419660-orderout)
    // but for some reason the previous application does not take focus. Tauri also
    // suffers from this so it's not a regression with using EGUI but instead
    // something else entirely that apps like Alfred & Raycast don't experience.
    // Will need more research.
    // BUG: Linux -> Sometimes (not always) the app doesn't want to revive. It's
    // either the global hotkey has died, or something wrong here. Need more debug
    if !vis {
      ctx.send_viewport_cmd_to(
        ViewportId::ROOT,
        egui::ViewportCommand::InnerSize(self.config.get().styles.window_size.into()),
      );
      self.reset_state();
    }
    ctx.send_viewport_cmd_to(ViewportId::ROOT, egui::ViewportCommand::Visible(vis));
    if vis {
      ctx.send_viewport_cmd_to(ViewportId::ROOT, egui::ViewportCommand::Focus);
    }
  }

  pub fn update(&mut self, ctx: &eframe::egui::Context) {
    // Window does not play well auto-hiding on focus loss on Linux, so we'll
    // leave it as open until manually closed
    #[cfg(not(target_os = "linux"))]
    if ctx.input(|is| is.events.iter().any(|e| *e == Event::WindowFocused(false))) {
      self.close(ctx, false);
      return;
    }

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
      self.close(ctx, false);
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
            self.close(ctx, false);
            self.reset_state();
          }
          Ok(_) => self.reset_state(),
          Err(e) => error!("{:?}", e),
        }
      }
    }

    let Styles {
      window_size,
      window_rounding,
      window_padding,
      option_margin,
      option_rounding,
      bg_color,
      bg_color_selected,
      text_color,
      text_color_selected,
      font_family,
      font_size,
      ..
    } = self.config.get().styles.clone();

    let window_decor = egui::Frame {
      fill: bg_color,
      rounding: window_rounding,
      stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
      outer_margin: 0.5.into(), // so the stroke is within the bounds
      ..Default::default()
    };

    egui::CentralPanel::default()
      .frame(window_decor)
      .show(ctx, |ui| {
        let padding = window_padding;
        let rect = ui.max_rect().shrink(padding);
        let mut ui = ui.child_ui(rect, *ui.layout());
        ui.visuals_mut().override_text_color = Some(text_color);
        ui.style_mut().override_font_id = Some(FontId::new(font_size, font_family));

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
              self.state.options = self.get_options(&self.state.input);
              self.state.selected = 0;
              self.check_plugins_for_state_updates();
            }
          }

          for (idx, pv) in self.state.options.iter().enumerate() {
            let mut fm = egui::Frame::none()
              .inner_margin(option_margin)
              .rounding(option_rounding);
            if idx == self.state.selected {
              fm = fm.fill(bg_color_selected);
            }
            fm.show(ui, |ui| {
              if idx == self.state.selected {
                ui.style_mut().visuals.override_text_color = Some(text_color_selected);
              }
              pv.render(ui, &self.state);
              ui.set_width(ui.available_width());
            });
          }

          if res.changed() {
            let height = ui.min_rect().height() + (padding * 2.0);
            ctx.send_viewport_cmd_to(
              ViewportId::ROOT,
              egui::ViewportCommand::InnerSize([window_size.0, height].into()),
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
    .margin(egui::Vec2::from((0.0, 2.0)))
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
