use gtk::{Application, gdk, glib, prelude::*};

use crate::{config, log, widgets::window::LockWindow};

#[derive(Clone, glib::Downgrade, Debug, Default)]
pub struct Lock(pub gtk_session_lock::Instance);

impl Lock {
    fn locked(app: &Application, parent: Option<i32>) {
        glib::unix_signal_add_local_once(
            libc::SIGUSR1,
            glib::clone!(
                #[weak]
                app,
                move || app.quit()
            ),
        );

        // If we are want to daemonize, we shall send signal (SIGUSR2)
        // to notify that we are successfully locked screen.
        if let Some(parent) = parent {
            unsafe { libc::kill(parent, libc::SIGUSR2) };
        }
    }

    fn failed(app: &gtk::Application) {
        log::fatal!("failed to lock session");
        app.quit();
    }

    fn unlocked(app: &gtk::Application) {
        app.quit();
    }

    fn on_monitor_present(
        lock: &gtk_session_lock::Instance,
        monitor: &gdk::Monitor,
        app: &Application,
        config: &config::Config,
    ) {
        // This function will be called once for each monitor (aka output)
        // present when the session becomes locked, and also
        // whenever a new monitor is plugged in while the session is locked.
        let w = LockWindow::builder()
            .application(app)
            .lock(lock)
            .start_hidden(config.get_start_hidden())
            .idle_timeout(config.get_idle_timeout())
            .time_format(config.get_time_format())
            .date_format(config.get_date_format())
            .background(config.get_background())
            .monitor(monitor)
            .build();

        lock.assign_window_to_monitor(&w, monitor);
        // DONT call present, gtk_session_lock_instance_assign_window_to_monitor() does that for us
    }

    pub fn new(app: &gtk::Application, parent: Option<i32>, config: &config::Config) -> Self {
        let lock = gtk_session_lock::Instance::new();

        lock.connect_locked(glib::clone!(
            #[weak]
            app,
            move |_| Self::locked(&app, parent)
        ));
        lock.connect_failed(glib::clone!(
            #[weak]
            app,
            move |_| Self::failed(&app)
        ));
        lock.connect_unlocked(glib::clone!(
            #[weak]
            app,
            move |_| Self::unlocked(&app)
        ));
        lock.connect_monitor(glib::clone!(
            #[weak]
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

    pub fn is_supported() -> bool {
        gtk_session_lock::is_supported()
    }
}
