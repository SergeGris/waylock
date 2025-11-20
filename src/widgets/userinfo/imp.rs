use accounts_service::{User, UserManager, UserManagerExt};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    Box, Label, Orientation,
    gdk::{self, Paintable, Texture},
    glib::{self},
};
use std::cell::RefCell;

use crate::attach_style;

mod imp {
    use super::*;
    use gtk::{Align, Frame, Image, Revealer, RevealerTransitionType};

    // The instance struct — holds widget children (created in constructed)
    #[derive(Default)]
    pub struct UserInfo {
        // store children so we can access them from the public API
        pub revealer: RefCell<Revealer>,
        pub user_icon: RefCell<Image>,
        pub user_label: RefCell<Label>,
    }

    // Boilerplate: declare the GObject type.
    #[glib::object_subclass]
    impl ObjectSubclass for UserInfo {
        const NAME: &'static str = "UserInfo";
        type Type = super::UserInfo;
        type ParentType = Box;
    }

    impl ObjectImpl for UserInfo {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.set_halign(Align::Center);

            let vbox = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(8)
                .vexpand(false)
                .hexpand(false)
                .valign(Align::Center)
                .halign(Align::Center)
                .build();

            let image = Image::new();
            let label = Label::new(None);
            let frame = Frame::builder().child(&image).build();
            let revealer = Revealer::builder()
                .child(&vbox)
                .reveal_child(false)
                .transition_type(RevealerTransitionType::None)
                .build();

            frame.style_context().add_class("user-image");
            label.style_context().add_class("user-name");

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
}

glib::wrapper! {
    pub struct UserInfo(ObjectSubclass<imp::UserInfo>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable, gtk::Accessible, gtk::ConstraintTarget, gtk::Orientable;
}

impl UserInfo {
    /// Create a new instance of the widget.
    pub fn new() -> Self {
        // Using Object::builder is the modern, ergonomic way
        let obj: UserInfo = glib::Object::builder().build();

        attach_style!(
            r#"
.user-image {{
    -gtk-icon-size: 96pt;
    border-radius: 100%;
}}
.user-name {{
    font-size: 18pt;
}}
"#
        );

        let man = UserManager::default();

        if let Some(man) = man {
            let username = glib::user_name().into_string().unwrap();
            let user = man.user(&username).unwrap();
            user.connect_is_loaded_notify(glib::clone!(
                #[weak]
                obj,
                move |u| obj.set_user_info(u)
            ));
            user.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |u| obj.set_user_info(u)
            ));

            // TODO
            // user.connect_language_notify(glib::clone!(
            //     #[weak] obj,
            //     move |u| {
            //         let lang = u.language().map(|s| s.to_string());
            //         let l = if let Some(l) = lang {
            //             l
            //         } else {
            //             "".into()
            //         };
            //         obj.imp().language.borrow().set_text(&l);
            //     }
            // ));
        }

        obj
    }

    fn set_user_info(&self, u: &User) {
        if !u.is_loaded() {
            return;
        }

        let username = u
            .real_name()
            .unwrap_or_else(|| u.user_name().unwrap_or("unknown".into()));
        self.set_user_name(&username);

        // let paintable: Option<Paintable> = if let Some(path) = u.icon_file() {
        //     match Texture::from_filename(path) {
        //         Ok(t) => Some(t.into()),
        //         Err(e) => {
        //             eprintln!("{e}");
        //             None
        //         }
        //     }
        // } else {
        //     None
        // };

        let paintable = u
            .icon_file()
            .and_then(|path| {
                Texture::from_filename(path)
                    .map_err(|e| eprintln!("Failed to open user {e}"))
                    .ok()
                    .map(|t| t.into())
            })
            .or_else(|| Self::paintable_from_icon_name("avatar-default-symbolic"));

        if let Some(ref p) = paintable {
            self.set_image_from_paintable(p);
        }

        // if let Some(p) = paintable.or_else(|| Self::paintable_from_icon_name("avatar-default-symbolic", 1)) {
        //     self.set_image_from_paintable(&p);
        // }

        self.imp().revealer.borrow().set_reveal_child(true);
    }

    fn paintable_from_icon_name(icon_name: &str) -> Option<gdk::Paintable> {
        use gtk::gdk;
        use gtk::{IconLookupFlags, IconTheme, TextDirection};

        let display = gdk::Display::default()?;
        let theme = IconTheme::for_display(&display);

        Some(
            theme
                .lookup_icon(
                    icon_name,
                    &[],
                    1,
                    1,
                    TextDirection::None,
                    IconLookupFlags::empty(),
                )
                .into(),
        )
        //.and_then(|info| info.load_icon()) // → Option<Paintable>
    }

    /// Set the label text.
    pub fn set_user_name(&self, text: &str) {
        self.imp().user_label.borrow().set_text(text);
    }

    /// Set the image by path (loads a file on disk).
    /// Use absolute paths or bundle images as resources.
    pub fn set_image_from_paintable(&self, texture: &impl IsA<Paintable>) {
        self.imp().user_icon.borrow().set_paintable(Some(texture))
    }
}

impl Default for UserInfo {
    fn default() -> Self {
        Self::new()
    }
}

// mod avatar {
// use gtk::prelude::*;
// use gtk::{DrawingArea, gdk};

// pub fn create_avatar(paintable: Option<gdk::Paintable>, size: i32) -> DrawingArea {
//     let area = DrawingArea::new();
//     area.set_content_width(size);
//     area.set_content_height(size);

//     area.set_draw_func(move |widget, cr, width, height| {
//         if let Some(paint) = &paintable {
//             // Clip to circle
//             let radius = (width.min(height) as f64) / 2.0;
//             cr.arc(width as f64 / 2.0, height as f64 / 2.0, radius, 0.0, std::f64::consts::TAU);
//             cr.clip();

//             // Draw the paintable scaled

//             let snapshot = gtk::Snapshot::new();
//             paint.snapshot(&snapshot, width as f64, height as f64);
//             // snapshot.render(cr);
//             // paint.snapshot();
//             // paint.snapshot(Some(&widget.snapshot_child()), width as f64, height as f64);
//         }
//     });

//     area
// }
// }
