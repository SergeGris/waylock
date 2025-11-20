use gtk::{Application, gdk, glib, prelude::*};

use crate::{config, window::LockWindow};

#[derive(Clone, glib::Downgrade, Debug, Default)]
pub struct Lock(gtk_session_lock::Instance);

impl Lock {
    fn locked(app: &Application, parent: Option<i32>) {
        println!("Session locked successfully");

        glib::unix_signal_add_local(
            libc::SIGUSR1,
            glib::clone!(
                #[strong]
                app,
                move || {
                    app.quit();
                    glib::ControlFlow::Break
                }
            ),
        );

        if let Some(parent) = parent {
            unsafe { libc::kill(parent, libc::SIGUSR2) };
        }

        // glib::timeout_add_local(
        //     std::time::Duration::from_secs(2),
        //     move || {
        //         unsafe { libc::raise(libc::SIGUSR1) };
        //         glib::ControlFlow::Break
        //     });
    }

    fn failed(_: &gtk_session_lock::Instance) {
        eprintln!("The session could not be locked");
    }

    fn unlocked(app: &gtk::Application) {
        println!("Session unlocked");
        app.quit();
    }

    fn on_monitor_present(
        lock: &gtk_session_lock::Instance,
        monitor: &gdk::Monitor,
        app: &Application,
        config: &config::Config,
    ) {
        // This function will be called once for each monitor (aka output) present when the session becomes locked, and also
        // whenever a new monitor is plugged in while the session is locked.

        lock.assign_window_to_monitor(&LockWindow::new(app, lock, config), monitor);
        // DONT call present, gtk_session_lock_instance_assign_window_to_monitor() does that for us
    }

    pub fn new(app: &gtk::Application, parent: Option<i32>, config: &config::Config) -> Self {
        let lock = gtk_session_lock::Instance::new();

        lock.connect_locked(glib::clone!(
            #[strong]
            app,
            move |_| Self::locked(&app, parent)
        ));
        lock.connect_failed(Self::failed);
        lock.connect_unlocked(glib::clone!(
            #[strong]
            app,
            move |_| Self::unlocked(&app)
        ));

        lock.connect_monitor(glib::clone!(
            #[strong]
            app,
            #[strong]
            config,
            move |lock, monitor| Self::on_monitor_present(lock, monitor, &app, &config)
        ));

        Self(lock)
    }

    pub fn enlock(&self) -> bool {
        self.0.lock()
    }

    pub fn unlock(&self) {
        self.0.unlock();
    }

    // pub fn available() -> bool {
    //     gtk_session_lock::is_supported()
    // }
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.unlock();
    }
}
