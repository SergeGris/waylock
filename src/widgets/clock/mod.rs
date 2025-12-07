mod imp;

use gtk::{Accessible, Box, Buildable, ConstraintTarget, Orientable, Widget, glib};

glib::wrapper! {
    pub struct Clock(ObjectSubclass<imp::Clock>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl Clock {
    pub fn new(time_format: impl AsRef<str>, date_format: impl AsRef<str>) -> Self {
        glib::Object::builder()
            .property("time-format", time_format.as_ref())
            .property("date-format", date_format.as_ref())
            .build()
    }
}
