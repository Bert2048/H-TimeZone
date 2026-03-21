use egui::{FontId, RichText};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{colors, helpers::all_tz_names};

// ── TzPickerTarget ────────────────────────────────────────────────────────────

/// Identifies which field the user is picking a timezone for.
#[derive(Clone, Copy, PartialEq)]
pub enum TzPickerTarget {
    Clock,
    ConverterFrom,
    ConverterTo,
}

// ── TzPickerState ─────────────────────────────────────────────────────────────

pub struct TzPickerState {
    pub target: TzPickerTarget,
    query: String,
    filtered: Vec<String>,
    /// False until the search field has been focused on first open.
    focused: bool,
}

impl TzPickerState {
    pub fn new(target: TzPickerTarget) -> Self {
        Self {
            target,
            query: String::new(),
            filtered: all_tz_names().iter().map(|s| s.to_string()).collect(),
            focused: false,
        }
    }

    fn title(&self) -> &'static str {
        match self.target {
            TzPickerTarget::Clock => "Add Clock",
            TzPickerTarget::ConverterFrom => "From — Select Timezone",
            TzPickerTarget::ConverterTo => "To — Select Timezone",
        }
    }

    fn update_filter(&mut self) {
        let matcher = SkimMatcherV2::default();
        if self.query.is_empty() {
            self.filtered = all_tz_names().iter().map(|s| s.to_string()).collect();
        } else {
            let q = &self.query;
            let mut scored: Vec<(i64, String)> = all_tz_names()
                .iter()
                .filter_map(|tz| matcher.fuzzy_match(tz, q).map(|s| (s, tz.to_string())))
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered = scored.into_iter().map(|(_, s)| s).collect();
        }
    }

    /// Draw the picker as an independent OS window (immediate viewport).
    ///
    /// The viewport carries a title bar with the OS close (X) button.
    /// Returns `(chosen_tz_name, should_close)`.
    pub fn draw_window(&mut self, ctx: &egui::Context) -> (Option<String>, bool) {
        let mut close = false;
        let mut chosen: Option<String> = None;

        let search_id = egui::Id::new("tz_picker_search");
        let title = self.title();

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("tz_picker"),
            egui::ViewportBuilder::default()
                .with_title(title)
                .with_inner_size([300.0, 400.0])
                .with_resizable(false)
                .with_always_on_top()
                .with_taskbar(false),
            |ctx, _class| {
                // OS title-bar X button — let the OS close the window and
                // signal the caller to clear picker state.
                if ctx.input(|i| i.viewport().close_requested()) {
                    close = true;
                }

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(colors::settings_bg())
                            .inner_margin(egui::Margin {
                                left: 12,
                                right: 12,
                                top: 10,
                                bottom: 10,
                            }),
                    )
                    .show(ctx, |ui| {
                        // ── Search bar ────────────────────────────────────────
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.query)
                                .id(search_id)
                                .desired_width(f32::INFINITY)
                                .hint_text("Search timezones…"),
                        );

                        // Auto-focus on first open.
                        if !self.focused {
                            resp.request_focus();
                            self.focused = true;
                        }

                        let query_changed = resp.changed();
                        if query_changed {
                            self.update_filter();
                        }

                        ui.add_space(4.0);

                        // ── Keyboard shortcuts ────────────────────────────────
                        // Enter → select top result.
                        if ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && !self.filtered.is_empty()
                        {
                            chosen = Some(self.filtered[0].clone());
                            close = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        // Escape → cancel.
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            close = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        ui.separator();

                        // ── Results list ──────────────────────────────────────
                        egui::ScrollArea::vertical().max_height(310.0).show(ui, |ui| {
                            if self.filtered.is_empty() {
                                ui.add_space(12.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "No timezones match \"{}\"",
                                            self.query
                                        ))
                                        .font(FontId::monospace(10.0))
                                        .color(colors::DIM),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(
                                            "Try a city like \"Edmonton\" or \
                                             a region like \"America\"",
                                        )
                                        .font(FontId::monospace(9.0))
                                        .color(colors::GHOST),
                                    );
                                });
                            } else {
                                for tz in &self.filtered.clone() {
                                    if ui
                                        .selectable_label(
                                            false,
                                            RichText::new(tz)
                                                .font(FontId::monospace(11.0))
                                                .color(colors::DIM),
                                        )
                                        .clicked()
                                    {
                                        chosen = Some(tz.clone());
                                        close = true;
                                        ui.ctx()
                                            .send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                }
                            }
                        });
                    });
            },
        );

        (chosen, close)
    }
}
