use egui::{FontId, RichText};

use crate::colors;

// ── Card viewport dimensions ──────────────────────────────────────────────────

pub const CARD_W: f32 = 128.0;
pub const CARD_H: f32 = 48.0; // city row + time/date row
pub const CARD_GAP: f32 = 6.0; // vertical gap between stacked default positions

// ── ClockEntry ────────────────────────────────────────────────────────────────

/// A single city clock tracked by the app.
#[derive(Clone)]
pub struct ClockEntry {
    pub tz_name: String,
}

impl ClockEntry {
    pub fn new(tz_name: &str) -> Self {
        Self { tz_name: tz_name.to_string() }
    }
}

// ── Card content drawing ──────────────────────────────────────────────────────

/// Draw the contents of a city card inside its viewport.
///
/// Layout:
/// ```text
/// ┌──────────────────────┐
/// │ Shanghai    Mar 20   │  ← city (DIM 10px) + date (DIM 10px right-aligned)
/// │ 14:30                │  ← time (PRIMARY 20px)
/// └──────────────────────┘
/// ```
///
/// Returns `(double_clicked, current_window_pos)`.
/// - `double_clicked` → caller should open the settings window.
/// - `current_window_pos` → caller should track for position persistence.
pub fn draw_card_content(
    ctx: &egui::Context,
    city: &str,
    time_str: &str,
    date_str: &str,
    opacity: f32,
) -> (bool, Option<egui::Pos2>) {
    let tracked_pos = ctx.input(|i| i.viewport().inner_rect.map(|r| r.min));
    let mut double_clicked = false;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(colors::card_bg(opacity))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin { left: 10, right: 8, top: 5, bottom: 5 }),
        )
        .show(ctx, |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            ui.spacing_mut().item_spacing.y = 2.0;

            // Full-card drag + double-click surface
            let card_resp = ui.interact(
                ui.max_rect(),
                egui::Id::new("card"),
                egui::Sense::click_and_drag(),
            );
            if card_resp.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if card_resp.double_clicked() {
                double_clicked = true;
            }

            // Row 1: city name
            ui.add(
                egui::Label::new(
                    RichText::new(city).font(FontId::monospace(10.0)).color(colors::DIM),
                )
                .selectable(false),
            );

            // Row 2: time (left) + date (right-aligned)
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(time_str)
                            .font(FontId::monospace(20.0))
                            .color(colors::PRIMARY),
                    )
                    .selectable(false),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(date_str)
                                .font(FontId::monospace(10.0))
                                .color(colors::DIM),
                        )
                        .selectable(false),
                    );
                });
            });
        });

    (double_clicked, tracked_pos)
}
