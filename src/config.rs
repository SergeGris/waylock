use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{blur::BlurMethod, log};

#[derive(clap::Parser, Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Set GTK theme
    #[arg(long, short = 'g')]
    #[serde(default)]
    gtk_theme: Option<String>,

    /// Path to config
    #[arg(long, short = 'C')]
    #[serde(default = "default::config")]
    config: Option<PathBuf>,

    // Path to CSS style file
    #[arg(long, short = 'S')]
    #[serde(default = "default::style")]
    style: Option<PathBuf>,

    // Path to background
    #[arg(long, short = 'b')]
    #[serde(default)]
    background: Option<PathBuf>,

    /// Idle timeout in seconds
    #[arg(long)]
    #[serde(default = "default::idle_timeout")]
    idle_timeout: Option<u64>,

    /// Start with hidden form
    #[arg(long)]
    //, action = clap::ArgAction::SetTrue)] //, value_parser = clap::builder::BoolishValueParser::new())]
    #[serde(default)]
    start_hidden: Option<bool>,

    /// Set time format
    #[arg(long)]
    #[serde(default = "default::time_format")]
    time_format: Option<String>,

    /// Set date format
    #[arg(long)]
    #[serde(default = "default::date_format")]
    date_format: Option<String>,
    // TODO
    // #[arg(long)]
    // #[serde()]
    // blur_method: Option<BlurMethod>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gtk_theme: None,
            config: default::config(),
            style: default::style(),
            background: None,
            idle_timeout: default::idle_timeout(),
            start_hidden: default::start_hidden(),
            time_format: default::time_format(),
            date_format: default::date_format(),
        }
    }
}

macro_rules! merge {
    ($lhs:expr, $rhs:expr, { $($field:ident),* $(,)? }) => {
        {
            let (a, b) = ($lhs, $rhs);
            Self {
                $(
                    $field: b.$field.or(a.$field),
                )*
            }
        }
    };
}

impl Config {
    pub fn merge(self, other: Self) -> Self {
        merge!(self, other, { gtk_theme, config, style, background, idle_timeout, start_hidden, time_format, date_format })
    }

    pub const fn get_gtk_theme(&self) -> Option<&String> {
        self.gtk_theme.as_ref()
    }

    pub fn get_style(&self) -> Option<PathBuf> {
        self.style.clone().or_else(default::style)
    }

    pub fn get_background(&self) -> Option<&Path> {
        self.background.as_deref()
    }

    pub fn get_config(&self) -> Option<PathBuf> {
        self.config.clone().or_else(default::config)
    }

    pub fn get_idle_timeout(&self) -> u64 {
        self.idle_timeout.unwrap_or(default::IDLE_TIMEOUT)
    }

    pub fn get_start_hidden(&self) -> bool {
        self.start_hidden.unwrap_or(default::START_HIDDEN)
    }

    pub fn get_time_format(&self) -> &str {
        self.time_format.as_deref().unwrap_or(default::TIME_FORMAT)
    }

    pub fn get_date_format(&self) -> &str {
        self.date_format.as_deref().unwrap_or(default::DATE_FORMAT)
    }
}

pub mod default {
    pub const TIME_FORMAT: &str = "%H:%M";
    pub const DATE_FORMAT: &str = "%A, %d %B %Y";
    pub const START_HIDDEN: bool = false;
    pub const IDLE_TIMEOUT: u64 = 30;

    use std::path::PathBuf;

    pub fn config() -> Option<PathBuf> {
        xdg::BaseDirectories::with_prefix("waylock").get_config_file("config.toml")
    }

    pub fn style() -> Option<PathBuf> {
        xdg::BaseDirectories::with_prefix("waylock").get_config_file("style.css")
    }

    pub const fn idle_timeout() -> Option<u64> {
        Some(IDLE_TIMEOUT)
    }

    pub const fn start_hidden() -> Option<bool> {
        Some(START_HIDDEN)
    }

    pub fn time_format() -> Option<String> {
        Some(TIME_FORMAT.into())
    }

    pub fn date_format() -> Option<String> {
        Some(DATE_FORMAT.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config: {0}")]
    Toml(#[from] toml::de::Error),
}

// TODO
// pub fn default_config() -> String {
//     let c = Config::default();
//     toml::to_string(&c).unwrap_or_default()
// }

fn raw_load_config(path: impl AsRef<Path>) -> Result<Config, Error> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

pub fn load_config(path: impl AsRef<Path>) -> Config {
    match raw_load_config(&path) {
        Ok(c) => {
            log::info!("config loaded: {:?}", path.as_ref());
            c
        }
        Err(e) => {
            log::info!("failed load config: {e}");
            Config::default()
        }
    }
}
