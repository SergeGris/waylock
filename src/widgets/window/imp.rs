use gtk::subclass::prelude::*;
use gtk::{Align, Application, Box, Button, Frame, Label, Orientation, Overlay, Revealer, Spinner};
use gtk::{gdk, glib, gio, prelude::*};

use std::cell::Ref;

use crate::{clock, config, pam, password_entry, powerbar};

#[cfg(feature = "userinfo")]
use crate::user;

// use zbus::Connection;

// fn get_current_language() -> zbus::Result<String> {
//     let conn = Connection::session()?;

//     // IBus input engine controller
//     let input = zbus::Proxy::new(
//         &conn,
//         "org.freedesktop.IBus",
//         "/org/freedesktop/IBus",
//         "org.freedesktop.IBus"
//     )?;

//     let engine_desc: String = input.call("CurrentInputEngine")?;
//     Ok(engine_desc)
// }

mod imp {
    use crate::password_entry;
    #[cfg(feature = "userinfo")]
    use crate::user;
    use gtk::glib::SourceId;
    use gtk::subclass::prelude::*;
    use gtk::{ApplicationWindow, Button, Label, Revealer, Spinner, gio, glib};
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct LockWindow {
        pub idle_source: RefCell<Option<SourceId>>,
        pub password_entry: RefCell<password_entry::PasswordEntry>,
        pub error_label: RefCell<Label>,
        pub error_revealer: RefCell<Revealer>,
        pub body_revealer: RefCell<Revealer>,
        pub submit_button: RefCell<Button>,
        pub lock: RefCell<gtk_session_lock::Instance>,
        pub spinner: RefCell<Spinner>,
        pub busy_guard: RefCell<Option<gio::ApplicationBusyGuard>>,
        pub powerbar_revealer: RefCell<Revealer>,
        #[cfg(feature = "userinfo")]
        pub userinfo: RefCell<user::UserInfo>,
        // pub info_box: RefCell<Box>,
        // pub time_label: RefCell<Label>,

        // pub lock: RefCell<SessionLockInstance>,
        // error_label: Option<Label>,
        // clock_label: Option<Label>,
        // date_label: Option<Label>,
        // language: String,
        // session_lock: SessionLockInstance,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LockWindow {
        const NAME: &'static str = "LockWindow";
        type Type = super::LockWindow;
        type ParentType = ApplicationWindow;
    }

    impl ObjectImpl for LockWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for LockWindow {}
    impl WindowImpl for LockWindow {}
    impl ApplicationWindowImpl for LockWindow {}
}

glib::wrapper! {
    pub struct LockWindow(ObjectSubclass<imp::LockWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gtk::Accessible, gtk::ConstraintTarget, gtk::Buildable, gtk::ShortcutManager, gtk::Root, gtk::Native, gtk::gio::ActionMap, gtk::gio::ActionGroup;
}

struct Layout<'a> {
    submit_button: &'a Button,
    spinner: &'a Spinner,
    error_label: &'a Label,
    password_entry: &'a password_entry::PasswordEntry,
}

impl LockWindow {
    fn do_layout(&self, layout: Layout<'_>, config: &config::Config) {
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
        error_revealer.set_child(Some(layout.error_label));

        submit_row.append(&error_revealer);
        submit_row.append(layout.spinner);
        submit_row.append(layout.submit_button);

        body.append(layout.password_entry);
        body.append(&submit_row);

        body_revealer.set_child(Some(&body));
        // TODO body_revealer.set_reveal_child(!config.get_start_hidden()); // TODO

        #[cfg(feature = "userinfo")]
        {
            let userinfo = user::UserInfo::new();
            main_box.append(&userinfo);
            *self.imp().userinfo.borrow_mut() = userinfo;
        }

        main_box.append(&clock::create_digital_clock(
            config.get_time_format(),
            config.get_date_format(),
        ));

        main_box.append(&body_revealer);

        let powerbar_revealer = Revealer::builder()
            .child(&powerbar::PowerBar::new())
            // .halign(Align::Center)
            // .valign(Align::End)
            .build();

        let overlay = Overlay::new();
        let background = Frame::new(None);
        background.style_context().add_class("background");
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

        *self.imp().body_revealer.borrow_mut() = body_revealer;
        *self.imp().error_revealer.borrow_mut() = error_revealer;
        *self.imp().powerbar_revealer.borrow_mut() = powerbar_revealer;
        self.set_child(Some(&overlay));
    }

    pub fn new(
        app: &Application,
        lock: &gtk_session_lock::Instance,
        config: &config::Config,
    ) -> Self {
        let obj: Self = glib::Object::builder().property("application", app).build();
        obj.style_context().add_class("window");

        let password_entry = password_entry::PasswordEntry::new();
        password_entry.set_hexpand(true);

        let error_label = Label::new(None);

        let button = Button::builder()
            .label("Submit")
            .css_classes(["suggested-action"])
            .build();
        error_label.style_context().add_class("error-label");
        button.style_context().add_class("submit-button");

        let spinner = Spinner::builder().spinning(false).build();

        let layout = Layout {
            submit_button: &button,
            password_entry: &password_entry,
            error_label: &error_label,
            spinner: &spinner,
        };

        obj.do_layout(layout, config);
        let window = obj;

        let idle_timeout = config.get_idle_timeout();

        let key_controller = gtk::EventControllerKey::new();
        // TODO key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        key_controller.connect_key_pressed(glib::clone!(
            #[strong]
            password_entry,
            #[strong]
            window,
            move |_, key, _, _| Self::key_pressed(&password_entry, &window, key, idle_timeout)
        ));

        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        click.connect_pressed(glib::clone!(
            #[weak]
            window,
            move |_, _, _, _| window.idle_show(idle_timeout)
        ));
        window.add_controller(click);

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::all());
        scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
        scroll.connect_scroll(glib::clone!(
            #[strong]
            window,
            move |_, _, _| {
                window.idle_show(idle_timeout);
                glib::Propagation::Proceed
            }
        ));
        window.add_controller(scroll);

        // body_revealer.connect_reveal_child_notify(glib::clone!(#[strong] window, move |_| window.imp().do_show_idle.set(false)));
        // body_revealer.connect_child_revealed_notify(glib::clone!(#[strong] window, move |_| window.imp().do_show_idle.set(true)));
        // // body_revealer.connect_child_revealed_notify();

        // let motion_controller = gtk::EventControllerMotion::new();
        // motion_controller.connect_motion(glib::clone!(
        //     #[weak]
        //     window,
        //     move |_, _, _| {
        //         return;
        //         // TODO
        //         let body_revealer = window.body_revealer();
        //         let mut t = false;
        //         #[cfg(feature = "userinfo")] {
        //             let userinfo = window.imp().userinfo.borrow();
        //             let revealer = userinfo.imp().revealer.borrow();
        //             println!("{} {}", !revealer.is_child_revealed(), revealer.reveals_child());
        //             // println!("{} {}", !body_revealer.is_child_revealed(), body_revealer.reveals_child());
        //             t = !revealer.is_child_revealed();  // || revealer.reveals_child();
        //         }
        //         if t {
        //             return;
        //         }
        //         // TODO
        //         if !body_revealer.is_child_revealed()
        //             || body_revealer.reveals_child()
        //         {
        //             window.idle_show(idle_timeout);
        //         }
        //     }
        // ));
        // window.add_controller(motion_controller);

        // let focus = gtk::EventControllerFocus::new();
        // focus.connect_enter(glib::clone!(#[strong] window, move |_| {
        //     println!("Focus entered the window/UI {window:?}");
        //     // window.idle_show();
        // }));

        // focus.connect_leave(glib::clone!(#[strong] window, move |_| {
        //     println!("Focus left the window/UI {window:?}");
        //     // window.imp().password_entry.borrow().set_text("");
        //     // window.idle_hide();
        // }));
        // window.add_controller(focus);

        if config.get_start_hidden() {
            window.idle_hide();
        }

        *window.imp().lock.borrow_mut() = lock.clone();
        *window.imp().error_label.borrow_mut() = error_label;
        *window.imp().password_entry.borrow_mut() = password_entry;
        *window.imp().submit_button.borrow_mut() = button;
        *window.imp().spinner.borrow_mut() = spinner;

        window.add_controller(key_controller);
        window.set_title(Some("Waylock"));
        window.set_decorated(false);
        window.connect_authenticate(Self::authenticate);
        window
    }

    fn key_pressed(
        password_entry: &password_entry::PasswordEntry,
        window: &Self,
        key: gtk::gdk::Key,
        idle_timeout: u64,
    ) -> glib::Propagation {
        use gtk::gdk::Key;

        window.idle_show(idle_timeout);

        // TODO pretty silly solution
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
            password_entry.grab_focus_without_selecting();

            if !matches!(key, Key::Delete | Key::KP_Delete | Key::BackSpace) {
                //println!("{key} {} {}", window.body_revealer().is_child_revealed(), window.body_revealer().reveals_child());
                if !window.body_revealer().is_child_revealed()
                    || window.body_revealer().reveals_child()
                {
                    // TODO handle other keys?

                    // if !matches!(key, Key::ISO_Enter | Key::KP_Enter | Key::Return) {
                    let mut pos = password_entry
                        .text()
                        .len()
                        .clamp(0, (i32::MAX - 4) as usize) as i32;

                    if let Some(c) = key.to_unicode() {
                        password_entry.insert_text(&c.to_string(), &mut pos);
                    }

                    password_entry.set_position(pos);
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
            // This runs in a thread pool, not blocking the main thread
            let result = pam::authenticate(
                String::from_utf8_lossy(gtk::glib::user_name().as_encoded_bytes()).to_string(),
                pwd,
            );

            // Send result back to main thread
            println!("{:?}", tx.send(result));
        });

        // Set up idle handler to check for messages from the channel
        glib::idle_add_local(glib::clone!(
            #[strong(rename_to = window)]
            self,
            move || {
                // Non-blocking check for messages
                if let Ok(message) = rx.try_recv() {
                    match message {
                        Ok(()) => window.lock().unlock(),
                        Err(e) => window.set_error(e),
                    }
                    window.set_busy(false);
                    return glib::ControlFlow::Break;
                }

                glib::ControlFlow::Continue
            }
        ));

        // glib::timeout_add_local(
        //     std::time::Duration::from_secs(10),
        //     {
        //         let window = self.clone();
        //         let lock = self.imp().lock.borrow().clone();
        //         move || {
        //             println!("unlocking");
        //         let r = pam::authenticate(whoami::username(), "serge".to_owned());

        //     match r {
        //         Ok(()) => lock.unlock(),
        //         Err(e) => window.set_error(e),
        //     }

        //         glib::ControlFlow::Break
        //     }
        //     }
        // );
    }

    fn get_password(&self) -> String {
        let i = self.password_entry();
        let p = i.text().to_string();
        i.set_text("");
        p
    }

    fn set_error(&self, error: nonstick::ErrorCode) {
        self.error_label().set_text(&error.to_string());
        self.error_revealer().set_reveal_child(true);
    }

    fn clear_error(&self) {
        self.error_revealer().set_reveal_child(false);
    }

    fn connect_authenticate<F>(&self, callback: F)
    where
        F: Fn(&Self) + 'static + Clone,
    {
        self.password_entry().connect_activate(glib::clone!(
            #[strong]
            callback,
            #[weak(rename_to = window)]
            self,
            move |_| callback(&window)
        ));

        self.submit_button().connect_clicked(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| callback(&window)
        ));
    }

    fn set_busy(&self, busy: bool) {
        self.clear_error();
        *self.imp().busy_guard.borrow_mut() = if busy {
            Some(self.application().unwrap().mark_busy())
        } else {
            None
        };

        if let Some(cursor) = gdk::Cursor::from_name(if busy { "wait" } else { "default" }, None) {
            self.set_cursor(Some(&cursor));
        }
        let spinner = self.spinner();
        spinner.set_spinning(busy);
        self.password_entry().set_sensitive(!busy);
        self.submit_button().set_sensitive(!busy);
    }

    fn lock(&self) -> Ref<'_, gtk_session_lock::Instance> {
        self.imp().lock.borrow()
    }

    fn spinner(&self) -> Ref<'_, Spinner> {
        self.imp().spinner.borrow()
    }

    fn password_entry(&self) -> Ref<'_, password_entry::PasswordEntry> {
        self.imp().password_entry.borrow()
    }

    fn submit_button(&self) -> Ref<'_, Button> {
        self.imp().submit_button.borrow()
    }

    fn error_label(&self) -> Ref<'_, Label> {
        self.imp().error_label.borrow()
    }

    fn error_revealer(&self) -> Ref<'_, Revealer> {
        self.imp().error_revealer.borrow()
    }

    fn body_revealer(&self) -> Ref<'_, Revealer> {
        self.imp().body_revealer.borrow()
    }

    fn powerbar_revealer(&self) -> Ref<'_, Revealer> {
        self.imp().powerbar_revealer.borrow()
    }

    fn add_idle_handler(&self, idle_timeout: u64) {
        let mut source = self.imp().idle_source.borrow_mut();

        if let Some(s) = source.take() {
            glib::SourceId::remove(s);
        }

        *source = Some(glib::timeout_add_local(
            std::time::Duration::from_secs(idle_timeout),
            glib::clone!(
                #[strong(rename_to = window)]
                self,
                move || {
                    window.idle_hide();
                    // let _ = window.imp().idle_source.borrow_mut().take();
                    if let Some(s) = window.imp().idle_source.borrow_mut().take() {
                        glib::SourceId::remove(s);
                    }
                    glib::ControlFlow::Break
                }
            ),
        ));
    }

    fn idle_show(&self, idle_timeout: u64) {
        self.add_idle_handler(idle_timeout);

        if self.style_context().has_class("hidden") {
            self.style_context().remove_class("hidden");
            self.body_revealer().set_reveal_child(true);
            self.powerbar_revealer().set_reveal_child(true);
            if let Some(cursor) = gdk::Cursor::from_name("default", None) {
                self.set_cursor(Some(&cursor));
            }
            self.password_entry().grab_focus_without_selecting();
        }
    }

    fn idle_hide(&self) {
        if !self.style_context().has_class("hidden") {
            self.style_context().add_class("hidden");
            if let Some(cursor) = gdk::Cursor::from_name("none", None) {
                self.set_cursor(Some(&cursor));
            }
            self.body_revealer().set_reveal_child(false);
            self.powerbar_revealer().set_reveal_child(false);
        }
    }
}
