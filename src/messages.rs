use std::cell::RefCell;

use glib::Object;
use gtk::{Box, Label, Orientation, ScrolledWindow, glib, prelude::*, subclass::prelude::*};

use crate::css;

enum MessageType {
    Info,
    Error,
}

impl MessageType {
    const fn get_color(&self) -> &str {
        match self {
            MessageType::Info => "green",
            MessageType::Error => "red",
        }
    }
}

#[derive(Default)]
pub struct MessageWindowPriv {
    pub container: RefCell<Option<Box>>,
    pub scroll: RefCell<Option<ScrolledWindow>>,
}

#[glib::object_subclass]
impl ObjectSubclass for MessageWindowPriv {
    const NAME: &'static str = "MessageWindow";
    type Type = MessageWindow;
    type ParentType = gtk::Box;

    fn new() -> Self {
        Self {
            container: RefCell::new(None),
            scroll: RefCell::new(None),
        }
    }
}

impl ObjectImpl for MessageWindowPriv {
    fn constructed(&self) {
        self.parent_constructed();

        let root = self.obj();

        root.set_orientation(Orientation::Vertical);

        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&container)
            .build();

        root.append(&scroll);

        *self.scroll.borrow_mut() = Some(scroll);
        *self.container.borrow_mut() = Some(container);
    }
}

impl WidgetImpl for MessageWindowPriv {}
impl BoxImpl for MessageWindowPriv {}

glib::wrapper! {
    pub struct MessageWindow(ObjectSubclass<MessageWindowPriv>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for MessageWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageWindow {
    pub fn new() -> Self {
        Object::new()
    }

    /// Add a new message with fade-in animation & auto scrolling
    pub fn add_message(&self, text: &str) {
        let priv_ = self.imp();
        let container = priv_.container.borrow();
        let Some(container) = container.as_ref() else {
            return;
        };
        let scroll = priv_.scroll.borrow();
        let Some(scroll) = scroll.as_ref() else {
            return;
        };

        // Create label
        let frame = gtk::Frame::new(None);
        let overlay = gtk::Overlay::new();
        let close = gtk::Button::from_icon_name("window-close-symbolic");
        close.add_css_class("flat");
        close.add_css_class("circular");
        close.connect_clicked(glib::clone!(
            #[weak]
            container,
            #[weak]
            frame,
            move |_| container.remove(&frame)
        ));
        close.set_halign(gtk::Align::End);
        close.set_valign(gtk::Align::Center);

        // TODO SET COLORS
        // css::attach_style("");

        let label = Label::builder()
            .margin_bottom(4)
            .margin_top(4)
            .margin_start(4)
            .margin_end(4)
            .build();
        label.set_text(text);
        // label.set_xalign(0.0);
        label.set_wrap(true);
        label.add_css_class("message-bubble");

        overlay.set_child(Some(&label));
        overlay.add_overlay(&close);
        frame.set_child(Some(&overlay));

        frame.set_opacity(0.0);
        container.append(&frame);

        gtk::glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            let mut opacity = frame.opacity();
            if opacity < 1.0 {
                opacity += 0.08;
                frame.set_opacity(opacity);
                glib::ControlFlow::Continue
            } else {
                frame.set_opacity(1.0);
                glib::ControlFlow::Break
            }
        });

        // Auto-scroll to bottom after small delay
        let adj = scroll.vadjustment();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
            adj.set_value(adj.upper() - adj.page_size());
        });
    }
}

// use gtk::prelude::*;
// use gtk::{glib, Align, Button, Label, PolicyType, Revealer, ScrolledWindow};
// use std::cell::RefCell;
// use std::rc::Rc;

// #[derive(Clone, Debug)]
// pub struct MessageList {
//     widget: gtk::Box,
//     messages_container: gtk::Box,
//     scrolled_window: ScrolledWindow,
//     discard_all_button: Button,
//     messages: Rc<RefCell<Vec<gtk::Widget>>>,
// }

// impl MessageList {
//     pub fn new() -> Self {
//         // Main container
//         let widget = gtk::Box::new(gtk::Orientation::Vertical, 10);
//         //widget.set_margin_all(10);

//         // Scrolled window for messages
//         let scrolled_window = ScrolledWindow::new();
//         scrolled_window.set_min_content_height(200);
//         scrolled_window.set_max_content_height(300);
//         //scrolled_window.set_propagate_natural_height(true);
//         scrolled_window.set_policy(PolicyType::Automatic, PolicyType::Automatic);

//         // Container for individual messages
//         let messages_container = gtk::Box::new(gtk::Orientation::Vertical, 8);
//         scrolled_window.set_child(Some(&messages_container));

//         // Discard all button
//         let discard_all_button = Button::with_label("Discard All");
//         discard_all_button.add_css_class("destructive-action");

//         // Assemble the widget
//         widget.append(&scrolled_window);
//         widget.append(&discard_all_button);

//         let message_list = Self {
//             widget,
//             messages_container,
//             scrolled_window,
//             discard_all_button,
//             messages: Rc::new(RefCell::new(Vec::new())),
//         };

//         // Connect discard all button
//         message_list.connect_discard_all();

//         message_list
//     }

//     /// Add a new message to the list with animation
//     pub fn add_message(&self, text: &str) {
//         let message_widget = self.create_message_widget(text);
//         let revealer = self.create_message_revealer(&message_widget);

//         // Animate in
//         self.animate_message_in(&revealer);

//         self.messages_container.append(&revealer);
//         self.messages.borrow_mut().push(revealer.upcast());

//         // Auto-scroll to bottom
//         self.scroll_to_bottom();
//     }

//     /// Create a single message widget with close button
//     fn create_message_widget(&self, text: &str) -> gtk::Box {
//         let message_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
//         message_box.add_css_class("message");
//         //message_box.set_margin_all(8);

//         // Message label
//         let label = Label::new(Some(text));
//         label.set_hexpand(true);
//         label.set_halign(Align::Start);
//         label.set_wrap(true);
//         label.set_max_width_chars(50);

//         // Close button
//         let close_button = Button::from_icon_name("window-close-symbolic");
//         close_button.add_css_class("flat");
//         close_button.add_css_class("circular");

//         message_box.append(&label);
//         message_box.append(&close_button);

//         // Connect close button
//         let messages = self.messages.clone();
//         let messages_container = self.messages_container.clone();
//         close_button.connect_clicked(move |btn| {
//             if let Some(message_widget) = btn.parent().and_then(|p| p.parent()) {
//                 if let Some(revealer) = message_widget.downcast_ref::<Revealer>() {
//                     Self::animate_message_out(revealer, &messages_container, &messages);
//                 }
//             }
//         });

//         message_box
//     }

//     /// Create revealer for message animation
//     fn create_message_revealer(&self, message_widget: &gtk::Box) -> Revealer {
//         let revealer = Revealer::new();
//         revealer.set_child(Some(message_widget));
//         revealer.set_reveal_child(false);
//         revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
//         revealer.set_transition_duration(300);
//         revealer
//     }

//     /// Animate message in
//     fn animate_message_in(&self, revealer: &Revealer) {
//         revealer.set_reveal_child(true);
//     }

//     /// Animate message out and remove
//     fn animate_message_out(
//         revealer: &Revealer,
//         container: &gtk::Box,
//         messages: &Rc<RefCell<Vec<gtk::Widget>>>,
//     ) {
//         // revealer.set_reveal_child(false);

//         // // Remove from container after animation
//         // let container = container.clone();
//         // let messages = messages.clone();
//         // let revealer_widget: gtk::Widget = revealer.clone().upcast();

//         // glib::timeout_add_seconds_local_once(300, move || {
//         //     container.remove(&revealer_widget);
//         //     messages.borrow_mut().retain(|w| !w.eq(&revealer_widget));
//         // });
//     }

//     /// Connect discard all button
//     fn connect_discard_all(&self) {
//         let messages = self.messages.clone();
//         let container = self.messages_container.clone();
//         let button = self.discard_all_button.clone();

//         button.connect_clicked(move |_| {
//             let messages_vec: Vec<gtk::Widget> = messages.borrow().iter().cloned().collect();

//             for message in messages_vec {
//                 if let Some(revealer) = message.downcast_ref::<Revealer>() {
//                     Self::animate_message_out(revealer, &container, &messages);
//                 }
//             }
//         });
//     }

//     /// Scroll to bottom of scrolled window
//     fn scroll_to_bottom(&self) {
//         let vadjustment = self.scrolled_window.vadjustment();
//         glib::timeout_add_seconds_local_once(50, move || {
//             vadjustment.set_value(vadjustment.upper());
//         });
//     }

//     /// Get the underlying widget
//     pub fn widget(&self) -> &gtk::Widget {
//         self.widget.upcast_ref()
//     }
// }

// impl Default for MessageList {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// // Example usage
// fn main() -> glib::ExitCode {
//     let application = gtk::Application::new(
//         Some("com.example.message-list"),
//         Default::default(),
//     );

//     application.connect_activate(|app| {
//         let window = gtk::ApplicationWindow::new(app);
//         window.set_title("Message List Example");
//         window.set_default_size(400, 500);

//         // Create message list
//         let message_list = MessageList::new();

//         // Add some test messages
//         let message_list_clone = message_list.clone();
//         glib::timeout_add_local_once(100, move || {
//             message_list_clone.add_message("Welcome to the message list!");
//             message_list_clone.add_message("This is a longer message that should wrap properly in the container and show how text behaves with longer content.");
//             message_list_clone.add_message("Click the close button to remove individual messages.");
//         });

//         // Button to add more messages
//         let add_button = Button::with_label("Add Message");
//         let message_list_for_button = message_list.clone();
//         add_button.connect_clicked(move |_| {
//             let count = message_list_for_button.messages.borrow().len() + 1;
//             message_list_for_button.add_message(&format!("New message #{}", count));
//         });

//         // Main container
//         let main_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
//         main_box.set_margin_all(10);
//         main_box.append(message_list.widget());
//         main_box.append(&add_button);

//         window.set_child(Some(&main_box));
//         window.present();
//     });

//     application.run()
// }

// // Example usage in main application
// pub fn create_app() -> gtk::Application {
//     let app = gtk::Application::builder()
//         .application_id("com.example.messagewidget")
//         .build();

//     app.connect_activate(|app| {
//         let window = gtk::ApplicationWindow::builder()
//             .application(app)
//             .title("Message Widget Demo")
//             .default_width(400)
//             .default_height(500)
//             .build();

//         let main_box = Box::new(Orientation::Vertical, 12);

//         // Create message widget
//         let msg_widget = MessageWidget::new();

//         // Add some test buttons
//         let btn_box = Box::new(Orientation::Horizontal, 8);
//         btn_box.set_margin_start(8);
//         btn_box.set_margin_end(8);

//         let add_btn = Button::with_label("Add Message");
//         let add_multiple_btn = Button::with_label("Add 5 Messages");

//         let msg_widget_clone = msg_widget.clone();
//         let counter = Rc::new(RefCell::new(1u32));
//         let counter_clone = counter.clone();
//         add_btn.connect_clicked(move |_| {
//             let count = *counter_clone.borrow();
//             msg_widget_clone.add_message(&format!("Message #{}", count));
//             *counter_clone.borrow_mut() += 1;
//         });

//         let msg_widget_clone2 = msg_widget.clone();
//         add_multiple_btn.connect_clicked(move |_| {
//             for i in 0..5 {
//                 let count = *counter.borrow() + i;
//                 msg_widget_clone2.add_message(&format!("Batch message #{}", count));
//             }
//             *counter.borrow_mut() += 5;
//         });

//         btn_box.append(&add_btn);
//         btn_box.append(&add_multiple_btn);

//         main_box.append(&btn_box);
//         main_box.append(&msg_widget.widget());

//         window.set_child(Some(&main_box));
//         window.present();
//     });

//     app
// }
