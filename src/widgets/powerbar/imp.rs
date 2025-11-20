use std::ffi::OsString;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{Box, Button, Orientation, glib};

enum PowerOption {
    Shutdown,
    Reboot,
    Suspend,
    UserSwitch,
    Logout,
}

const BUTTON_TIMEOUT: Duration = Duration::from_secs(5);

impl PowerOption {
    const fn get_icon(&self) -> &'static str {
        match self {
            Self::Shutdown => "system-shutdown-symbolic",
            Self::Reboot => "system-reboot-symbolic",
            Self::Suspend => "weather-clear-night-symbolic",
            Self::UserSwitch => "system-switch-user-symbolic",
            Self::Logout => "system-log-out-symbolic",
        }
    }

    fn get_default_command(&self) -> Option<OsString> {
        return Some("ls".into());
        match self {
            Self::Shutdown => Some("systemctl -i poweroff".into()),
            Self::Reboot => Some("systemctl reboot".into()),
            Self::Suspend => Some("systemctl suspend".into()),
            Self::UserSwitch => None,
            Self::Logout => None,
        }
    }
}

fn button_clicked(button: &Button, _p: &PowerOption, cmdline: &OsString) {
    if BUTTON_TIMEOUT > Duration::ZERO {
        button.set_sensitive(false);

        glib::timeout_add_local(BUTTON_TIMEOUT, {
            let btn = button.clone();
            move || {
                btn.set_sensitive(true);
                glib::ControlFlow::Break
            }
        });
    }

    glib::spawn_command_line_async(cmdline).unwrap();
}

mod imp {
    use super::{
        Box, BoxExt, Button, ButtonExt, OrientableExt, Orientation, PowerOption, button_clicked,
        glib,
    };
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct PowerBar;

    #[glib::object_subclass]
    impl ObjectSubclass for PowerBar {
        const NAME: &'static str = "PowerBar";
        type Type = super::PowerBar;
        type ParentType = Box;
    }

    impl ObjectImpl for PowerBar {
        fn constructed(&self) {
            let obj = self.obj();

            self.parent_constructed();
            obj.set_orientation(Orientation::Horizontal);
            obj.set_spacing(8);

            for n in [
                PowerOption::Shutdown,
                PowerOption::Reboot,
                PowerOption::Suspend,
                PowerOption::UserSwitch,
                PowerOption::Logout,
            ] {
                if let Some(cmdline) = n.get_default_command() {
                    let button = Button::new();
                    button.set_icon_name(n.get_icon());
                    button.connect_clicked(glib::clone!(
                        #[weak]
                        button,
                        move |_| button_clicked(&button, &n, &cmdline)
                    ));
                    obj.append(&button);
                }
            }
        }
    }

    impl WidgetImpl for PowerBar {}
    impl BoxImpl for PowerBar {}
}

glib::wrapper! {
    pub struct PowerBar(ObjectSubclass<imp::PowerBar>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable, gtk::Orientable, gtk::ConstraintTarget, gtk::Accessible;
}

impl PowerBar {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for PowerBar {
    fn default() -> Self {
        Self::new()
    }
}
