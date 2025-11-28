use std::cell::RefCell;

#[cfg(feature = "video")]
use gtk::MediaFile;
use gtk::{
    Align,
    ApplicationWindow,
    Box,
    Button,
    ContentFit,
    EventControllerFocus,
    EventControllerKey,
    EventControllerScroll,
    EventControllerScrollFlags,
    GestureClick,
    Image,
    Label,
    Orientation,
    Overlay,
    PasswordEntry,
    Picture,
    PropagationPhase,
    Revealer,
    Spinner,
    gdk,
    gio,
    glib::{self, ControlFlow, GString, Propagation, SourceId},
    prelude::*,
    subclass::prelude::*,
};

#[cfg(feature = "userinfo")]
use crate::userinfo;
use crate::{
    config,
    css,
    log,
    messages,
    pam,
    widgets::{clock, powerbar},
};

#[cfg(feature = "screenshot")]
fn capture_monitor_screenshot(monitor: &gdk::Monitor) -> Option<Screenshot> {
    use grim_rs::{CaptureParameters, Grim, Result};

    let mut grim = Grim::new().ok()?;
    let description = monitor.description()?;
    let description = description.as_str();
    let outputs = grim.get_outputs().ok()?;

    for output in outputs {
        if let Some(output_description) = output.description()
            && description == output_description
        {
            let result = grim.capture_output(output.name()).ok()?;
            let width = result.width();
            let height = result.height();
            let mut data = result.into_data();

            return Some(Screenshot {
                width,
                height,
                data,
            });
        }
    }

    None
}

#[cfg(feature = "screenshot")]
#[derive(Default, Debug, Clone)]
struct Screenshot {
    // iter: usize,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[derive(Debug, Default, glib::Properties)]
#[properties(wrapper_type = super::LockWindow)]
pub struct LockWindow {
    pub idle_source: RefCell<Option<SourceId>>,
    pub password_entry: RefCell<PasswordEntry>,
    pub error_label: RefCell<Label>,
    pub error_revealer: RefCell<Revealer>,
    pub body_revealer: RefCell<Revealer>,
    #[cfg(feature = "show-submit-button")]
    pub submit_button: RefCell<Button>,
    pub spinner: RefCell<Spinner>,
    pub busy_guard: RefCell<Option<gio::ApplicationBusyGuard>>,
    pub powerbar_revealer: RefCell<Revealer>,
    pub layout_names: RefCell<Vec<GString>>,
    pub active_layout_index: RefCell<i32>,
    pub active_layout_label: RefCell<Label>,

    #[cfg(feature = "userinfo")]
    pub userinfo: RefCell<userinfo::UserInfo>,

    pub feed: RefCell<messages::MessageWindow>,

    #[cfg(feature = "screenshot")]
    pub screenshot: RefCell<Screenshot>,

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
    /// Monitor
    #[property(get, set, construct_only)]
    pub monitor: RefCell<Option<gdk::Monitor>>,
}

#[glib::object_subclass]
impl ObjectSubclass for LockWindow {
    const NAME: &str = "LockWindow";
    type Type = super::LockWindow;
    type ParentType = ApplicationWindow;
}

#[glib::derived_properties]
impl ObjectImpl for LockWindow {
    fn constructed(&self) {
        self.parent_constructed();

        let window = self.obj();

        let password_entry = PasswordEntry::builder()
            .hexpand(true)
            .show_peek_icon(true)
            .placeholder_text("Password")
            .width_request(380) // TODO configure size?
            .build();

        let error_label = Label::new(None);

        #[cfg(feature = "show-submit-button")]
        let submit_button = Button::builder()
            .label("Submit")
            .css_classes(["suggested-action", "submit-button"])
            .build();

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
        let body_revealer = Revealer::builder().child(&body).build();
        let error_revealer = Revealer::builder()
            .css_classes(["error-label"])
            .child(&error_label)
            .build();

        submit_row.append(&error_revealer);
        submit_row.append(&spinner);
        #[cfg(feature = "show-submit-button")]
        submit_row.append(&submit_button);

        let top_bar = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_top(4)
            .margin_start(4)
            .margin_end(4)
            .margin_bottom(4)
            .valign(Align::Start)
            .build();

        let active_layout_label = Label::new(None);

        let layout = Box::builder()
            .orientation(Orientation::Horizontal)
            .halign(Align::Center)
            .spacing(4)
            .build();

        layout.append(&Image::from_icon_name(
            "preferences-desktop-keyboard-symbolic",
        ));
        layout.append(&active_layout_label);

        #[cfg(feature = "playerctl")]
        {
            let playerctl = crate::playerctl::PlayerControls::new();
            top_bar.append(&playerctl);
        }

        top_bar.append(&Label::builder().hexpand(true).build());
        top_bar.append(&layout);

        let caps_lock_revealer = Revealer::builder()
            .child(&Label::new(Some("Caps Lock is on")))
            .build();

        #[cfg(feature = "show-numlock")]
        let num_lock_revealer = Revealer::builder()
            .child(&Label::new(Some("Num Lock is on")))
            .build();

        if let Some(display) = gdk::Display::default()
            && let Some(seat) = display.default_seat()
            && let Some(keyboard) = seat.keyboard()
        {
            keyboard.connect_caps_lock_state_notify(glib::clone!(
                #[weak]
                caps_lock_revealer,
                move |keyboard| caps_lock_revealer.set_reveal_child(keyboard.is_caps_locked()),
            ));

            #[cfg(feature = "show-numlock")]
            keyboard.connect_num_lock_state_notify(glib::clone!(
                #[weak]
                num_lock_revealer,
                move |keyboard| num_lock_revealer.set_reveal_child(keyboard.is_num_locked()),
            ));

            keyboard.connect_layout_names_notify(glib::clone!(
                #[weak]
                window,
                #[weak]
                active_layout_label,
                move |keyboard| {
                    let names = keyboard.layout_names();
                    active_layout_label.set_width_request(Self::compute_max_label_size(
                        &active_layout_label,
                        &names,
                    ));
                    *window.imp().layout_names.borrow_mut() = names;
                    Self::handle_layout_change(&window);
                }
            ));

            keyboard.connect_active_layout_index_notify(glib::clone!(
                #[weak]
                window,
                move |keyboard| {
                    let active_layout = keyboard.active_layout_index();
                    *window.imp().active_layout_index.borrow_mut() = active_layout;
                    Self::handle_layout_change(&window);
                }
            ));

            let names = keyboard.layout_names();
            let index = keyboard.active_layout_index();

            active_layout_label.set_xalign(0.0);
            active_layout_label
                .set_width_request(Self::compute_max_label_size(&active_layout_label, &names));

            // For this properties we shall call them manually
            if let Some(name) = names.get(index.cast_unsigned() as usize) {
                active_layout_label.set_text(name);
            }

            *window.imp().layout_names.borrow_mut() = names;
            *window.imp().active_layout_index.borrow_mut() = index;
        }

        body.append(&password_entry);
        body.append(&submit_row);
        body.append(&caps_lock_revealer);
        #[cfg(feature = "show-numlock")]
        body.append(&num_lock_revealer);

        #[cfg(feature = "userinfo")]
        {
            let userinfo = userinfo::UserInfo::new();
            main_box.append(&userinfo);
            *self.userinfo.borrow_mut() = userinfo;
        }

        main_box.append(&clock::Clock::new(
            window.time_format(),
            window.date_format(),
        ));

        main_box.append(&body_revealer);

        let msg = messages::MessageWindow::new();
        main_box.append(&msg);

        let powerbar_revealer = Revealer::builder()
            .child(&powerbar::PowerBar::new())
            .halign(Align::Center)
            .valign(Align::End)
            .margin_bottom(8)
            .build();

        ////
        // // Try to enumerate monitors via xcap
        //     let monitors = match xcap::Monitor::all() {
        //         Ok(m) => m,
        //         Err(err) => {
        //             eprintln!("Failed to query monitors via xcap: {:?}", err);
        //             return;
        //         }
        //     };

        // // Try to pick a monitor — e.g. first one
        // let monitor = match monitors.into_iter().next() {
        //     Some(m) => m,
        //     None => {
        //         eprintln!("No monitor found");
        //         return;
        //     }
        // };

        // // Capture image of that monitor
        // let image = match monitor.capture_image() {
        //     Ok(img) => img,
        //     Err(err) => {
        //         eprintln!("Failed to capture image: {:?}", err);
        //         return;
        //     }
        // };

        // let width = image.width() as i32;
        // let height = image.height() as i32;

        // println!("Captured monitor: {}x{}", width, height);

        // // xcap's image format is RGBA8888 (premultiplied or not?) — passes raw pixel buffer
        // // Convert its raw buffer into gdk::Texture (which is a Paintable)
        // let bytes = image.into_raw();  // returns Vec<u8> in RGBA order
        // let tex = gdk::Texture::from_bytes(
        //     &glib::Bytes::from_owned(bytes),
        // );
        //     ////

        let background = Picture::builder()
            .css_name("background")
            .content_fit(ContentFit::Cover)
            .build();

        #[cfg(feature = "screenshot")]
        let screenshot_blur = Picture::builder().content_fit(ContentFit::Cover).build();

        let background_is_screenshot = self.handle_background(&background);

        let overlay = Overlay::new();
        overlay.set_child(Some(&main_box));
        overlay.add_overlay(&powerbar_revealer);
        overlay.add_overlay(&top_bar);

        let background_overlay = Overlay::new();
        background_overlay.add_overlay(&background);
        #[cfg(feature = "screenshot")]
        background_overlay.add_overlay(&screenshot_blur);
        background_overlay.add_overlay(&overlay);

        window.set_child(Some(&background_overlay));

        *self.body_revealer.borrow_mut() = body_revealer;
        *self.error_revealer.borrow_mut() = error_revealer;
        *self.powerbar_revealer.borrow_mut() = powerbar_revealer;
        *self.error_label.borrow_mut() = error_label;
        *self.password_entry.borrow_mut() = password_entry;
        #[cfg(feature = "show-submit-button")]
        {
            *self.submit_button.borrow_mut() = submit_button;
        }
        *self.spinner.borrow_mut() = spinner;
        *self.active_layout_label.borrow_mut() = active_layout_label;
        *self.feed.borrow_mut() = msg;

        self.setup_controllers(&window);

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

        #[cfg(feature = "screenshot")]
        if background_is_screenshot {
            use libblur::*;

            let mut sh = window.imp().screenshot.borrow_mut();

            let width = sh.width;
            let height = sh.height;
            let rowstride = (width * 4) as usize; // rowstride: bytes per row (width * 4 bytes for RGBA)
            // TODO...
            let mut data: &'static mut _ = unsafe { std::mem::transmute(sh.data.as_mut_slice()) };

            let mut blur = BlurImageMut::borrow(data, width, height, FastBlurChannels::Channels4);

            //stack_blur(&mut blur, AnisotropicRadius::new(32), ThreadingPolicy::Adaptive);
            //libblur::fast_gaussian(&mut blur, AnisotropicRadius::new(8), ThreadingPolicy::Adaptive, EdgeMode2D::new(EdgeMode::Clamp));

            for i in 0..3 {
                libblur::fast_gaussian_next(
                    &mut blur,
                    AnisotropicRadius::new(24),
                    ThreadingPolicy::Adaptive,
                    EdgeMode2D::new(EdgeMode::Reflect),
                );
            }

            screenshot_blur.set_paintable(Some(&gdk::MemoryTexture::new(
                width as i32,
                height as i32,
                gdk::MemoryFormat::R8g8b8a8,
                &glib::Bytes::from_owned(data),
                (width * 4) as usize, // rowstride: bytes per row (width * 4 bytes for RGBA)
            )));
        }

        // TODO overlay.set_reveal_child(true);???
        overlay.set_opacity(0.0);
        screenshot_blur.set_opacity(0.0);

        let fade_speed = 0.02;
        let mut o = 0.0;
        glib::timeout_add_local(
            std::time::Duration::from_millis(16),
            glib::clone!(
                #[weak]
                overlay,
                #[weak]
                screenshot_blur,
                #[upgrade_or]
                ControlFlow::Continue,
                move || {
                    o += fade_speed;

                    if o >= 1.0 {
                        overlay.set_opacity(1.0);
                        screenshot_blur.set_opacity(1.0);
                        background_overlay.remove_overlay(&background);
                        ControlFlow::Break
                    } else {
                        overlay.set_opacity(o);
                        screenshot_blur.set_opacity(o);
                        ControlFlow::Continue
                    }
                }
            ),
        );
    }
}

impl WidgetImpl for LockWindow {}
impl WindowImpl for LockWindow {}
impl ApplicationWindowImpl for LockWindow {}

enum ConversationMessage {
    LoginResult(Result<(), nonstick::ErrorCode>),
    InfoMessage(std::ffi::OsString),
    ErrorMessage(std::ffi::OsString),
}

impl LockWindow {
    fn handle_layout_change(w: &super::LockWindow) {
        let names = w.imp().layout_names.borrow();
        let active_layout = w.imp().active_layout_index.borrow();
        let active_layout_label = w.imp().active_layout_label.borrow();
        if let Some(name) = names.get(active_layout.cast_unsigned() as usize) {
            active_layout_label.set_text(name);
        }
    }

    fn compute_max_label_size(label: &Label, names: &[GString]) -> i32 {
        let initial = label.text();

        let size = names
            .iter()
            .map(|s| {
                label.set_text(s);
                label.preferred_size().1.width()
            })
            .fold(0, std::cmp::max);

        label.set_text(initial.as_str());

        size
    }

    fn grab_focus_without_selecting(entry: &PasswordEntry) {
        entry.grab_focus();

        // Move cursor to end without selecting
        let len = entry.text().len().clamp(0, i32::MAX as usize) as i32;
        entry.set_position(len);
        entry.select_region(len, len);
    }

    fn key_pressed(window: &Self, key: gdk::Key) -> Propagation {
        use gdk::Key;

        if !window.password_entry.borrow().is_sensitive() {
            return Propagation::Proceed;
        }

        window.idle_show();

        println!("{key}");
        // TODO pretty silly solution
        if matches!(
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
                | Key::Up
                | Key::Left
                | Key::Right
                | Key::Down
                | Key::KP_Up
                | Key::KP_Left
                | Key::KP_Right
                | Key::KP_Down
        ) {
            return Propagation::Proceed;
        }

        let entry = window.password_entry.borrow();

        //TODO entry.grab_focus_without_selecting();
        Self::grab_focus_without_selecting(&entry);

        if matches!(key, Key::Delete | Key::KP_Delete | Key::BackSpace) {
            return Propagation::Proceed;
        }

        //println!("{key} {} {}", window.body_revealer().is_child_revealed(), window.body_revealer().reveals_child());
        if !window.body_revealer.borrow().is_child_revealed()
            || window.body_revealer.borrow().reveals_child()
        {
            // TODO handle other keys?

            // if !matches!(key, Key::ISO_Enter | Key::KP_Enter | Key::Return) {
            let mut pos = entry.text().len().clamp(0, (i32::MAX - 4) as usize) as i32;

            if let Some(c) = key.to_unicode() {
                entry.insert_text(&c.to_string(), &mut pos);
            }

            entry.set_position(pos);
            return Propagation::Stop;
        }

        Propagation::Proceed
    }

    fn authenticate(&self) {
        self.set_busy(true);
        let pwd = self.get_password();
        let (tx, rx) = std::sync::mpsc::channel::<ConversationMessage>();

        gio::spawn_blocking(move || {
            // This runs in a thread pool, not blocking the main thread
            let result = pam::authenticate(
                |text: &std::ffi::OsStr| {
                    log::warning!(
                        "{:?}",
                        tx.send(ConversationMessage::InfoMessage(text.into()))
                    );
                },
                |text: &std::ffi::OsStr| {
                    let _ = tx.send(ConversationMessage::ErrorMessage(text.into()));
                },
                String::from_utf8_lossy(glib::user_name().as_encoded_bytes()).to_string(),
                pwd,
            );

            // Send result back to main thread
            if let Err(err) = tx.send(ConversationMessage::LoginResult(result)) {
                log::warning!("{err}");
            }
        });

        // Set up idle handler to check for messages from the channel
        glib::idle_add_local(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[upgrade_or]
            ControlFlow::Break,
            move || {
                // Non-blocking check for messages
                while let Ok(message) = rx.try_recv() {
                    match message {
                        ConversationMessage::LoginResult(result) => {
                            match result {
                                Ok(()) => window.lock.borrow().unlock(),
                                Err(e) => window.set_error(e),
                            }
                            window.set_busy(false);
                            return ControlFlow::Break;
                        }

                        // TODO
                        ConversationMessage::InfoMessage(message) => {
                            let msg = "info: ".to_string()
                                + &message.into_string().ok().unwrap_or("".into());
                            log::info!("{msg}");
                            window.feed.borrow().add_message(&msg);
                        }
                        ConversationMessage::ErrorMessage(message) => {
                            let msg = "error: ".to_string()
                                + &message.into_string().ok().unwrap_or("".into());
                            log::info!("{msg:?}");
                            window.feed.borrow().add_message(&msg);
                        }
                    }
                }

                ControlFlow::Continue
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

        #[cfg(feature = "show-submit-button")]
        self.submit_button.borrow().connect_clicked(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| callback(&window)
        ));
    }

    fn set_busy(&self, busy: bool) {
        if busy {
            self.clear_error();
        }

        let window = self.obj();

        *self.busy_guard.borrow_mut() = if busy && let Some(application) = window.application() {
            Some(application.mark_busy())
        } else {
            None
        };

        if let Some(cursor) = gdk::Cursor::from_name(if busy { "wait" } else { "default" }, None) {
            window.set_cursor(Some(&cursor));
        }

        let spinner = self.spinner.borrow();
        spinner.set_spinning(busy);
        self.password_entry.borrow().set_sensitive(!busy);
        #[cfg(feature = "show-submit-button")]
        self.submit_button.borrow().set_sensitive(!busy);
    }

    fn add_idle_handler(&self) {
        let mut source = self.idle_source.borrow_mut();

        if let Some(s) = source.take() {
            SourceId::remove(s);
        }

        if self.obj().idle_timeout() == 0 {
            return;
        }

        *source = Some(glib::timeout_add_seconds_local_once(
            self.obj().idle_timeout() as u32,
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                move || {
                    window.idle_hide();
                    if let Some(s) = window.idle_source.borrow_mut().take() {
                        SourceId::remove(s);
                    }
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
        //self.password_entry.borrow().grab_focus_without_selecting();
        Self::grab_focus_without_selecting(&self.password_entry.borrow());
    }

    fn idle_hide(&self) {
        let window = self.obj();

        if let Some(app) = window.application()
            && app.is_busy()
        {
            return;
        }

        window.add_css_class("hidden");
        if let Some(cursor) = gdk::Cursor::from_name("none", None) {
            window.set_cursor(Some(&cursor));
        }
        self.body_revealer.borrow().set_reveal_child(false);
        self.powerbar_revealer.borrow().set_reveal_child(false);
    }

    fn setup_controllers(&self, window: &super::LockWindow) {
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[upgrade_or]
            Propagation::Proceed,
            move |_, key, _, _| Self::key_pressed(&window, key)
        ));
        window.add_controller(key_controller);

        let click = GestureClick::new();
        click.set_propagation_phase(PropagationPhase::Capture);
        click.connect_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _, _, _| window.idle_show()
        ));
        window.add_controller(click);

        // window.connect_motion();

        // let motion_controller = EventControllerMotion::new();
        // motion_controller.connect_motion(glib::clone!(
        //     #[weak]
        //     window,
        //     move |_, x, y| {
        //         static mut PREV_X: f64 = 0.0;
        //         static mut PREV_Y: f64 = 0.0;
        //         unsafe {
        //             if PREV_X == x && PREV_Y == y {
        //                 return;
        //             }
        //             PREV_X = x;
        //             PREV_Y = y;
        //         }
        //         println!("{x} {y}");
        //         window.imp().idle_show();
        //     }
        // ));
        // window.add_controller(motion_controller);

        let scroll = EventControllerScroll::new(EventControllerScrollFlags::all());
        scroll.set_propagation_phase(PropagationPhase::Capture);
        scroll.connect_scroll(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[upgrade_or]
            Propagation::Proceed,
            move |_, _, _| {
                window.idle_show();
                Propagation::Proceed
            }
        ));
        window.add_controller(scroll);

        // window.connect_is_active_notify(|window| {
        //     if window.is_active() {
        //         window.add_css_class("focused");
        //         //window.imp().idle_show();
        //     } else {
        //         window.remove_css_class("focused");
        //         //window.imp().idle_hide();
        //     }
        // });

        let focus = EventControllerFocus::new();
        focus.set_propagation_phase(PropagationPhase::Capture);
        focus.connect_enter(glib::clone!(
            #[weak]
            window,
            move |_| window.add_css_class("focused"),
        ));

        focus.connect_leave(glib::clone!(
            #[weak]
            window,
            move |_| window.remove_css_class("focused"),
        ));
        window.add_controller(focus);
    }

    fn handle_background(&self, background: &Picture) -> bool {
        if let Some(path) = self.background.borrow().as_ref() {
            if path == "screenshot" {
                #[cfg(feature = "screenshot")]
                if let Some(monitor) = self.monitor.borrow().as_ref() {
                    if let Some(sh) = capture_monitor_screenshot(monitor) {
                        let width = sh.width;
                        let height = sh.height;

                        background.set_paintable(Some(&gdk::MemoryTexture::new(
                            width as i32,
                            height as i32,
                            gdk::MemoryFormat::R8g8b8a8,
                            &glib::Bytes::from_owned(sh.data.clone()),
                            (width * 4) as usize, // rowstride: bytes per row (width * 4 bytes for RGBA)
                        )));

                        *self.screenshot.borrow_mut() = sh;
                        return true;
                    }

                    return false;
                }

                #[cfg(not(feature = "screenshot"))]
                log::warning!("screenshot background is not supported");

                return false;
            }

            #[cfg(feature = "video")]
            infer::get_from_path(path)
                .map_err(|err| log::warning!("failed to load background {path:?}: {err}"))
                .ok()
                .flatten()
                .map(|kind| match kind.matcher_type() {
                    infer::MatcherType::Video => {
                        let video = MediaFile::for_filename(path);

                        video.set_loop(true);
                        video.set_muted(true);
                        video.play();

                        background.set_paintable(Some(&video));
                    }
                    infer::MatcherType::Image => background.set_filename(Some(&path)),
                    _ => log::warning!("unsupported type {:?}", kind.matcher_type()),
                });

            #[cfg(not(feature = "video"))]
            background.set_filename(Some(&path));
        }

        return false;
    }
}
