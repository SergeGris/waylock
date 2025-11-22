use std::{cell::RefCell, rc::Rc};

use gtk::{Box, Button, Label, Orientation, glib, prelude::*};
use mpris::{DBusError, Player, PlayerFinder};

pub struct PlayerControls {
    // container: Box,
    // title: Label,
    // play_pause: Button,
    // next: Button,
    // prev: Button,
    // player: Rc<RefCell<Player>>,
}

enum PlayerButton {
    Prev,
    PlayPause,
    Next,
}

impl PlayerButton {
    const fn get_icon(&self) -> &'static str {
        match self {
            Self::Prev => "media-skip-backward",
            Self::PlayPause => "media-playback-start",
            Self::Next => "media-skip-forward",
        }
    }

    const fn get_callback(&self) -> fn(&Player) -> Result<(), DBusError> {
        match self {
            Self::Prev => Player::previous,
            Self::PlayPause => Player::play_pause,
            Self::Next => Player::next,
        }
    }
}

impl PlayerControls {
    pub fn new() -> gtk::Widget {
        let finder = PlayerFinder::new();

        if let Err(e) = finder {
            return gtk::Label::new(Some(&format!("{e}"))).into();
        }
        let p = finder.unwrap().find_active();
        if let Err(e) = p {
            return gtk::Label::new(Some(&format!("{e}"))).into();
        }
        let player = Rc::new(RefCell::new(p.unwrap()));

        let container = Box::new(Orientation::Horizontal, 8);
        container.set_vexpand(false);
        let title = Label::new(None);

        if let Ok(meta) = player.borrow().get_metadata() {
            if let Some(t) = meta.title() {
                title.set_label(t);
            }

            if let Some(art_url) = meta.art_url() {
                container.append(&gtk::Picture::for_filename(art_url));
            }
        }

        let b = Box::new(Orientation::Horizontal, 0);
        for btn in [
            PlayerButton::Prev,
            PlayerButton::PlayPause,
            PlayerButton::Next,
        ] {
            let button = Button::from_icon_name(btn.get_icon());
            button.connect_clicked(glib::clone!(
                #[strong]
                player,
                move |_| {
                    let _ = btn.get_callback()(&player.borrow());
                }
            ));
            b.append(&button);
        }

        container.append(&b);
        container.append(&title);

        container.into()
    }
}
