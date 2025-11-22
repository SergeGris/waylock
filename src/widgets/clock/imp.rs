use std::cell::RefCell;

use gtk::{Align, Box, Label, Orientation, glib, prelude::*, subclass::prelude::*};

fn format(now: &glib::DateTime, fmt: &str) -> glib::GString {
    now.format(fmt)
        .map_err(|err| err.message)
        .unwrap_or_else(std::convert::Into::into)
}

fn update_time(time_format: &str, date_format: &str, time: &Label, date: &Label) {
    match glib::DateTime::now_local() {
        Ok(now) => {
            time.set_text(&format(&now, time_format));
            date.set_text(&format(&now, date_format));
        }
        Err(err) => {
            time.set_text(&err.message);
            date.set_text("");
        }
    }
}

// The instance struct — holds widget children (created in constructed)
#[derive(Debug, Default, glib::Properties)]
#[properties(wrapper_type = super::Clock)]
pub struct Clock {
    // store children so we can access them from the public API
    pub time_label: RefCell<Label>,
    pub date_label: RefCell<Label>,

    /// Time format
    #[property(get, set, construct)]
    pub time_format: RefCell<String>,

    /// Date format
    #[property(get, set, construct)]
    pub date_format: RefCell<String>,
}

// Boilerplate: declare the GObject type.
#[glib::object_subclass]
impl ObjectSubclass for Clock {
    const NAME: &'static str = "Clock";
    type Type = super::Clock;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for Clock {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(8);
        obj.set_halign(Align::Center);

        let time_label = Label::new(None);
        let date_label = Label::new(None);

        time_label.add_css_class("clock-label");
        date_label.add_css_class("date-label");

        update_time(
            &self.time_format.borrow(),
            &self.date_format.borrow(),
            &time_label,
            &date_label,
        );

        obj.append(&time_label);
        obj.append(&date_label);

        glib::timeout_add_local(
            std::time::Duration::from_secs(1),
            glib::clone!(
                #[strong(rename_to = clock)]
                self.obj(),
                move || {
                    update_time(
                        &clock.imp().time_format.borrow(),
                        &clock.imp().date_format.borrow(),
                        &clock.imp().time_label.borrow(),
                        &clock.imp().date_label.borrow(),
                    );
                    glib::ControlFlow::Continue
                }
            ),
        );

        *self.time_label.borrow_mut() = time_label;
        *self.date_label.borrow_mut() = date_label;
    }
}

impl WidgetImpl for Clock {}
impl BoxImpl for Clock {}

// use glib::{DateTime, translate::ToGlibPtr};
// use gtk::{Application, ApplicationWindow, Box, Button, DrawingArea, Entry, Label, Orientation};
// use gtk::{cairo, glib, prelude::*, subclass::prelude::*};
// use std::f64::consts::PI;
// use std::ffi::CString;
// use std::time::SystemTime;

// fn set_label_fontsize(label: &Label, font_size: f64) {
//     let css_provider = gtk::CssProvider::new();
//     css_provider.load_from_data(&format!("label {{ font-size: {font_size}px; }}"));
//     label
//         .style_context()
//         .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
// }

// // Alternative function with different date format preferences
// fn get_localized_date_string(datetime: &DateTime) -> glib::GString {
//     // return datetime.format("%A, %d %B %Y").unwrap();

//     // Check locale preferences by trying different formats
//     let format_options = [
//         "%A, %d %B %Y",  // European style: "Monday, 15 January 2024"
//         "%A, %B %d, %Y", // US style: "Monday, January 15, 2024"
//         "%a, %d %b %Y",  // Short: "Mon, 15 Jan 2024"
//         "%x",            // System preferred date format
//         "%F",            // ISO format: "2024-01-15"
//     ];

//     for format in &format_options {
//         if let Ok(formatted) = datetime.format(format) {
//             if !formatted.is_empty() {
//                 return formatted;
//             }
//         }
//     }

//     "Date unavailable".into()
// }

// fn get_localized_date_string(datetime: &DateTime) -> glib::GString {
//     // Try different localized format strings
//     // GLib will automatically use the system locale for month/day names

//     // First try full format with day name
//     if let Ok(date_str) = datetime.format("%A, %d %B, %Y") {
//         return date_str;
//     }

//     // First try full format with day name
//     if let Ok(date_str) = datetime.format("%A, %B %d, %Y") {
//         return date_str;
//     }

//     // Try without day name
//     if let Ok(date_str) = datetime.format("%B %d, %Y") {
//         return date_str;
//     }

//     // Try short format
//     if let Ok(date_str) = datetime.format("%x") {
//         return date_str;
//     }

//     // Try ISO format as last resort
//     if let Ok(date_str) = datetime.format("%Y-%m-%d") {
//         return date_str;
//     }

//     // Final fallback
//     "Date unavailable".into()
// }

// // Function to get just the date parts if you need more control
// fn get_date_components(datetime: &DateTime) -> (glib::GString, glib::GString, i32, i32) {
//     let weekday = datetime.format("%A").unwrap_or_default();
//     let month = datetime.format("%B").unwrap_or_default();
//     let day = datetime.day_of_month();
//     let year = datetime.year();

//     (weekday, month, day, year)
// }

// fn create_custom_date_format(datetime: &DateTime) -> String {
//     let (weekday, month, day, year) = get_date_components(datetime);
//     format!("{}, {} {}, {}", weekday, month, day, year)
// }

// fn create_analog_clock() -> DrawingArea {
//     let drawing_area = DrawingArea::new();
//     drawing_area.set_size_request(400, 400);

//     // Set up the drawing function
//     drawing_area.set_draw_func(move |_, cr, width, height| {
//         draw_analog_clock(cr, width, height);
//     });

//     // Set up timer to update every second
//     let drawing_area_clone = drawing_area.clone();
//     glib::timeout_add_seconds_local(1, move || {
//         drawing_area_clone.queue_draw();
//         glib::ControlFlow::Continue
//     });

//     drawing_area
// }

// fn draw_analog_clock(cr: &cairo::Context, width: i32, height: i32) {
//     let center_x = width as f64 / 2.0;
//     let center_y = height as f64 / 2.0;
//     let radius = f64::min(center_x, center_y) - 20.0;

//     // Clear background
//     cr.set_source_rgb(0.9, 0.9, 0.9);
//     cr.paint().unwrap();

//     // Draw clock face
//     cr.set_source_rgb(1.0, 1.0, 1.0);
//     cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
//     cr.fill_preserve().unwrap();
//     cr.set_source_rgb(0.0, 0.0, 0.0);
//     cr.set_line_width(2.0);
//     cr.stroke().unwrap();

//     // Draw hour markers
//     cr.set_line_width(4.0);
//     for i in 0..12 {
//         let angle = (i as f64) * PI / 6.0;
//         let x1 = center_x + (radius - 30.0) * angle.cos();
//         let y1 = center_y + (radius - 30.0) * angle.sin();
//         let x2 = center_x + (radius - 10.0) * angle.cos();
//         let y2 = center_y + (radius - 10.0) * angle.sin();
//         cr.move_to(x1, y1);
//         cr.line_to(x2, y2);
//     }
//     cr.stroke().unwrap();

//     // Draw minute markers
//     cr.set_line_width(1.0);
//     for i in 0..60 {
//         if i % 5 != 0 { // Skip hour markers
//             let angle = (i as f64) * PI / 30.0;
//             let x1 = center_x + (radius - 15.0) * angle.cos();
//             let y1 = center_y + (radius - 15.0) * angle.sin();
//             let x2 = center_x + (radius - 5.0) * angle.cos();
//             let y2 = center_y + (radius - 5.0) * angle.sin();
//             cr.move_to(x1, y1);
//             cr.line_to(x2, y2);
//         }
//     }
//     cr.stroke().unwrap();

//     // Get current time
//     let now: DateTime<Local> = Local::now();
//     let seconds = now.second() as f64;
//     let minutes = now.minute() as f64 + seconds / 60.0;
//     let hours = (now.hour() % 12) as f64 + minutes / 60.0;

//     // Draw hour hand
//     let hour_angle = (hours * PI / 6.0) - (PI / 2.0);
//     let hour_x = center_x + (radius * 0.5) * hour_angle.cos();
//     let hour_y = center_y + (radius * 0.5) * hour_angle.sin();
//     cr.set_source_rgb(0.0, 0.0, 0.0);
//     cr.set_line_width(6.0);
//     cr.move_to(center_x, center_y);
//     cr.line_to(hour_x, hour_y);
//     cr.stroke().unwrap();

//     // Draw minute hand
//     let minute_angle = (minutes * PI / 30.0) - (PI / 2.0);
//     let minute_x = center_x + (radius * 0.7) * minute_angle.cos();
//     let minute_y = center_y + (radius * 0.7) * minute_angle.sin();
//     cr.set_source_rgb(0.0, 0.0, 0.0);
//     cr.set_line_width(4.0);
//     cr.move_to(center_x, center_y);
//     cr.line_to(minute_x, minute_y);
//     cr.stroke().unwrap();

//     // Draw second hand
//     let second_angle = (seconds * PI / 30.0) - (PI / 2.0);
//     let second_x = center_x + (radius * 0.9) * second_angle.cos();
//     let second_y = center_y + (radius * 0.9) * second_angle.sin();
//     cr.set_source_rgb(1.0, 0.0, 0.0); // Red color
//     cr.set_line_width(2.0);
//     cr.move_to(center_x, center_y);
//     cr.line_to(second_x, second_y);
//     cr.stroke().unwrap();

//     // Draw center dot
//     cr.set_source_rgb(0.0, 0.0, 0.0);
//     cr.arc(center_x, center_y, 8.0, 0.0, 2.0 * PI);
//     cr.fill().unwrap();

//     // Draw numbers
//     cr.set_source_rgb(0.0, 0.0, 0.0);
//     cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
//     cr.set_font_size(24.0);

//     for i in 1..=12 {
//         let angle = (i as f64) * PI / 6.0 - (PI / 2.0);
//         let number_x = center_x + (radius - 40.0) * angle.cos();
//         let number_y = center_y + (radius - 40.0) * angle.sin();
//         let text = i.to_string();
//         let text_extents = cr.text_extents(&text).unwrap();
//         cr.move_to(number_x - text_extents.width() / 2.0, number_y + text_extents.height() / 2.0);
//         cr.show_text(&text).unwrap();
//     }
// }
