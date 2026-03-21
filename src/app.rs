use chrono::{DateTime, Datelike};
use chrono_tz::Tz;
use egui::{FontId, RichText};
use std::collections::{HashMap, HashSet};

use crate::{
    clock::{self, ClockEntry, CARD_GAP, CARD_H, CARD_W},
    colors,
    config,
    converter::ConverterState,
    helpers::city_name,
    picker::{TzPickerState, TzPickerTarget},
};

// ── TimeZoneApp ───────────────────────────────────────────────────────────────

pub struct TimeZoneApp {
    clocks: Vec<ClockEntry>,
    show_settings: bool,
    picker: Option<TzPickerState>,
    converter: ConverterState,
    /// Saved window positions per tz_name (loaded from config, updated on drag).
    positions: HashMap<String, egui::Pos2>,
    /// Cards that have had `with_position()` applied this session.
    /// Separate from `positions` so OS-managed dragging is not overridden.
    shown: HashSet<String>,
    /// Card background opacity 0.0–1.0.
    opacity: f32,
    /// Whether card windows float always-on-top.
    pinned: bool,
    /// True when state changed and config needs re-saving.
    config_dirty: bool,
    /// Quit requested from settings or tray — applied at end of frame.
    want_quit: bool,
    /// Tray menu item IDs for event matching.
    quit_id: String,
    settings_id: String,
    /// App icon applied to decorated viewports (settings, picker).
    icon: Option<egui::IconData>,
    /// Frames remaining to call DWM transparency fix (Windows only).
    dwm_pending: u8,
}

impl TimeZoneApp {
    pub fn new(quit_id: String, settings_id: String) -> Self {
        let cfg = config::load();
        let clocks = cfg.clocks.iter().map(|s| ClockEntry::new(s)).collect();
        let positions = cfg
            .positions
            .iter()
            .map(|(k, &[x, y])| (k.clone(), egui::pos2(x, y)))
            .collect();
        let icon = Self::load_icon();
        Self {
            clocks,
            show_settings: false,
            picker: None,
            converter: ConverterState::default(),
            positions,
            shown: HashSet::new(),
            opacity: cfg.opacity,
            pinned: cfg.pinned,
            config_dirty: false,
            want_quit: false,
            quit_id,
            settings_id,
            icon,
            dwm_pending: 1,
        }
    }

    fn load_icon() -> Option<egui::IconData> {
        let bytes = include_bytes!("../res/htz.ico");
        let img = image::load_from_memory(bytes).ok()?.into_rgba8();
        let width = img.width();
        let height = img.height();
        Some(egui::IconData { rgba: img.into_raw(), width, height })
    }

    fn save_config(&mut self) {
        let positions = self
            .positions
            .iter()
            .map(|(k, p)| (k.clone(), [p.x, p.y]))
            .collect();
        config::save(&config::Config {
            clocks: self.clocks.iter().map(|c| c.tz_name.clone()).collect(),
            positions,
            opacity: self.opacity,
            pinned: self.pinned,
        });
        self.config_dirty = false;
    }

    // ── Settings panel ────────────────────────────────────────────────────────
    //
    //  ─── CITIES ──────────────────────────────
    //   [✕] Shanghai   (Asia/Shanghai)
    //   [✕] New York   (America/New_York)
    //   [⊕  Add Timezone]
    //
    //  ─── APPEARANCE ──────────────────────────
    //   Opacity  [━━━━━━━━━━━] 0.70
    //
    //  ─── CONVERTER ───────────────────────────
    //   From: [Asia/Shanghai]
    //   Date: [2026]-[03]-[20]  Time: [10]:[16]:[00]
    //   To:   [America/New_York]
    //         [Convert]
    //   Result: 2026-03-20  22:16  -04:00
    //
    //  ─────────────────────────────────────────
    //  [ Quit ]

    fn draw_settings_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
        self.draw_settings_inner(ui);
        });
    }

    fn draw_settings_inner(&mut self, ui: &mut egui::Ui) {
        // ── CITIES ────────────────────────────────────────────────────────────
        ui.label(
            RichText::new("CITIES").font(FontId::monospace(9.0)).color(colors::GHOST),
        );
        ui.add(egui::Separator::default().spacing(6.0));

        let mut remove_idx: Option<usize> = None;
        for (i, clock) in self.clocks.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("✕")
                                .font(FontId::monospace(9.0))
                                .color(colors::ERROR),
                        )
                        .frame(true),
                    )
                    .clicked()
                {
                    remove_idx = Some(i);
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new(city_name(&clock.tz_name))
                        .font(FontId::monospace(11.0))
                        .color(colors::PRIMARY),
                );
                ui.label(
                    RichText::new(format!("({})", clock.tz_name))
                        .font(FontId::monospace(9.0))
                        .color(colors::GHOST),
                );
            });
            ui.add_space(2.0);
        }

        if let Some(idx) = remove_idx {
            let tz = self.clocks.remove(idx).tz_name;
            self.positions.remove(&tz);
            self.config_dirty = true;
        }

        ui.add_space(4.0);
        if ui
            .add(
                egui::Button::new(
                    RichText::new("\u{2795}  Add Timezone")
                        .font(FontId::monospace(10.0))
                        .color(colors::DIM),
                )
                .frame(false),
            )
            .clicked()
        {
            self.picker = Some(TzPickerState::new(TzPickerTarget::Clock));
        }

        ui.add_space(12.0);

        // ── APPEARANCE ────────────────────────────────────────────────────────
        ui.label(
            RichText::new("APPEARANCE").font(FontId::monospace(9.0)).color(colors::GHOST),
        );
        ui.add(egui::Separator::default().spacing(6.0));

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Opacity").font(FontId::monospace(10.0)).color(colors::DIM),
            );
            let prev = self.opacity;
            ui.add(
                egui::Slider::new(&mut self.opacity, 0.1_f32..=1.0_f32)
                    .fixed_decimals(2)
                    .text(""),
            );
            if (self.opacity - prev).abs() > 0.001 {
                self.config_dirty = true;
            }
        });

        ui.add_space(4.0);
        if ui
            .add(egui::Checkbox::new(
                &mut self.pinned,
                RichText::new("Always on top").font(FontId::monospace(10.0)).color(colors::DIM),
            ))
            .changed()
        {
            self.config_dirty = true;
        }

        ui.add_space(12.0);

        // ── CONVERTER ─────────────────────────────────────────────────────────
        ui.label(
            RichText::new("CONVERTER").font(FontId::monospace(9.0)).color(colors::GHOST),
        );
        ui.add(egui::Separator::default().spacing(6.0));

        // converter.draw() returns Some(target) when the user clicks a From/To button
        if let Some(target) = self.converter.draw(ui) {
            self.picker = Some(TzPickerState::new(target));
        }

        ui.add_space(16.0);
        ui.add(egui::Separator::default().spacing(6.0));

        // ── QUIT ──────────────────────────────────────────────────────────────
        if ui
            .add(
                egui::Button::new(
                    RichText::new("Quit").font(FontId::monospace(10.0)).color(colors::ERROR),
                )
                .frame(true),
            )
            .clicked()
        {
            self.want_quit = true;
        }
    }
}

// ── eframe::App ───────────────────────────────────────────────────────────────

impl eframe::App for TimeZoneApp {
    /// Clear to fully transparent so per-pixel alpha blends against the desktop
    /// via the DWM compositor on Windows.  Without this, the default opaque
    /// clear color prevents window transparency even when `with_transparent(true)`
    /// is set on the viewport.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        // eframe's clear_color() override zeros the root viewport's framebuffer.
        // Child immediate viewports use `visuals.panel_fill` as their clear colour,
        // so we set it to TRANSPARENT here.  Every panel that needs a visible fill
        // supplies its own explicit Frame::fill(), so nothing breaks.
        ctx.style_mut(|s| s.visuals.panel_fill = egui::Color32::TRANSPARENT);

        // Root window: invisible 1×1 host for child viewports
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |_ui| {});

        // Poll system tray menu events
        while let Ok(ev) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if ev.id.0 == self.quit_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if ev.id.0 == self.settings_id {
                self.show_settings = true;
            }
        }

        let now_utc = chrono::Utc::now();
        let n = self.clocks.len();
        let mut open_settings = false;
        // Position updates collected outside viewport closures to avoid borrow conflicts.
        let mut pos_updates: Vec<(String, egui::Pos2)> = Vec::new();

        // ── City card viewports ───────────────────────────────────────────────
        //
        //  Each city is an independent OS window (immediate viewport).
        //  Positions are applied via with_position() on first render only;
        //  after that the OS manages the window and we just read the position.
        //
        //  ┌──────────────────────┐
        //  │ Shanghai    Mar 20   │  ← city + date  DIM 10px
        //  │ 14:30                │  ← time         PRIMARY 20px
        //  └──────────────────────┘

        for i in 0..n {
            let tz_name = self.clocks[i].tz_name.clone();
            let tz: Option<Tz> = tz_name.parse().ok();

            let (time_str, date_str) = tz.as_ref().map_or(
                ("??:??".to_string(), "---".to_string()),
                |tz| {
                    let dt: DateTime<Tz> = now_utc.with_timezone(tz);
                    (dt.format("%H:%M").to_string(), format!("{} {}", dt.format("%b"), dt.day()))
                },
            );

            let city = city_name(&tz_name);
            let vp_id = egui::ViewportId::from_hash_of(&tz_name);

            // `shown` tracks whether with_position() has been applied this session.
            let first_render = !self.shown.contains(&tz_name);
            if first_render {
                self.shown.insert(tz_name.clone());
            }

            let apply_pos = if first_render {
                Some(self.positions.get(&tz_name).copied().unwrap_or_else(|| {
                    let p = egui::pos2(60.0, 60.0 + i as f32 * (CARD_H + CARD_GAP));
                    self.positions.insert(tz_name.clone(), p);
                    self.config_dirty = true;
                    p
                }))
            } else {
                None
            };

            let pinned = self.pinned;
            let mut vb = egui::ViewportBuilder::default()
                .with_title(city.as_str())
                .with_decorations(false)
                .with_transparent(true)
                .with_inner_size([CARD_W, CARD_H])
                .with_resizable(false)
                .with_taskbar(false);

            if pinned {
                vb = vb.with_always_on_top();
            }

            if let Some(pos) = apply_pos {
                vb = vb.with_position(pos);
            }

            let mut tracked_pos: Option<egui::Pos2> = None;

            ctx.show_viewport_immediate(vp_id, vb, |ctx, _class| {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if pinned {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                }));
                let (double_clicked, pos) =
                    clock::draw_card_content(ctx, &city, &time_str, &date_str, self.opacity);
                if double_clicked {
                    open_settings = true;
                }
                tracked_pos = pos;
            });

            if let Some(p) = tracked_pos {
                pos_updates.push((tz_name, p));
            }
        }

        // Apply position updates — persist config if anything moved
        for (tz_name, new_pos) in pos_updates {
            if self.positions.get(&tz_name) != Some(&new_pos) {
                self.positions.insert(tz_name, new_pos);
                self.config_dirty = true;
            }
        }

        if open_settings {
            self.show_settings = true;
        }

        if self.config_dirty {
            self.save_config();
        }

        if self.want_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // ── Settings viewport ─────────────────────────────────────────────────
        if self.show_settings {
            let mut close_settings = false;

            let pinned = self.pinned;
            let mut settings_vb = egui::ViewportBuilder::default()
                .with_title("TimeZone — Settings")
                .with_inner_size([340.0, 500.0])
                .with_resizable(false)
                .with_taskbar(false)
                .with_transparent(true);
            if pinned {
                settings_vb = settings_vb.with_always_on_top();
            }
            if let Some(ref icon) = self.icon {
                settings_vb = settings_vb.with_icon(icon.clone());
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("settings"),
                settings_vb,
                |ctx, _class| {
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if pinned {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    }));
                    if ctx.input(|i| i.viewport().close_requested()) {
                        close_settings = true;
                    }

                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::new()
                                .fill(colors::settings_bg())
                                .inner_margin(egui::Margin {
                                    left: 14,
                                    right: 14,
                                    top: 12,
                                    bottom: 12,
                                }),
                        )
                        .show(ctx, |ui| {
                            self.draw_settings_ui(ui);
                        });

                    // Picker window floats within the settings viewport.
                    // Collect result first to drop the picker borrow before mutating app state.
                    let picker_result = self
                        .picker
                        .as_mut()
                        .map(|p| (p.target, p.draw_window(ctx, self.icon.clone())));

                    if let Some((target, (chosen, close))) = picker_result {
                        if let Some(tz) = chosen {
                            match target {
                                TzPickerTarget::Clock => {
                                    self.positions.remove(&tz);
                                    self.clocks.push(ClockEntry::new(&tz));
                                    self.config_dirty = true;
                                    // Re-apply DWM fix this frame and the next to cover
                                    // the newly created card viewport.
                                    self.dwm_pending = self.dwm_pending.max(2);
                                }
                                TzPickerTarget::ConverterFrom => {
                                    self.converter.from_tz = tz;
                                }
                                TzPickerTarget::ConverterTo => {
                                    self.converter.to_tz = tz;
                                }
                            }
                        }
                        if close {
                            self.picker = None;
                        }
                    }
                },
            );

            if close_settings {
                self.show_settings = false;
            }
        }

        // ── Windows DWM per-pixel alpha fix ───────────────────────────────────
        // winit calls DwmEnableBlurBehindWindow with DWM_BB_BLURREGION + empty
        // region at window creation, but the correct call for GPU-rendered
        // transparent windows is DWM_BB_ENABLE only with a NULL region.
        // We re-apply it here for all process windows after viewports are shown.
        #[cfg(windows)]
        if self.dwm_pending > 0 {
            Self::apply_dwm_transparency();
            self.dwm_pending -= 1;
        }
    }
}

// ── Windows DWM helper ────────────────────────────────────────────────────────

#[cfg(windows)]
impl TimeZoneApp {
    /// Re-apply DwmEnableBlurBehindWindow with the correct DWM_BB_ENABLE flag
    /// (no blur region) for all windows belonging to this process.
    /// This ensures per-pixel alpha compositing works for undecorated viewports.
    fn apply_dwm_transparency() {
        use windows_sys::Win32::{
            Foundation::BOOL,
            Graphics::Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE, DWM_BLURBEHIND},
            System::Threading::GetCurrentProcessId,
            UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId},
        };

        unsafe extern "system" fn enum_cb(hwnd: isize, lparam: isize) -> BOOL {
            let target_pid = lparam as u32;
            let mut wpid: u32 = 0;
            // SAFETY: hwnd is a valid HWND supplied by the OS callback;
            // wpid is a local stack variable valid for the duration of this call.
            unsafe { GetWindowThreadProcessId(hwnd, &mut wpid) };
            if wpid == target_pid {
                let bb = DWM_BLURBEHIND {
                    dwFlags: DWM_BB_ENABLE,
                    fEnable: 1,   // TRUE
                    hRgnBlur: 0,  // NULL — apply DWM compositing to entire window
                    fTransitionOnMaximized: 0,
                };
                // SAFETY: hwnd is a valid HWND from the OS; bb is fully initialized
                // on the stack and lives for the duration of this call.
                // Ignore HRESULT: failure is non-fatal (e.g. DWM composition
                // disabled in a VM); transparency simply won't apply to that window.
                let _ = unsafe { DwmEnableBlurBehindWindow(hwnd, &bb) };
            }
            1 // TRUE — continue enumeration
        }

        // SAFETY: enum_cb is a valid WNDENUMPROC; pid is u32 cast to isize
        // (zero-extended on 64-bit Windows; PIDs never exceed i32::MAX in practice).
        unsafe {
            let pid = GetCurrentProcessId();
            EnumWindows(Some(enum_cb), pid as isize);
        }
    }
}
