use std::rc::Rc;

use gtk::{Box, Button, Label, Orientation, Picture, glib, prelude::*, subclass::prelude::*};
use mpris::{PlaybackStatus, Player, PlayerFinder};

use crate::log;

enum PlayerButton {
    Prev,
    Play,
    Pause,
    Next,
}

impl PlayerButton {
    const fn icon_name(&self) -> &str {
        match self {
            Self::Prev => "media-skip-backward",
            Self::Play => "media-playback-start",
            Self::Pause => "media-playback-pause",
            Self::Next => "media-skip-forward",
        }
    }
}

#[derive(Default)]
pub struct PlayerControls {}

#[glib::object_subclass]
impl ObjectSubclass for PlayerControls {
    const NAME: &str = "PlayerControls";
    type Type = super::PlayerControls;
    type ParentType = Box;
}

impl ObjectImpl for PlayerControls {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        obj.set_orientation(Orientation::Horizontal);
        obj.set_spacing(8);
        obj.set_vexpand(false);
        obj.add_css_class("player-controls");

        let finder = match PlayerFinder::new() {
            Ok(f) => f,
            Err(err) => {
                log::warning!("Failed to locate MPRIS player: {err}");
                return;
            }
        };

        let player = match finder.find_active() {
            Ok(p) => Rc::new(p),
            Err(err) => {
                log::warning!("Failed to locate MPRIS player: {err}");
                return;
            }
        };

        let title = Label::new(None);
        title.add_css_class("title");

        if let Ok(meta) = player.get_metadata() {
            if let Some(t) = meta.title() {
                title.set_label(t);
            }

            if let Some(art) = meta.art_url() {
                if let Ok(pic) = Picture::for_filename(&art).downcast::<Picture>() {
                    pic.set_size_request(64, 64);
                    pic.add_css_class("album-art");
                    obj.append(&pic);
                }
            }
        }

        let controls = Box::new(Orientation::Horizontal, 4);

        // Play/Pause toggle button
        let play_button = make_play_pause_button(player.clone());

        controls.append(&make_simple_button(
            PlayerButton::Prev,
            player.clone(),
            |p| {
                p.previous();
            },
        ));
        controls.append(&play_button);
        controls.append(&make_simple_button(
            PlayerButton::Next,
            player.clone(),
            |p| {
                p.next();
            },
        ));

        obj.append(&controls);
        obj.append(&title);
    }
}

impl WidgetImpl for PlayerControls {}
impl BoxImpl for PlayerControls {}

// -----------------------------------------------------------------------------
// Helper: create play/pause toggle button
// -----------------------------------------------------------------------------
fn make_play_pause_button(player: Rc<Player>) -> Button {
    let button = Button::from_icon_name("media-playback-start");
    button.add_css_class("circular");

    // Set correct initial icon
    update_play_button_icon(&button, &player);

    button.connect_clicked(glib::clone!(
        #[strong]
        player,
        #[weak]
        button,
        move |_| {
            match player.get_playback_status() {
                Ok(PlaybackStatus::Playing) => {
                    let _ = player.pause();
                }
                Ok(_) => {
                    let _ = player.play();
                }
                Err(err) => eprintln!("Failed to query playback status: {err}"),
            }

            update_play_button_icon(&button, &player);
        }
    ));

    button
}

// -----------------------------------------------------------------------------
// Helper: simple prev/next button
// -----------------------------------------------------------------------------
fn make_simple_button<F>(kind: PlayerButton, player: Rc<Player>, func: F) -> Button
where
    F: Fn(&Player) + 'static,
{
    let button = Button::from_icon_name(kind.icon_name());
    button.add_css_class("circular");

    button.connect_clicked(glib::clone!(
        #[strong]
        player,
        move |_| func(&player)
    ));

    button
}

fn update_play_button_icon(button: &Button, player: &Player) {
    println!("{:?}", player.get_playback_status());
    let icon = match player.get_playback_status() {
        Ok(PlaybackStatus::Playing) => PlayerButton::Pause.icon_name(),
        Ok(PlaybackStatus::Paused) => PlayerButton::Play.icon_name(),
        _ => PlayerButton::Play.icon_name(),
    };

    button.set_icon_name(icon);
}

// use std::{cell::RefCell, rc::Rc};

// use gtk::{Box, Button, Image, Label, Orientation, glib, prelude::*, subclass::prelude::*};
// use mpris::{DBusError, PlaybackStatus, Player, PlayerFinder};

// enum PlayerButton {
//     Prev,
//     Play,
//     Pause,
//     Next,
// }

// impl PlayerButton {
//     const fn get_icon(&self) -> &str {
//         match self {
//             Self::Prev => "media-skip-backward",
//             Self::Play => "media-playback-start",
//             Self::Pause => "media-playback-pause",
//             Self::Next => "media-skip-forward",
//         }
//     }
// }

// #[derive(Default)]
// pub struct PlayerControls {}

// #[glib::object_subclass]
// impl ObjectSubclass for PlayerControls {
//     const NAME: &str = "PlayerControls";
//     type Type = super::PlayerControls;
//     type ParentType = Box;
// }

// impl ObjectImpl for PlayerControls {
//     fn constructed(&self) {
//         self.parent_constructed();

//         let obj = self.obj();

//         obj.set_orientation(Orientation::Horizontal);
//         obj.set_vexpand(false);

//         let finder = match PlayerFinder::new() {
//             Ok(f) => f,
//             Err(err) => {
//                 eprintln!("{err}");
//                 return;
//             }
//         };
//         let player = match finder.find_active() {
//             Ok(player) => Rc::new(RefCell::new(player)),
//             Err(err) => {
//                 eprintln!("{err}");
//                 return;
//             }
//         };

//         let title = Label::new(None);

//         if let Ok(meta) = player.borrow().get_metadata() {
//             if let Some(t) = meta.title() {
//                 title.set_label(t);
//             }

//             if let Some(art_url) = meta.art_url() {
//                 obj.append(&gtk::Picture::for_filename(art_url));
//             }
//         }

//         //        player.borrow().connect_pla

//         let b = Box::new(Orientation::Horizontal, 0);
//         {
//             let button = Button::from_icon_name(PlayerButton::Pause.get_icon());

//             button.connect_clicked(glib::clone!(
//                 #[strong]
//                 player,
//                 move |_| {
//                     //let _ = btn.get_callback()(&player.borrow());
//                     if player.borrow().get_playback_status().unwrap() == PlaybackStatus::Playing {
//                         player.borrow().play();
//                     } else {
//                         player.borrow().pause();
//                     }
//                 }
//             ));

//             b.append(&button);
//         }

//         obj.append(&b);
//         obj.append(&title);
//     }
// }

// impl WidgetImpl for PlayerControls {}
// impl BoxImpl for PlayerControls {}
