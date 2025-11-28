use std::cell::RefCell;

use gtk::{
    Align,
    Box,
    Frame,
    Image,
    Label,
    Orientation,
    Revealer,
    RevealerTransitionType,
    glib,
    prelude::*,
    subclass::prelude::*,
};

#[derive(Default)]
pub struct UserInfo {
    pub revealer: RefCell<Revealer>,
    pub user_icon: RefCell<Image>,
    pub user_label: RefCell<Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for UserInfo {
    const NAME: &str = "UserInfo";
    type Type = super::UserInfo;
    type ParentType = Box;
}

impl ObjectImpl for UserInfo {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        obj.set_orientation(Orientation::Vertical);
        obj.set_halign(Align::Center);

        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .valign(Align::Center)
            .build();

        let image = Image::new();
        let label = Label::builder().css_classes(["user-name"]).build();
        let frame = Frame::builder()
            .css_classes(["user-image"])
            .child(&image)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();
        let revealer = Revealer::builder()
            .child(&vbox)
            .transition_type(RevealerTransitionType::Crossfade)
            .build();

        vbox.append(&frame);
        vbox.append(&label);

        obj.append(&revealer);

        *self.user_icon.borrow_mut() = image;
        *self.user_label.borrow_mut() = label;
        *self.revealer.borrow_mut() = revealer;
    }
}

impl WidgetImpl for UserInfo {}
impl BoxImpl for UserInfo {}
