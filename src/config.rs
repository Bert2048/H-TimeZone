use std::collections::HashMap;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Ordered list of IANA timezone names to show as cards.
    #[serde(default = "default_clocks")]
    pub clocks: Vec<String>,
    /// Saved window positions keyed by IANA tz name: [x, y] in screen pixels.
    #[serde(default)]
    pub positions: HashMap<String, [f32; 2]>,
    /// Card background opacity 0.0–1.0 (default 0.70).
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_opacity() -> f32 {
    0.70
}

fn default_clocks() -> Vec<String> {
    [
        "Asia/Shanghai",
        "America/New_York",
        "Europe/London",
        "America/Edmonton",
        "America/Los_Angeles",
        "Asia/Tokyo",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clocks: default_clocks(),
            positions: HashMap::new(),
            opacity: default_opacity(),
        }
    }
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("timezone-tool").join("config.toml"))
}

pub fn load() -> Config {
    config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(cfg: &Config) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(s) = toml::to_string(cfg) {
        let _ = std::fs::write(path, s);
    }
}
