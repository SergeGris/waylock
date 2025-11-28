use gtk::{Accessible, Box, Buildable, ConstraintTarget, Orientable, Widget, glib};

use crate::widgets::playerctl::imp;

glib::wrapper! {
    pub struct PlayerControls(ObjectSubclass<imp::PlayerControls>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl PlayerControls {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for PlayerControls {
    fn default() -> Self {
        Self::new()
    }
}
