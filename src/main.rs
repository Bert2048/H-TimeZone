#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod clock;
mod colors;
mod config;
mod converter;
mod helpers;
mod picker;

const APP_NAME: &str = "H-TimeZone";

fn main() -> eframe::Result {
    let transparent = !std::env::args().any(|a| a == "--no-transparency");

    // Tray icon: the only way to quit / open settings (no taskbar entry)
    let (quit_id, settings_id) = setup_tray();

    // Root window: invisible 1×1 host — cards and settings are child viewports
    let vb = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_inner_size([1.0, 1.0])
        .with_min_inner_size([1.0, 1.0])
        .with_decorations(false)
        .with_transparent(transparent)
        .with_taskbar(false);

    let options = eframe::NativeOptions {
        viewport: vb,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |_cc| Ok(Box::new(app::TimeZoneApp::new(quit_id, settings_id)))),
    )
}

/// Creates the system tray icon with Settings + Quit menu.
/// Returns (quit_id, settings_id) as Strings for event matching.
fn setup_tray() -> (String, String) {
    use tray_icon::menu::{Menu, MenuItem};
    use tray_icon::TrayIconBuilder;

    let tray_menu = Menu::new();
    let settings_item = MenuItem::new("Settings", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    let settings_id = settings_item.id().0.clone();
    let quit_id = quit_item.id().0.clone();

    tray_menu.append(&settings_item).ok();
    tray_menu.append(&quit_item).ok();

    // Load tray icon from the embedded ICO file.
    let icon = {
        let bytes = include_bytes!("../res/htz.ico");
        let img = image::load_from_memory(bytes)
            .expect("tray icon decode")
            .into_rgba8();
        let (w, h) = img.dimensions();
        tray_icon::Icon::from_rgba(img.into_raw(), w, h).expect("tray icon")
    };

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_icon(icon)
        .with_tooltip(APP_NAME)
        .build()
        .expect("tray icon build");

    // Keep tray alive for the entire process lifetime
    std::mem::forget(tray);

    (quit_id, settings_id)
}
