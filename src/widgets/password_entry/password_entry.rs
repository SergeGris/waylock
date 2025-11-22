use gtk::{Accessible, Buildable, CellEditable, ConstraintTarget, Editable, Entry, Widget, glib};

use crate::password_entry::imp;

glib::wrapper! {
    pub struct PasswordEntry(ObjectSubclass<imp::PasswordEntry>)
        @extends Widget, Entry,
        @implements Accessible, Buildable, ConstraintTarget, Editable, CellEditable;
}

impl PasswordEntry {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for PasswordEntry {
    fn default() -> Self {
        Self::new()
    }
}
