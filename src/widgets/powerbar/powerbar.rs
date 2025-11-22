use gtk::{Accessible, Box, Buildable, ConstraintTarget, Orientable, Widget, glib};

use crate::powerbar::imp;

glib::wrapper! {
    pub struct PowerBar(ObjectSubclass<imp::PowerBar>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
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
