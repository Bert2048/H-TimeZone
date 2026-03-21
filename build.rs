fn main() {
    // Embed res/htz.ico as the Windows executable icon (shown in Explorer, taskbar, Alt+Tab).
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("res/htz.ico");
        res.compile().unwrap();
    }
}
