use std::cell::RefCell;

use gtk::{Align, Box, Label, Orientation, glib, prelude::*, subclass::prelude::*};

#[derive(Debug, Default, glib::Properties)]
#[properties(wrapper_type = super::Clock)]
pub struct Clock {
    pub time_label: RefCell<Label>,
    pub date_label: RefCell<Label>,

    /// Time format
    #[property(get, set, construct)]
    pub time_format: RefCell<String>,

    /// Date format
    #[property(get, set, construct)]
    pub date_format: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for Clock {
    const NAME: &str = "Clock";
    type Type = super::Clock;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for Clock {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(8);
        obj.set_halign(Align::Center);

        let time_label = Label::builder().css_classes(["time"]).build();
        let date_label = Label::builder().css_classes(["date"]).build();

        update_time(
            &self.time_format.borrow(),
            &self.date_format.borrow(),
            &time_label,
            &date_label,
        );

        obj.append(&time_label);
        obj.append(&date_label);

        glib::timeout_add_local(
            std::time::Duration::from_secs(1),
            glib::clone!(
                #[strong(rename_to = clock)]
                self.obj(),
                move || {
                    update_time(
                        &clock.imp().time_format.borrow(),
                        &clock.imp().date_format.borrow(),
                        &clock.imp().time_label.borrow(),
                        &clock.imp().date_label.borrow(),
                    );
                    glib::ControlFlow::Continue
                }
            ),
        );

        *self.time_label.borrow_mut() = time_label;
        *self.date_label.borrow_mut() = date_label;
    }
}

impl WidgetImpl for Clock {}
impl BoxImpl for Clock {}

fn format(now: &glib::DateTime, fmt: &str) -> glib::GString {
    now.format(fmt)
        .map_err(|err| err.message)
        .unwrap_or_else(std::convert::Into::into)
}

fn update_time(time_format: &str, date_format: &str, time: &Label, date: &Label) {
    match glib::DateTime::now_local() {
        Ok(now) => {
            time.set_text(&format(&now, time_format));
            date.set_text(&format(&now, date_format));
        }
        Err(err) => {
            time.set_text(&err.message);
            date.set_text("");
        }
    }
}
