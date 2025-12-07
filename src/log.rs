pub const DOMAIN: &str = "waylock";

#[macro_export]
macro_rules! _debug {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_debug!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! _info {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_info!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! _warning {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_warning!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! _error {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_error!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

#[macro_export]
macro_rules! _fatal {
    ($fmt:literal $(, $args:expr)* ) => {
        gtk::glib::g_critical!($crate::log::DOMAIN, $fmt $(, $args)*)
    }
}

pub use crate::{_error as error, _fatal as fatal, _info as info, _warning as warning};
