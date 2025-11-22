
use clap::Parser;
use gtk::{Application, glib, prelude::*};

mod args;
mod config;
mod css;
mod lock;
mod pam;
mod widgets;

use widgets::*;

#[must_use]
pub fn daemonize() -> i32 {
    use std::process::exit;

    use libc::{
        SIG_BLOCK,
        SIGUSR2,
        WEXITSTATUS,
        WIFEXITED,
        fork,
        setsid,
        sigaddset,
        sigemptyset,
        sigprocmask,
        sigtimedwait,
        timespec,
        waitpid,
    };

    let parent = unsafe { libc::getpid() };

    unsafe {
        let pid = fork();
        if pid < 0 {
            eprintln!("Failed to daemonize");
            exit(1);
        }

        // Parent waits for first child
        if pid > 0 {
            let mut status = 0;
            waitpid(pid, &raw mut status, 0);

            if WIFEXITED(status) && WEXITSTATUS(status) == 0 {
                let mut set = std::mem::zeroed();
                sigemptyset(&raw mut set);
                sigaddset(&raw mut set, SIGUSR2);
                sigprocmask(SIG_BLOCK, &raw const set, std::ptr::null_mut());

                // Wait 1 second for SIGUSR2 from grandchild
                let ts = timespec {
                    tv_sec: 1,
                    tv_nsec: 0,
                };

                let ret = sigtimedwait(&raw const set, std::ptr::null_mut(), &raw const ts);

                if ret == SIGUSR2 {
                    exit(0);
                }

                exit(1);
            }

            eprintln!("Failed to daemonize");
            exit(1);
        }

        if setsid() < 0 {
            exit(1);
        }

        let pid2 = fork();
        if pid2 < 0 {
            exit(1);
        } else if pid2 > 0 {
            exit(0);
        }

        libc::kill(libc::getppid(), SIGUSR2);
    }

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

    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_theme_name(args.config.get_gtk_theme().map(String::as_str));
    }

    let app = Application::builder()
        .flags(gtk::gio::ApplicationFlags::FLAGS_NONE)
        .build();

    let lock = lock::Lock::new(&app, parent, &args.config);

    app.connect_activate(glib::clone!(
        #[weak]
        lock,
        move |_| activate(&lock, &args)
    ));
    app.connect_shutdown(glib::clone!(
        #[weak]
        lock,
        move |_| shutdown(&lock)
    ));

    glib::unix_signal_add_local(
        libc::SIGTERM,
        glib::clone!(
            #[strong]
            app,
            move || {
                app.quit();
                glib::ControlFlow::Break
            }
        ),
    );

    app.run_with_args::<glib::GString>(&[])
}

fn activate(lock: &lock::Lock, args: &args::Args) {
    // Get default transition duration
    let revealer = gtk::Revealer::new();
    let duration = revealer.transition_duration();
    drop(revealer);

    attach_style!(
        r"
#window .clock-label {{
    transition: {duration}ms ease-in-out;
    font-size: 64pt;
    font-family: monospace;
}}
#window.focused:not(.hidden) .clock-label {{
    font-size: 48pt;
}}
#window .date-label {{
    transition: {duration}ms ease-in-out;
    font-size: 16pt;
    font-family: monospace;
}}
#window.focused:not(.hidden) .date-label {{
    font-size: 12pt;
    font-family: monospace;
}}

#error-label {{
    color: red;
}}
"
    );

    if let Some(style_path) = args.config.get_style() {
        css::attach_custom_style(style_path);
    }

    lock.enlock();
}

fn shutdown(lock: &lock::Lock) {
    lock.unlock();
}
