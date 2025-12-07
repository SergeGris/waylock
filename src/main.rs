// #![warn(clippy::all)]
// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
// #![warn(clippy::restriction)]
// #![warn(clippy::cargo)]
// #![warn(clippy::style)]
// #![warn(clippy::complexity)]
// #![warn(clippy::perf)]
// #![warn(clippy::suspicious)]
// #![warn(clippy::correctness)]
// #![warn(clippy::unwrap_used)]
// #![warn(clippy::expect_used)]
// #![warn(clippy::todo)]
// #![warn(rust_2018_idioms)]
// #![warn(deprecated)]
// #![warn(trivial_casts)]
// #![warn(trivial_numeric_casts)]
// #![warn(unused)]
// #![allow(clippy::implicit_return)]
// #![allow(missing_docs)]
// #![allow(missing_doc_code_examples)]
// #![allow(missing_copy_implementations)]
// #![allow(missing_debug_implementations)]
// #![warn(clippy::all)]
// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
// #![warn(clippy::cargo)]

use clap::Parser as _;
use gtk::{Application, Settings, gio, glib, prelude::*};

mod args;
mod blur;
mod config;
mod css;
mod lock;
mod log;
mod messages;
mod pam;
mod widgets;

#[cfg(feature = "idlenotifier")]
mod idlenotifier;

use widgets::*;

use crate::lock::Lock;

#[must_use]
pub fn daemonize() -> i32 {
    use std::process::exit;

    use libc::{
        SIG_BLOCK,
        SIGUSR2,
        WEXITSTATUS,
        WIFEXITED,
        fork,
        getppid,
        kill,
        setsid,
        sigaddset,
        sigemptyset,
        sigprocmask,
        sigtimedwait,
        timespec,
        waitpid,
    };

    let parent = unsafe { libc::getpid() };

    let err = || std::io::Error::from_raw_os_error(unsafe { *libc::__errno_location() });

    let pid = unsafe { fork() };
    if pid < 0 {
        log::error!("Failed to fork: {}", err());
        exit(1);
    }

    if pid > 0 {
        let mut status = 0;
        unsafe { waitpid(pid, &raw mut status, 0) };

        if WIFEXITED(status) && WEXITSTATUS(status) == 0 {
            let mut set = unsafe { std::mem::zeroed() };

            unsafe {
                sigemptyset(&raw mut set);
                sigaddset(&raw mut set, SIGUSR2);
                sigprocmask(SIG_BLOCK, &raw const set, std::ptr::null_mut());
            }

            // Wait 1 second for SIGUSR2 from grandchild
            let ts = timespec {
                tv_sec: 1,
                tv_nsec: 0,
            };

            let ret = unsafe { sigtimedwait(&raw const set, std::ptr::null_mut(), &raw const ts) };

            if ret == SIGUSR2 {
                exit(0);
            }

            exit(1);
        }

        log::error!("Failed to daemonize: {}", err());
        exit(1);
    }

    if unsafe { setsid() } < 0 {
        exit(1);
    }

    let pid2 = unsafe { fork() };

    if pid2 < 0 {
        exit(1);
    } else if pid2 > 0 {
        exit(0);
    }

    unsafe { kill(getppid(), SIGUSR2) };

    parent
}

fn main() -> glib::ExitCode {
    let mut args = args::Args::parse();

    let parent = if args.daemonize {
        Some(daemonize())
    } else {
        None::<i32>
    };

    if let Some(path) = args.config.get_config() {
        args.config = config::load_config(path).merge(args.config);
    }

    // TODO
    // For many reasons we shall initialize gtk manually and earlier.
    gtk::init().unwrap();

    if let Some(settings) = Settings::default() {
        settings.set_gtk_theme_name(args.config.get_gtk_theme().map(String::as_str));
    }

    println!("{:?}", args.config);

    let app = Application::new(None::<&str>, gio::ApplicationFlags::FLAGS_NONE);
    let lock = Lock::new(&app, parent, &args.config);
    let hld = app.hold();
    app.connect_activate(glib::clone!(
        #[weak]
        lock,
        move |app| activate(app, &lock, &args)
    ));

    app.connect_shutdown(glib::clone!(
        #[weak]
        lock,
        move |_| shutdown(&lock)
    ));

    glib::unix_signal_add_local_once(
        libc::SIGTERM,
        glib::clone!(
            #[weak]
            app,
            move || app.quit()
        ),
    );

    app.run_with_args::<glib::GString>(&[])
}

fn activate(app: &Application, lock: &Lock, args: &args::Args) {
    // let _hold_guard = app.hold(); // TODO

    if !Lock::is_supported() {
        log::fatal!("your compositor does not support ext-session-lock");
    }

    // Get default transition duration
    let revealer = gtk::Revealer::new();
    let duration = revealer.transition_duration();
    drop(revealer);

    attach_style!(
        "
.time {{
    transition: {duration}ms ease-in-out;
    font-size: 48pt;
    font-family: monospace;
}}
window:not(.hidden) .time {{
    font-size: 40pt;
}}
.date {{
    transition: {duration}ms ease-in-out;
    font-size: 24pt;
    font-family: monospace;
}}
window:not(.hidden) .date {{
    font-size: 20pt;
}}

.error-label {{
    color: red;
}}
"
    );

    //         css::attach_style(
    //             "
    // background {
    //     background-image: linear-gradient(to bottom right, #0A0A4B, #4B0A0A);
    // }
    // ",
    //         );

    if let Some(style_path) = args.config.get_style() {
        css::attach_custom_style(style_path);
    }

    if true {
        lock.enlock();
    } else {
        let w = window::LockWindow::builder()
            .application(app)
            .lock(&lock.0)
            .start_hidden(args.config.get_start_hidden())
            .idle_timeout(args.config.get_idle_timeout())
            .time_format(args.config.get_time_format())
            .date_format(args.config.get_date_format())
            .background(args.config.get_background())
            .build();

        w.present();
    }
}

fn shutdown(lock: &lock::Lock) {
    //lock.unlock();
}
