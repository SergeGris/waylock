use gtk::{
    Accessible,
    Application,
    ApplicationWindow,
    Buildable,
    ConstraintTarget,
    Native,
    Root,
    ShortcutManager,
    Widget,
    Window,
    gio,
    glib::{self, object::IsA},
};

use crate::ffmpeg_mediastream::imp;

glib::wrapper! {
    pub struct FfmpegMediaStream(ObjectSubclass<imp::FfmpegMediaStream>)
        @extends gtk::MediaStream, gtk::MediaFile,
        @implements gtk::gdk::Paintable;
}

impl FfmpegMediaStream {
    fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for FfmpegMediaStream {
    fn default() -> Self {
        Self::new()
    }
}
