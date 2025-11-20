// #[derive(Debug, Default, Deserialize)]
// pub struct Config {
//     #[serde(default)]
//     pub general: GeneralConfig,

//     #[serde(default)]
//     pub debug: DebugConfig,
// }

// #[derive(Debug, Default, Deserialize)]
// pub struct GeneralConfig {
//     #[serde(default = "default_timeout")]
//     pub timeout: u32,

//     #[serde(default)]
//     pub theme: Option<String>,

//     #[serde(default)]
//     pub lock_command: Option<String>,
// }

// #[derive(Debug, Default, Deserialize)]
// pub struct DebugConfig {
//     #[serde(default)]
//     pub verbose: bool,

//     #[serde(default)]
//     pub style_dump: bool,
// }

// fn default_timeout() -> u32 {
//     30
// }

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(clap::Parser, Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[arg(long, short = 'g')]
    #[serde(default)]
    gtk_theme: Option<String>,

    /// Path to config
    #[arg(long, short = 'C')]
    #[serde(default = "default::config")]
    config: Option<PathBuf>,

    #[arg(long, short = 'S')]
    #[serde(default = "default::style")]
    style: Option<PathBuf>,

    #[arg(long, short = 'b')]
    #[serde(default)]
    background: Option<PathBuf>,

    /// Idle timeout in seconds
    #[arg(long)]
    #[serde(default = "default::idle_timeout")]
    idle_timeout: Option<u64>,

    /// Start with hidden form
    #[arg(long)]
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
        self.style
            .as_ref()
            .map(std::clone::Clone::clone)
            .or_else(default::style)
    }

    pub const fn get_background(&self) -> Option<&PathBuf> {
        self.background.as_ref()
    }

    pub fn get_config(&self) -> Option<PathBuf> {
        self.config
            .as_ref()
            .map(|p| p.clone())
            .or_else(default::config)
    }

    pub fn get_idle_timeout(&self) -> u64 {
        self.idle_timeout.unwrap_or(default::IDLE_TIMEOUT)
    }

    pub fn get_start_hidden(&self) -> bool {
        self.start_hidden.unwrap_or(default::START_HIDDEN)
    }

    pub fn get_time_format(&self) -> &str {
        self.time_format
            .as_ref()
            .map_or(default::TIME_FORMAT, String::as_str)
    }

    pub fn get_date_format(&self) -> &str {
        self.date_format
            .as_ref()
            .map_or(default::DATE_FORMAT, String::as_str)
    }
}

mod default {
    pub const TIME_FORMAT: &str = "%H:%M";
    pub const DATE_FORMAT: &str = "%A, %d %B %Y";
    pub const START_HIDDEN: bool = true;
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
        Some(true)
    }

    pub fn time_format() -> Option<String> {
        Some(TIME_FORMAT.into())
    }

    pub fn date_format() -> Option<String> {
        Some(DATE_FORMAT.into())
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config: {0}")]
    Toml(#[from] toml::de::Error),
}

pub fn default_config() -> String {
    let c = Config::default();
    toml::to_string(&c).unwrap_or_default()
}

fn raw_load_config(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

pub fn load_config(path: impl AsRef<Path>) -> Config {
    match raw_load_config(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: failed load config: {e}");
            Config::default()
        }
    }
}
