mod imp;

use gtk::{
    Accessible,
    Application,
    ApplicationWindow,
    Buildable,
    ConstraintTarget,
    Native,
    Root,
    ShortcutManager,
    Widget,
    Window,
    gdk,
    gio,
    glib::{self, object::IsA},
};

glib::wrapper! {
    pub struct LockWindow(ObjectSubclass<imp::LockWindow>)
        @extends Widget, Window, ApplicationWindow,
        @implements Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager, gio::ActionGroup, gio::ActionMap;
}

impl LockWindow {
    pub fn new(application: &Application, lock: &gtk_session_lock::Instance) -> Self {
        LockWindowBuilder::new()
            .application(application)
            .lock(lock)
            .build()
    }

    pub fn builder() -> LockWindowBuilder {
        LockWindowBuilder::new()
    }
}

#[must_use = "The builder must be built to be used"]
pub struct LockWindowBuilder {
    builder: glib::object::ObjectBuilder<'static, LockWindow>,
}

impl LockWindowBuilder {
    fn new() -> Self {
        Self {
            builder: glib::object::Object::builder(),
        }
    }

    pub fn build(self) -> LockWindow {
        self.builder.build()
    }

    pub fn application(self, application: &impl IsA<Application>) -> Self {
        Self {
            builder: self.builder.property("application", application),
        }
    }

    pub fn lock(self, lock: &impl IsA<gtk_session_lock::Instance>) -> Self {
        Self {
            builder: self.builder.property("lock", lock),
        }
    }

    pub fn time_format(self, format: &str) -> Self {
        Self {
            builder: self.builder.property("time-format", format),
        }
    }

    pub fn date_format(self, format: &str) -> Self {
        Self {
            builder: self.builder.property("date-format", format),
        }
    }

    pub fn idle_timeout(self, timeout: u64) -> Self {
        Self {
            builder: self.builder.property("idle-timeout", timeout),
        }
    }

    pub fn start_hidden(self, start_hidden: bool) -> Self {
        Self {
            builder: self.builder.property("start-hidden", start_hidden),
        }
    }

    pub fn background(self, background: Option<impl AsRef<std::path::Path>>) -> Self {
        if let Some(background) = background {
            Self {
                builder: self
                    .builder
                    .property("background", background.as_ref().to_str()),
            }
        } else {
            self
        }
    }

    pub fn monitor(self, monitor: &gdk::Monitor) -> Self {
        Self {
            builder: self.builder.property("monitor", monitor),
        }
    }
}
