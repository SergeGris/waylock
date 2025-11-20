use gtk::glib;
use gtk::prelude::*;
use std::cell::Cell;

mod imp {
    use super::{Cell, EntryExt, WidgetExt, glib};
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct PasswordEntry {
        pub visibility: Cell<bool>,
        pub key_controller: Cell<gtk::EventControllerKey>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PasswordEntry {
        const NAME: &'static str = "PasswordEntry";
        type Type = super::PasswordEntry;
        type ParentType = gtk::Entry;
    }

    impl ObjectImpl for PasswordEntry {
        fn constructed(&self) {
            const REVEAL: &str = "view-reveal-symbolic";
            const CONCEAL: &str = "view-conceal-symbolic";

            self.parent_constructed();

            let input_field = self.obj();
            input_field.set_placeholder_text(Some("Password"));
            input_field.set_visibility(false);
            self.visibility.set(false);
            input_field.set_input_purpose(gtk::InputPurpose::Password);

            // Add the "eye" icon to toggle visibility
            input_field.set_icon_from_icon_name(
                gtk::EntryIconPosition::Secondary,
                Some(REVEAL),
            );

            input_field.imp().visibility.set(false);
            input_field.connect_visibility_notify(
                move |entry| {
                    let icon_name = if entry.imp().visibility.get() { REVEAL } else { CONCEAL };

                    entry.set_icon_from_icon_name(
                        gtk::EntryIconPosition::Secondary,
                        Some(icon_name),
                    );
                }
            );

            input_field.connect_icon_press(|entry, pos| {
                if pos == gtk::EntryIconPosition::Secondary {
                    let v = entry.imp().visibility.get();
                    entry.imp().visibility.set(!v);
                    entry.set_visibility(!v);
                }
            });

            input_field.set_hexpand(true);
        }
    }

    impl WidgetImpl for PasswordEntry {}
    impl EditableImpl for PasswordEntry {}
    impl EntryImpl for PasswordEntry {}
}

glib::wrapper! {
    pub struct PasswordEntry(ObjectSubclass<imp::PasswordEntry>)
        @extends gtk::Widget, gtk::Entry, gtk::Editable,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible, gtk::CellEditable;
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
