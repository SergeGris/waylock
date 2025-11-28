pub const DOMAIN: &str = "waylock";

#[macro_export]
macro_rules! debug {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_debug!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! info {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_info!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! warning {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_warning!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! error {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_error!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! fatal {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_critical!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

pub use crate::{debug, error, fatal, info, warning};
