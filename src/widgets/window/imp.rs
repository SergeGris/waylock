use std::cell::RefCell;

#[cfg(feature = "video")]
use gtk::MediaFile;
use gtk::{
    Align,
    ApplicationWindow,
    Box,
    Button,
    ContentFit,
    EventControllerKey,
    EventControllerScroll,
    EventControllerScrollFlags,
    GestureClick,
    Image,
    Label,
    Orientation,
    Overlay,
    Picture,
    PropagationPhase,
    Revealer,
    Spinner,
    gdk,
    gio,
    glib,
    prelude::*,
    subclass::prelude::*,
};

#[cfg(feature = "userinfo")]
use crate::userinfo;
use crate::{
    attach_style,
    config,
    pam,
    widgets::{clock, password_entry, powerbar},
};

#[derive(Debug, Default, glib::Properties)]
#[properties(wrapper_type = super::LockWindow)]
pub struct LockWindow {
    pub idle_source: RefCell<Option<glib::SourceId>>,
    pub password_entry: RefCell<password_entry::PasswordEntry>,
    pub error_label: RefCell<Label>,
    pub error_revealer: RefCell<Revealer>,
    pub body_revealer: RefCell<Revealer>,
    pub submit_button: RefCell<Button>,
    pub spinner: RefCell<Spinner>,
    pub busy_guard: RefCell<Option<gio::ApplicationBusyGuard>>,
    pub powerbar_revealer: RefCell<Revealer>,
    pub caps_lock_label: RefCell<Label>,
    pub num_lock_label: RefCell<Label>,
    pub layout_names: RefCell<Vec<glib::GString>>,
    pub active_layout_index: RefCell<i32>,
    pub active_layout_label: RefCell<Label>,

    #[cfg(feature = "userinfo")]
    pub userinfo: RefCell<userinfo::UserInfo>,

    /// Lock instance
    #[property(get, set, construct_only)]
    pub lock: RefCell<gtk_session_lock::Instance>,
    /// Time format
    #[property(get, set, construct, default = config::default::TIME_FORMAT)]
    pub time_format: RefCell<String>,
    /// Time format
    #[property(get, set, construct, default = config::default::DATE_FORMAT)]
    pub date_format: RefCell<String>,
    /// Whether window should appear with hidden body
    #[property(get, set, construct, default = config::default::START_HIDDEN)]
    pub start_hidden: RefCell<bool>,
    /// Idle timeout
    #[property(get, set, construct, default = config::default::IDLE_TIMEOUT)]
    pub idle_timeout: RefCell<u64>,
    /// Path to background
    #[property(get, set, construct, default = None)]
    pub background: RefCell<Option<std::path::PathBuf>>,
}

#[glib::object_subclass]
impl ObjectSubclass for LockWindow {
    const NAME: &'static str = "LockWindow";
    type Type = super::LockWindow;
    type ParentType = ApplicationWindow;
}

#[glib::derived_properties]
impl ObjectImpl for LockWindow {
    fn constructed(&self) {
        self.parent_constructed();

        let window = self.obj();
        window.set_widget_name("window");

        let password_entry = password_entry::PasswordEntry::new();
        password_entry.set_hexpand(true);

        let error_label = Label::new(None);

        let submit_button = Button::builder()
            .label("Submit")
            .css_classes(["suggested-action"])
            .build();
        error_label.set_widget_name("error-label");
        submit_button.set_widget_name("submit-button");

        let spinner = Spinner::builder().spinning(false).build();

        let submit_row = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Horizontal)
            .spacing(16)
            .halign(Align::End)
            .build();

        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .halign(Align::Center)
            .valign(Align::Center)
            .hexpand(true)
            .vexpand(true)
            .build();

        let body = Box::new(Orientation::Vertical, 8);
        let body_revealer = Revealer::new();
        let error_revealer = Revealer::new();

        error_revealer.set_child(Some(&error_label));

        submit_row.append(&error_revealer);
        submit_row.append(&spinner);
        submit_row.append(&submit_button);

        let line = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .halign(Align::End)
            .build();
        let top_bar = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_top(4)
            .margin_bottom(4)
            .halign(Align::Center)
            .build();

        let active_layout_label = Label::new(None);
        let l = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(4)
            .margin_end(4)
            .halign(Align::Center)
            .build();
        l.append(&Image::from_icon_name(
            "preferences-desktop-keyboard-symbolic",
        ));
        l.append(&active_layout_label);
        line.append(&l);
        top_bar.append(&line);
        let caps_lock_label = Label::new(None);
        let num_lock_label = Label::new(None);

        if let Some(display) = gdk::Display::default()
            && let Some(seat) = display.default_seat()
            && let Some(keyboard) = seat.keyboard()
        {
            let handle_layout_change = move |w: &super::LockWindow| {
                let names = w.imp().layout_names.borrow();
                let active_layout = w.imp().active_layout_index.borrow();
                let active_layout_label = w.imp().active_layout_label.borrow();
                if let Some(name) = names.get(active_layout.cast_unsigned() as usize) {
                    active_layout_label.set_text(name);
                }
            };

            keyboard.connect_caps_lock_state_notify(glib::clone!(
                #[weak]
                caps_lock_label,
                move |keyboard| {
                    caps_lock_label.set_text(if keyboard.is_caps_locked() {
                        "Caps Lock is on"
                    } else {
                        ""
                    });
                }
            ));

            keyboard.connect_num_lock_state_notify(glib::clone!(
                #[weak]
                num_lock_label,
                move |keyboard| {
                    num_lock_label.set_text(if keyboard.is_num_locked() {
                        "Num Lock is on"
                    } else {
                        ""
                    });
                }
            ));

            keyboard.connect_layout_names_notify(glib::clone!(
                #[weak]
                window,
                move |keyboard| {
                    let names = keyboard.layout_names();
                    *window.imp().layout_names.borrow_mut() = names;
                    handle_layout_change(&window);
                }
            ));

            keyboard.connect_active_layout_index_notify(glib::clone!(
                #[weak]
                window,
                move |keyboard| {
                    let active_layout = keyboard.active_layout_index();
                    *window.imp().active_layout_index.borrow_mut() = active_layout;
                    handle_layout_change(&window);
                }
            ));

            let names = keyboard.layout_names();
            let index = keyboard.active_layout_index();

            // For this properties we shall call them manually
            if let Some(name) = names.get(index.cast_unsigned() as usize) {
                active_layout_label.set_text(name);
            }

            *window.imp().layout_names.borrow_mut() = names;
            *window.imp().active_layout_index.borrow_mut() = index;
        }

        body.append(&password_entry);
        body.append(&submit_row);
        body.append(&caps_lock_label);
        body.append(&num_lock_label);

        body_revealer.set_child(Some(&body));

        #[cfg(feature = "userinfo")]
        {
            let userinfo = userinfo::UserInfo::new();
            main_box.append(&userinfo);
            *self.userinfo.borrow_mut() = userinfo;
        }

        main_box.append(&clock::Clock::new(
            &window.time_format(),
            &window.date_format(),
        ));

        main_box.append(&body_revealer);

        let powerbar_revealer = Revealer::builder()
            .child(&powerbar::PowerBar::new())
            .halign(Align::Center)
            .build();

        let overlay = Overlay::new();
        let background = Overlay::new();

        let picture = self.background.borrow().as_ref().map_or_else(
            || None,
            |path| {
                #[cfg(feature = "video")]
                {
                    if let Some(kind) = infer::get_from_path(path).ok().flatten() {
                        match kind.matcher_type() {
                            infer::MatcherType::Video => {
                                let video = MediaFile::for_filename(path);
                                let picture = Picture::new();

                                video.play();
                                video.set_loop(true);
                                video.set_muted(true);

                                picture.set_paintable(Some(&video));
                                picture.set_content_fit(ContentFit::Cover);

                                Some(picture)
                            }
                            infer::MatcherType::Image => {
                                let picture = Picture::for_filename(path);
                                picture.set_content_fit(ContentFit::Cover);
                                Some(picture)
                            }
                            _ => {
                                eprintln!("unsupported type {:?}", kind.matcher_type());
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                #[cfg(not(feature = "video"))]
                {
                    let picture = Picture::for_filename(path);
                    picture.set_content_fit(ContentFit::Cover);
                    Some(picture)
                }
            },
        );

        if let Some(paintable) = picture {
            background.set_child(Some(&paintable));
        } else {
            attach_style!(
                r"
#window #background {{
    background-image: linear-gradient(to bottom right, #00004B, #4B0000);
}}"
            );
        }

        background.set_widget_name("background");
        overlay.set_child(Some(&background));

        let spacer = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .build();

        let vbox = Box::new(Orientation::Vertical, 0);

        #[cfg(feature = "playerctl")]
        {
            let playerctl = crate::playerctl::PlayerControls::new();
            playerctl.set_halign(Align::Center);
            vbox.append(&playerctl);
        }

        vbox.append(&spacer);
        vbox.append(&powerbar_revealer);

        let o = Overlay::builder()
            .margin_top(8)
            .margin_start(8)
            .margin_end(8)
            .margin_bottom(8)
            .child(&vbox)
            .build();

        o.add_overlay(&main_box);

        overlay.add_overlay(&o);

        let with_bar = Overlay::builder().child(&overlay).build();
        with_bar.add_overlay(&top_bar);

        *self.body_revealer.borrow_mut() = body_revealer;
        *self.error_revealer.borrow_mut() = error_revealer;
        *self.powerbar_revealer.borrow_mut() = powerbar_revealer;
        *self.error_label.borrow_mut() = error_label;
        *self.password_entry.borrow_mut() = password_entry;
        *self.submit_button.borrow_mut() = submit_button;
        *self.spinner.borrow_mut() = spinner;
        *self.caps_lock_label.borrow_mut() = caps_lock_label;
        *self.num_lock_label.borrow_mut() = num_lock_label;
        *self.active_layout_label.borrow_mut() = active_layout_label;
        window.set_child(Some(&with_bar));

        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(glib::clone!(
            #[strong]
            window,
            move |_, key, _, _| Self::key_pressed(window.imp(), key)
        ));
        window.add_controller(key_controller);

        let click = GestureClick::new();
        click.set_propagation_phase(PropagationPhase::Capture);
        click.connect_pressed(glib::clone!(
            #[weak]
            window,
            move |_, _, _, _| window.imp().idle_show()
        ));
        window.add_controller(click);

        let scroll = EventControllerScroll::new(EventControllerScrollFlags::all());
        scroll.set_propagation_phase(PropagationPhase::Capture);
        scroll.connect_scroll(glib::clone!(
            #[strong(rename_to = window)]
            self.obj(),
            move |_, _, _| {
                window.imp().idle_show();
                glib::Propagation::Proceed
            }
        ));
        window.add_controller(scroll);

        let focus = gtk::EventControllerFocus::new();
        focus.set_propagation_phase(gtk::PropagationPhase::Capture);
        focus.connect_enter(glib::clone!(
            #[weak]
            window,
            move |_| {
                window.add_css_class("focused");
            }));

        focus.connect_leave(glib::clone!(
            #[weak]
            window,
            move |_| {
                window.remove_css_class("focused");
            }));
        window.add_controller(focus);

        window.connect_start_hidden_notify(|w| {
            if w.start_hidden() {
                w.imp().idle_hide()
            } else {
                w.imp().idle_show();
            }
        });
        window.connect_idle_timeout_notify(|w| w.imp().add_idle_handler());

        window.set_title(Some("Waylock"));
        window.set_decorated(false);
        self.connect_authenticate(Self::authenticate);
    }
}

impl WidgetImpl for LockWindow {}
impl WindowImpl for LockWindow {}
impl ApplicationWindowImpl for LockWindow {}

impl LockWindow {
    fn key_pressed(window: &Self, key: gdk::Key) -> glib::Propagation {
        use gdk::Key;

        window.idle_show();

        // TODO
        // Pretty silly solution
        if !matches!(
            key,
            Key::ISO_Left_Tab
                | Key::Tab
                | Key::KP_Tab
                | Key::Shift_L
                | Key::Shift_R
                | Key::Escape
                | Key::ISO_Enter
                | Key::KP_Enter
                | Key::Return
        ) {
            let entry = window.password_entry.borrow();

            entry.grab_focus_without_selecting();

            if !matches!(key, Key::Delete | Key::KP_Delete | Key::BackSpace) {
                if !window.body_revealer.borrow().is_child_revealed()
                    || window.body_revealer.borrow().reveals_child()
                {
                    // TODO
                    // Handle other keys?

                    let mut pos = entry.text().len().clamp(0, (i32::MAX - 4) as usize) as i32;

                    if let Some(c) = key.to_unicode() {
                        entry.insert_text(&c.to_string(), &mut pos);
                    }

                    entry.set_position(pos);
                    return glib::Propagation::Stop;
                }
            }
        }

        glib::Propagation::Proceed
    }

    fn authenticate(&self) {
        self.set_busy(true);
        let pwd = self.get_password();
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), nonstick::ErrorCode>>();

        gio::spawn_blocking(move || {
            // This runs in a thread pool and do not block the main thread
            let result = pam::authenticate(
                String::from_utf8_lossy(glib::user_name().as_encoded_bytes()).to_string(),
                pwd,
            );

            // Send result back to main thread
            let _ = tx.send(result);
        });

        // Set up idle handler to check for messages from the channel
        glib::idle_add_local(glib::clone!(
            #[strong(rename_to = window)]
            self.obj(),
            move || {
                // Non-blocking check for messages
                if let Ok(message) = rx.try_recv() {
                    match message {
                        Ok(()) => window.imp().lock.borrow().unlock(),
                        Err(e) => window.imp().set_error(e),
                    }
                    window.imp().set_busy(false);
                    return glib::ControlFlow::Break;
                }

                glib::ControlFlow::Continue
            }
        ));
    }

    fn get_password(&self) -> String {
        let i = self.password_entry.borrow();
        let p = i.text().to_string();
        i.set_text("");
        p
    }

    fn set_error(&self, error: nonstick::ErrorCode) {
        self.error_label.borrow().set_text(&error.to_string());
        self.error_revealer.borrow().set_reveal_child(true);
    }

    fn clear_error(&self) {
        self.error_revealer.borrow().set_reveal_child(false);
    }

    fn connect_authenticate<F>(&self, callback: F)
    where
        F: Fn(&Self) + 'static + Clone,
    {
        self.password_entry.borrow().connect_activate(glib::clone!(
            #[strong]
            callback,
            #[weak(rename_to = window)]
            self,
            move |_| callback(&window)
        ));

        self.submit_button.borrow().connect_clicked(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| callback(&window)
        ));
    }

    fn set_busy(&self, busy: bool) {
        self.clear_error();
        let window = self.obj();
        *self.busy_guard.borrow_mut() = if busy {
            Some(window.application().unwrap().mark_busy())
        } else {
            None
        };

        if let Some(cursor) = gdk::Cursor::from_name(if busy { "wait" } else { "default" }, None) {
            window.set_cursor(Some(&cursor));
        }
        let spinner = self.spinner.borrow();
        spinner.set_spinning(busy);
        self.password_entry.borrow().set_sensitive(!busy);
        self.submit_button.borrow().set_sensitive(!busy);
    }

    fn add_idle_handler(&self) {
        let mut source = self.idle_source.borrow_mut();

        if let Some(s) = source.take() {
            glib::SourceId::remove(s);
        }

        *source = Some(glib::timeout_add_local(
            std::time::Duration::from_secs(self.obj().idle_timeout()),
            glib::clone!(
                #[strong(rename_to = window)]
                self.obj(),
                move || {
                    window.imp().idle_hide();
                    // let _ = window.imp().idle_source.borrow_mut().take();
                    if let Some(s) = window.imp().idle_source.borrow_mut().take() {
                        glib::SourceId::remove(s);
                    }
                    glib::ControlFlow::Break
                }
            ),
        ));
    }

    fn idle_show(&self) {
        let window = self.obj();

        self.add_idle_handler();

        window.remove_css_class("hidden");
        self.body_revealer.borrow().set_reveal_child(true);
        self.powerbar_revealer.borrow().set_reveal_child(true);
        if let Some(cursor) = gdk::Cursor::from_name("default", None) {
            window.set_cursor(Some(&cursor));
        }
        self.password_entry.borrow().grab_focus_without_selecting();
    }

    fn idle_hide(&self) {
        let window = self.obj();

        window.add_css_class("hidden");
        if let Some(cursor) = gdk::Cursor::from_name("none", None) {
            window.set_cursor(Some(&cursor));
        }
        self.body_revealer.borrow().set_reveal_child(false);
        self.powerbar_revealer.borrow().set_reveal_child(false);
    }
}
