use chrono::{Local, NaiveDate, Offset, TimeZone};
use chrono_tz::Tz;
use egui::{FontId, RichText, Ui};

use crate::{colors, helpers::fmt_offset, picker::TzPickerTarget};

// ── ConverterState ────────────────────────────────────────────────────────────

pub struct ConverterState {
    pub from_tz: String,
    pub to_tz: String,
    year: String,
    month: String,
    day: String,
    hour: String,
    minute: String,
    second: String,
    result: Option<String>,
    error: Option<String>,
    /// Run an initial conversion on first draw so the result field is not empty.
    initialized: bool,
}

impl Default for ConverterState {
    fn default() -> Self {
        let now = Local::now();
        Self {
            from_tz: "Asia/Shanghai".to_string(),
            to_tz: "America/New_York".to_string(),
            year: now.format("%Y").to_string(),
            month: now.format("%m").to_string(),
            day: now.format("%d").to_string(),
            hour: now.format("%H").to_string(),
            minute: now.format("%M").to_string(),
            second: now.format("%S").to_string(),
            result: None,
            error: None,
            initialized: false,
        }
    }
}

impl ConverterState {
    /// Draw the converter widget.
    ///
    /// Returns `Some(target)` when the user clicks a timezone button and wants
    /// to open the timezone picker for that field. The caller is responsible for
    /// creating and showing the picker, then calling `set_from_tz` / `set_to_tz`
    /// with the chosen value.
    pub fn draw(&mut self, ui: &mut Ui) -> Option<TzPickerTarget> {
        if !self.initialized {
            self.initialized = true;
            self.do_convert();
        }

        let mut open_picker: Option<TzPickerTarget> = None;

        ui.add_space(4.0);

        // FROM
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("From:").font(FontId::monospace(10.0)).color(colors::DIM),
            );
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(&self.from_tz)
                            .font(FontId::monospace(10.0))
                            .color(colors::PRIMARY),
                    )
                    .frame(false),
                )
                .clicked()
            {
                open_picker = Some(TzPickerTarget::ConverterFrom);
            }
        });

        ui.add_space(2.0);

        // Date fields
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Date:").font(FontId::monospace(10.0)).color(colors::DIM),
            );
            time_field(ui, &mut self.year, 4, "YYYY");
            ui.label(RichText::new("-").font(FontId::monospace(10.0)).color(colors::GHOST));
            time_field(ui, &mut self.month, 2, "MM");
            ui.label(RichText::new("-").font(FontId::monospace(10.0)).color(colors::GHOST));
            time_field(ui, &mut self.day, 2, "DD");
        });

        // Time fields
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Time:").font(FontId::monospace(10.0)).color(colors::DIM),
            );
            time_field(ui, &mut self.hour, 2, "HH");
            ui.label(RichText::new(":").font(FontId::monospace(10.0)).color(colors::GHOST));
            time_field(ui, &mut self.minute, 2, "MM");
            ui.label(RichText::new(":").font(FontId::monospace(10.0)).color(colors::GHOST));
            time_field(ui, &mut self.second, 2, "SS");
        });

        ui.add_space(2.0);

        // TO
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("  To:").font(FontId::monospace(10.0)).color(colors::DIM),
            );
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(&self.to_tz)
                            .font(FontId::monospace(10.0))
                            .color(colors::PRIMARY),
                    )
                    .frame(false),
                )
                .clicked()
            {
                open_picker = Some(TzPickerTarget::ConverterTo);
            }
        });

        ui.add_space(6.0);

        if ui
            .add(
                egui::Button::new(
                    RichText::new("  Convert  ")
                        .font(FontId::monospace(10.0))
                        .color(colors::PRIMARY),
                )
                .frame(true),
            )
            .clicked()
        {
            self.do_convert();
        }

        ui.add_space(6.0);

        if let Some(result) = &self.result {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Result:")
                        .font(FontId::monospace(10.0))
                        .color(colors::DIM),
                );
                ui.label(
                    RichText::new(result).font(FontId::monospace(13.0)).color(colors::GOLD),
                );
            });
        }
        if let Some(err) = &self.error {
            ui.label(RichText::new(err).font(FontId::monospace(10.0)).color(colors::ERROR));
        }

        open_picker
    }

    fn do_convert(&mut self) {
        self.result = None;
        self.error = None;

        let parse_u32 = |s: &str, name: &str| -> Result<u32, String> {
            s.trim().parse::<u32>().map_err(|_| format!("Invalid {name}: '{s}'"))
        };

        let result = (|| -> Result<String, String> {
            let year = self.year.trim().parse::<i32>().map_err(|_| "Invalid year")?;
            let month = parse_u32(&self.month, "month")?;
            let day = parse_u32(&self.day, "day")?;
            let hour = parse_u32(&self.hour, "hour")?;
            let minute = parse_u32(&self.minute, "minute")?;
            let second = parse_u32(&self.second, "second")?;

            let naive = NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|d| d.and_hms_opt(hour, minute, second))
                .ok_or("Invalid date/time values")?;

            let from_tz: Tz = self
                .from_tz
                .parse()
                .map_err(|_| format!("Unknown timezone: {}", self.from_tz))?;
            let to_tz: Tz = self
                .to_tz
                .parse()
                .map_err(|_| format!("Unknown timezone: {}", self.to_tz))?;

            let from_dt = from_tz
                .from_local_datetime(&naive)
                .single()
                .ok_or("Ambiguous or invalid local time (DST transition)")?;

            let to_dt = from_dt.with_timezone(&to_tz);
            let offset_str = fmt_offset(to_dt.offset().fix().local_minus_utc());
            Ok(format!(
                "{}  {}  {}",
                to_dt.format("%Y-%m-%d"),
                to_dt.format("%H:%M"),
                offset_str
            ))
        })();

        match result {
            Ok(s) => self.result = Some(s),
            Err(e) => self.error = Some(e),
        }
    }
}

// ── Widget helper ─────────────────────────────────────────────────────────────

/// Small fixed-width numeric input field that strips non-digit characters.
fn time_field(ui: &mut Ui, value: &mut String, max_chars: usize, hint: &str) {
    let desired_width = max_chars as f32 * 11.0 + 12.0;
    let response = ui.add(
        egui::TextEdit::singleline(value)
            .desired_width(desired_width)
            .hint_text(hint)
            .font(FontId::monospace(11.0)),
    );
    if response.changed() {
        value.retain(|c| c.is_ascii_digit());
        value.truncate(max_chars);
    }
}
