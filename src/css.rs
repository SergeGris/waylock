
/// Load CSS from a formatted string
pub fn attach_style_fmt(fmt: std::fmt::Arguments) {
    let css = fmt.to_string();

    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Load CSS from a file path
pub fn attach_custom_style(path: impl AsRef<std::path::Path>) {
    let provider = gtk::CssProvider::new();

    provider.load_from_path(path);

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }
}

#[macro_export]
macro_rules! attach_style {
    ($($arg:tt)*) => {{
        $crate::css::attach_style_fmt(format_args!($($arg)*));
    }};
}
