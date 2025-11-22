use accounts_service::{User, UserManager, UserManagerExt};
use gtk::{
    Accessible,
    Box,
    Buildable,
    ConstraintTarget,
    Orientable,
    Widget,
    gdk::{self, Paintable, Texture},
    glib,
    prelude::*,
    subclass::prelude::*,
};

use crate::{attach_style, widgets::userinfo::imp};

glib::wrapper! {
    pub struct UserInfo(ObjectSubclass<imp::UserInfo>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl UserInfo {
    /// Create a new instance of the widget.
    pub fn new() -> Self {
        // Using Object::builder is the modern, ergonomic way
        let obj: Self = glib::Object::builder().build();

        attach_style!(
            r"
user-image {{
    background: black;
    -gtk-icon-size: 96pt;
    border-radius: 100%;
}}
#user-name {{
    font-size: 16pt;
}}
"
        );

        let user_manager = UserManager::default();

        if let Some(user_manager) = user_manager {
            let username = glib::user_name().into_string().unwrap();
            let user = user_manager.user(&username).unwrap();
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
                    .map(std::convert::Into::into)
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
        use gtk::{IconLookupFlags, IconTheme, TextDirection, gdk};

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
        self.imp().user_icon.borrow().set_paintable(Some(texture));
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
