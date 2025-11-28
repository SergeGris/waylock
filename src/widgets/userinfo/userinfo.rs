use accounts_service::{User, UserManager, UserManagerExt};
use gtk::{
    Accessible,
    Box,
    Buildable,
    ConstraintTarget,
    Orientable,
    Widget,
    gdk::{self, Texture},
    glib,
    prelude::*,
    subclass::prelude::*,
};

use crate::{css, log, widgets::userinfo::imp};

glib::wrapper! {
    pub struct UserInfo(ObjectSubclass<imp::UserInfo>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl UserInfo {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new();

        css::attach_style(
            "
.user-image {
    -gtk-icon-size: 96pt;
    border-radius: 100%;
    border: none;
}
.user-name {
    font-size: 16pt;
}
",
        );

        let user_manager = UserManager::default();

        if let Some(user_manager) = user_manager {
            match glib::user_name().into_string() {
                Ok(username) => match user_manager.user(&username) {
                    Some(user) => {
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
                    None => {
                        log::warning!("User not found");
                    }
                },
                Err(err) => {
                    log::warning!("{err:?}");
                }
            }
        }

        obj
    }

    fn set_user_info(&self, u: &User) {
        if !u.is_loaded() {
            return;
        }

        let username = u.real_name().or_else(|| u.user_name());

        let paintable = u
            .icon_file()
            .and_then(|path| {
                Texture::from_filename(path)
                    .map_err(|e| log::warning!("failed to open user image {e}"))
                    .ok()
                    .map(std::convert::Into::into)
            })
            .or_else(|| Self::paintable_from_icon_name("avatar-default-symbolic"));

        // TODO visibility
        if let Some(ref username) = username {
            self.imp().user_label.borrow().set_visible(true);
            self.imp().user_label.borrow().set_text(username);
        } else {
            self.imp().user_label.borrow().set_visible(false);
        }

        if let Some(ref paintable) = paintable {
            self.imp().user_icon.borrow().set_visible(true);
            self.imp().user_icon.borrow().set_paintable(Some(paintable));
        } else {
            // For if we can not show anything, make frame invisible to save space.
            self.imp().user_icon.borrow().set_visible(false);
        }

        self.imp().revealer.borrow().set_reveal_child(true);
    }

    fn paintable_from_icon_name(icon_name: &str) -> Option<gdk::Paintable> {
        use gtk::{IconLookupFlags, IconTheme, TextDirection, gdk::Display};

        Some(
            IconTheme::for_display(&Display::default()?)
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
    }
}

impl Default for UserInfo {
    fn default() -> Self {
        Self::new()
    }
}
