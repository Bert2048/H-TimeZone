use egui::Color32;

pub const PRIMARY: Color32 = Color32::from_rgb(255, 195, 95); // amber
pub const DIM: Color32 = Color32::from_rgb(122, 92, 42); // #7A5C2A
pub const GHOST: Color32 = Color32::from_rgb(74, 56, 32); // #4A3820
pub const GOLD: Color32 = Color32::from_rgb(255, 215, 0); // #FFD700
pub const ERROR: Color32 = Color32::from_rgb(255, 85, 85); // #FF5555

pub fn card_bg(opacity: f32) -> Color32 {
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    Color32::from_rgba_unmultiplied(10, 8, 6, alpha)
}

pub fn settings_bg() -> Color32 {
    Color32::from_rgba_unmultiplied(12, 10, 7, 230)
}
