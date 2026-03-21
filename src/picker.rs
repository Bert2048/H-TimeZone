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

    /// Draw the picker as a floating window within `ctx`.
    ///
    /// Returns `(chosen_tz_name, should_close)`.
    /// - `chosen_tz_name` is `Some` when the user selects a timezone.
    /// - `should_close` is `true` on selection or when the user dismisses.
    pub fn draw_window(&mut self, ctx: &egui::Context) -> (Option<String>, bool) {
        let mut close = false;
        let mut chosen: Option<String> = None;

        let search_id = egui::Id::new("tz_picker_search");

        egui::Window::new(self.title())
            .collapsible(false)
            .resizable(true)
            .default_size([300.0, 340.0])
            .show(ctx, |ui| {
                let before = self.query.clone();

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("\u{1F50D}")
                            .font(FontId::monospace(11.0))
                            .color(colors::DIM),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .id(search_id)
                            .desired_width(f32::INFINITY),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("\u{2715}")
                                    .font(FontId::monospace(10.0))
                                    .color(colors::GHOST),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        close = true;
                    }
                });

                // Auto-focus the search field on first open.
                if !self.focused {
                    ui.memory_mut(|m| m.request_focus(search_id));
                    self.focused = true;
                }

                if self.query != before {
                    self.update_filter();
                }

                // Enter key selects the top result.
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.filtered.is_empty() {
                    chosen = Some(self.filtered[0].clone());
                    close = true;
                }

                ui.separator();

                egui::ScrollArea::vertical().max_height(270.0).show(ui, |ui| {
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
                                    "Try a city like \"Edmonton\" or a region like \"America\"",
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
                            }
                        }
                    }
                });
            });

        (chosen, close)
    }
}
