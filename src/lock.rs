use gtk::{Application, gdk, glib, prelude::*};

use crate::{config, widgets::window::LockWindow};

#[derive(Clone, glib::Downgrade, Debug, Default)]
pub struct Lock(gtk_session_lock::Instance);

impl Lock {
    fn locked(app: &Application, parent: Option<i32>) {
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

        // If we are want to daemonize, we shall send signal (SIGUSR2)
        // to notify that we are successfully locked screen.
        if let Some(parent) = parent {
            unsafe { libc::kill(parent, libc::SIGUSR2) };
        }
    }

    fn failed(_: &gtk_session_lock::Instance) { }

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
            .date_format(config.get_date_format());

        let w = if let Some(bg) = config.get_background() {
            w.background(bg).build()
        } else {
            w.build()
        };

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
        lock.connect_failed(Self::failed);
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
}
