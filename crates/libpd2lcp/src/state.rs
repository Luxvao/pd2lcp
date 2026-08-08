use std::path::{Path, PathBuf};

use crate::error::Error;
#[cfg(target_os = "linux")]
use crate::event::{Event, EventNotify};

#[derive(Clone, Debug)]
pub struct State {
    base: PathBuf,
    game_files: PathBuf,
    env: Environment,
}

#[derive(Clone, Debug)]
pub enum Environment {
    Wine {
        wine_binaries: PathBuf,
        wine_prefix: PathBuf,
    },
    External,
}

impl State {
    #[cfg(target_os = "linux")]
    pub async fn init(notify: EventNotify) -> Result<State, Error> {
        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;

        let state = State {
            base: home.join("Games/pd2lcp"),
            game_files: home.join("Games/pd2lcp/game"),
            env: Environment::Wine {
                wine_binaries: home.join("Games/pd2lcp/wine"),
                wine_prefix: home.join("Games/pd2lcp/prefix"),
            },
        };

        if !state.game_files.is_dir() {
            std::fs::create_dir_all(&state.game_files)?;
        }

        if !state.wine_dir()?.is_dir() {
            use crate::wine_manager::fetch_wine;

            notify.notify(Event::DownloadingWine)?;

            fetch_wine(&state).await?;

            notify.notify(Event::FinishedDownloadingWine)?;
        }

        if !state.wine_prefix()?.is_dir() {
            use crate::wine_manager::create_prefix;

            notify.notify(Event::InitPrefix)?;

            create_prefix(&state).await?;

            notify.notify(Event::FinishedInitPrefix)?;
        }

        Ok(state)
    }

    #[cfg(target_os = "linux")]
    pub fn init_raw() -> Result<State, Error> {
        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;

        Ok(State {
            base: home.join("Games/pd2lcp"),
            game_files: home.join("Games/pd2lcp/game"),
            env: Environment::Wine {
                wine_binaries: home.join("Games/pd2lcp/wine"),
                wine_prefix: home.join("Games/pd2lcp/prefix"),
            },
        })
    }

    #[cfg(target_os = "windows")]
    pub fn init() -> Result<State, Error> {
        let pd2_files = PathBuf::from("A:\\");

        Ok(State {
            pd2_files,
            env: Environment::Winlator,
        })
    }

    pub fn base(&self) -> &Path {
        &self.base
    }

    pub fn d2_dir(&self) -> &Path {
        &self.game_files
    }

    pub fn pd2_dir(&self) -> PathBuf {
        self.game_files.join("Diablo II/ProjectD2")
    }

    pub fn wine_dir(&self) -> Result<&Path, Error> {
        if let Environment::Wine {
            ref wine_binaries,
            wine_prefix: _,
        } = self.env
        {
            return Ok(wine_binaries);
        }

        Err(Error::InvalidPlatform)
    }

    pub fn wine_prefix(&self) -> Result<&Path, Error> {
        if let Environment::Wine {
            wine_binaries: _,
            ref wine_prefix,
        } = self.env
        {
            return Ok(wine_prefix);
        }

        Err(Error::InvalidPlatform)
    }

    pub fn wine_exe(&self, exe: &str) -> Result<PathBuf, Error> {
        self.wine_dir().map(|p| p.join("bin").join(exe))
    }

    pub fn wine_dosdevice(&self, dosdevice: &str) -> Result<PathBuf, Error> {
        self.wine_prefix()
            .map(|p| p.join("dosdevices").join(dosdevice))
    }
}
