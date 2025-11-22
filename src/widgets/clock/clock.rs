use gtk::{Accessible, Box, Buildable, ConstraintTarget, Orientable, Widget, glib};

use crate::clock::imp;

glib::wrapper! {
    pub struct Clock(ObjectSubclass<imp::Clock>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl Clock {
    pub fn new(time_format: &str, date_format: &str) -> Self {
        glib::Object::builder()
            .property("time-format", time_format)
            .property("date-format", date_format)
            .build()
    }
}
