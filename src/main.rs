#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
// #![warn(clippy::restriction)]   // WARNING: extremely strict; see notes below

// use clap::Parser;
use clap::Parser;
use gtk::{Application, glib, prelude::*};

mod args;
mod clock;
mod config;
mod css;
mod lock;
mod pam;
mod password_entry;
#[cfg(feature = "playerctl")]
mod playerctl;
mod powerbar;
#[cfg(feature = "userinfo")]
mod user;
mod window;

// const APP_ID: &str = ""; // TODO HUI

pub fn daemonize() -> i32 {
    use std::process::exit;

    use libc::{
        SIG_BLOCK, SIGUSR2, WEXITSTATUS, WIFEXITED, fork, setsid, sigaddset, sigemptyset,
        sigprocmask, sigtimedwait, timespec, waitpid,
    };

    let parent = unsafe { libc::getpid() };

    unsafe {
        // --- FIRST FORK ---
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

        // --- GRANDCHILD (ACTUAL DAEMON) ---
        // At this point you should:
        //   • change working directory
        //   • redirect stdio
        //   • reset umask
        //   • send SIGUSR2 to parent if needed

        // Example: signal parent daemon is ready
        libc::kill(libc::getppid(), SIGUSR2);
    }

    parent
}

fn main() -> glib::ExitCode {
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(msg) = info.payload().downcast_ref::<String>() {
            msg.clone()
        } else if let Some(msg) = info.payload().downcast_ref::<&str>() {
            msg.to_string()
        } else {
            "unknown panic".into()
        };

        eprintln!("panic: {msg}");
        // std::panic::resume_unwind(err);
        //Err(anyhow::Error::msg(msg))
    }));

    let mut args = args::Args::parse();

    let parent = if args.daemonize {
        Some(daemonize())
    } else {
        None::<i32>
    };

    if let Some(path) = args.config.get_config() {
        args.config = config::load_config(path).merge(args.config);
    }

    println!("{}", config::default_config());

    // TODO
    gtk::init().unwrap();

    let app = Application::builder()
        .flags(gtk::gio::ApplicationFlags::FLAGS_NONE)
        //.application_id("com.example.LockWindow")
        .build();

    // if let Some(name) = args.name {
    //     glib::set_application_name("Waylock");
    // }

    let lock = lock::Lock::new(&app, parent, &args.config);

    app.connect_activate(glib::clone!(
        #[strong]
        lock,
        move |app| activate(app, &lock, &args)
    ));
    app.connect_shutdown(glib::clone!(
        #[strong]
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

    // app.run()
    app.run_with_args::<glib::GString>(&[]) // TODO??   app.run()
}

fn activate(_app: &Application, lock: &lock::Lock, args: &args::Args) {
    println!("Activate");

    if let Some(path) = args.config.get_background()
        && let Ok(ref uri) =
            url::Url::from_file_path(path.canonicalize().unwrap_or_else(|_| path.clone()))
    {
        let s = uri.as_str();

        attach_style!(
            r#"
.background {{
    background-image: url("{s}");
    background-size: cover;
}}"#
        );
    } else {
        attach_style!(
            r"
.background {{
    background-image: linear-gradient(to bottom right, #00004B, #4B0000);
}}"
        );
    }

    attach_style!(
        r"
.window #clock-label {{
    font-size: 80pt;
    font-family: monospace;
}}
.window.focused:not(.hidden) #clock-label {{
    font-size: 32pt;
    font-family: monospace;
}}
.window #date-label {{
    font-size: 28px;
}}
.window.focused:not(.hidden) #date-label {{
    font-size: 12pt;
}}
.error-label {{
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
    println!("Shutdown");
    lock.unlock();
}
