use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::filter::Filter;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GraphicsMode {
    DDRAW,
    _3DFX,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    // Launch args
    pub graphics_mode: GraphicsMode,
    pub skiptobnet: bool,
    pub sndbkg: bool,
    pub no_updates: bool,

    // Launcher
    pub scale_factor: f32,

    // I just cache filters here
    pub downloaded_filters: HashMap<Filter, String>,
    pub active_filter: Option<(Filter, String)>,
}

// This exists so we can actually serialise to toml
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsSerialisable {
    // Launch args
    graphics_mode: GraphicsMode,
    skiptobnet: bool,
    sndbkg: bool,
    no_updates: bool,

    // Launcher
    scale_factor: f32,

    // I just cache filters here
    downloaded_filters: Vec<(Filter, String)>,
    active_filter: Option<(Filter, String)>,
}

impl Display for GraphicsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsMode::DDRAW => write!(f, "ddraw"),
            GraphicsMode::_3DFX => write!(f, "3dfx"),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            graphics_mode: GraphicsMode::_3DFX,
            skiptobnet: true,
            sndbkg: false,
            no_updates: false,
            scale_factor: 1.0,
            downloaded_filters: HashMap::new(),
            active_filter: None,
        }
    }
}

impl From<SettingsSerialisable> for Settings {
    fn from(value: SettingsSerialisable) -> Self {
        Settings {
            graphics_mode: value.graphics_mode,
            skiptobnet: value.skiptobnet,
            sndbkg: value.sndbkg,
            no_updates: value.no_updates,
            scale_factor: value.scale_factor,
            active_filter: value.active_filter,
            downloaded_filters: value.downloaded_filters.into_iter().collect(),
        }
    }
}

impl From<Settings> for SettingsSerialisable {
    fn from(value: Settings) -> Self {
        SettingsSerialisable {
            graphics_mode: value.graphics_mode,
            skiptobnet: value.skiptobnet,
            sndbkg: value.sndbkg,
            no_updates: value.no_updates,
            scale_factor: value.scale_factor,
            active_filter: value.active_filter,
            downloaded_filters: value.downloaded_filters.into_iter().collect(),
        }
    }
}

impl Settings {
    pub fn compose_args(&self) -> Vec<&str> {
        let mut args = Vec::new();

        match self.graphics_mode {
            GraphicsMode::DDRAW => args.push("-ddraw"),
            GraphicsMode::_3DFX => args.push("-3dfx"),
        }

        if self.skiptobnet {
            args.push("-skiptobnet");
        }

        if self.sndbkg {
            args.push("-sndbkg");
        }

        args
    }
}
