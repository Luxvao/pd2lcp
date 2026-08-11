use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::filter::Filter;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GraphicsMode {
    DDRAW,
    _3DFX,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    // Launch args
    pub graphics_mode: GraphicsMode,
    pub skiptobnet: bool,
    pub sndbkg: bool,
    pub no_updates: bool,

    // I just cache filters here
    pub downloaded_filters: Vec<Filter>,
    pub active_filter: Option<Filter>,
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
            downloaded_filters: Vec::new(),
            active_filter: None,
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
