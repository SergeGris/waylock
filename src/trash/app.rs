use gtk::{
    Align, Application, ApplicationWindow, Box, Button, Entry, Grid, Label, Orientation, Window, Overlay,
    gdk, glib,
    prelude::*, subclass::prelude::*,
};

use crate::lock;

#[derive(Debug, Clone)]
pub struct App {
    app: Application,
    lock: lock::Lock,

    time: String,
    date: String,
    time_format: String,
    date_format: String,
    config_path: String,
    lock_command: String,
    unlock_command: String,
}

impl App {
    pub fn new<'a>(app: Application) -> App {
        App {
            app: app.clone(),
            lock: lock::Lock::new(&app),

            time: "".to_owned(),
            date: "".to_owned(),
            time_format: "".to_owned(),
            date_format: "".to_owned(),
            config_path: "".to_owned(),
            lock_command: "".to_owned(),
            unlock_command: "".to_owned(),
        }
    }

    pub fn activate(&self) {
        // glib::unix_signal_add(
        //     libc::SIGTERM,
        //     glib::clone!(#[strong(rename_to = app)] self.app, move || { glib::ControlFlow::Break})
        // );
    }

    pub fn shutdown(&self) {
        self.lock.unlock();
    }

// void gtklock_remove_window(struct GtkLock *gtklock, struct Window *win);
// void gtklock_focus_window(struct GtkLock *gtklock, struct Window *win);
// void gtklock_update_clocks(struct GtkLock *gtklock);
// void gtklock_update_dates(struct GtkLock *gtklock);
// void gtklock_idle_hide(struct GtkLock *gtklock);
// void gtklock_idle_show(struct GtkLock *gtklock);
// void gtklock_activate(struct GtkLock *gtklock);
// void gtklock_shutdown(struct GtkLock *gtklock);

}

// use glib::{OptionArg, OptionEntry};

// #[derive(Debug, Default)]
// pub struct Config {
//     pub version: bool,
//     pub config_path: Option<String>,
//     pub daemonize: bool,

//     pub gtk_theme: Option<String>,
//     pub style_path: Option<String>,
//     pub layout_path: Option<String>,
//     pub module_path: Option<Vec<String>>,
//     pub background_path: Option<String>,
//     pub time_format: Option<String>,
//     pub date_format: Option<String>,
//     pub follow_focus: bool,
//     pub idle_hide: bool,
//     pub idle_timeout: Option<i32>,
//     pub start_hidden: bool,
//     pub lock_command: Option<String>,
//     pub unlock_command: Option<String>,
//     pub monitor_priority: Option<Vec<String>>,
// }

// pub fn build_option_entries() -> Vec<OptionEntry> {
//     let e = |long, short, arg, desc| {
//         OptionEntry::new(long, short, 0, arg, desc, None::<&str>)
//     };

//     vec![
//         e("version",         Some('v'), OptionArg::None,          "Show version"),
//         e("config",          Some('c'), OptionArg::String,        "Load config file"),
//         e("daemonize",       Some('d'), OptionArg::None,          "Detach from terminal"),

//         e("gtk-theme",       Some('g'), OptionArg::String,        "GTK theme"),
//         e("style",           Some('s'), OptionArg::String,        "CSS style file"),
//         e("layout",          Some('x'), OptionArg::String,        "XML layout"),
//         e("modules",         Some('m'), OptionArg::StringArray,   "Modules"),
//         e("background",      Some('b'), OptionArg::String,        "Background"),
//         e("time-format",     Some('t'), OptionArg::String,        "Time format"),
//         e("date-format",     Some('D'), OptionArg::String,        "Date format"),

//         e("follow-focus",    Some('f'), OptionArg::None,          "Follow focus"),
//         e("idle-hide",       Some('H'), OptionArg::None,          "Hide on idle"),
//         e("idle-timeout",    Some('T'), OptionArg::Int,           "Idle timeout"),

//         e("start-hidden",    Some('S'), OptionArg::None,          "Start hidden"),
//         e("lock-command",    Some('L'), OptionArg::String,        "Lock command"),
//         e("unlock-command",  Some('U'), OptionArg::String,        "Unlock command"),
//         e("monitor-priority",Some('M'), OptionArg::StringArray,   "Monitor priority"),
//     ]
// }

// IN MAIN

// let entries = build_option_entries();

// app.add_main_option_entries(&entries);

// app.connect_command_line(|app, cmd| {
//     let mut cfg = Config::default();

//     if cmd.is_option_set("version") {
//         cfg.version = true;
//     }

//     cfg.config_path = cmd.option_string("config");
//     cfg.gtk_theme = cmd.option_string("gtk-theme");
//     cfg.style_path = cmd.option_string("style");
//     cfg.layout_path = cmd.option_string("layout");
//     cfg.module_path = cmd.option_strv("modules");
//     cfg.background_path = cmd.option_string("background");
//     cfg.time_format = cmd.option_string("time-format");
//     cfg.date_format = cmd.option_string("date-format");
//     cfg.idle_timeout = cmd.option_int("idle-timeout");

//     // bool flags:
//     cfg.follow_focus = cmd.is_option_set("follow-focus");
//     cfg.idle_hide = cmd.is_option_set("idle-hide");
//     cfg.start_hidden = cmd.is_option_set("start-hidden");
//     cfg.daemonize = cmd.is_option_set("daemonize");

//     app.hold();
//     0
// });
