use std::io::Read;
use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

use crate::error::Error;
use crate::event::EventNotify;
use crate::settings::Settings;

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
        use crate::{
            event::Event,
            wine_manager::{create_prefix, fetch_wine},
        };

        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;

        let state = State {
            base: home.join("Games/pd2lcp"),
            game_files: home.join("Games/pd2lcp/game"),
            env: Environment::Wine {
                wine_binaries: home.join("Games/pd2lcp/wine"),
                wine_prefix: home.join("Games/pd2lcp/prefix"),
            },
        };

        if state.game_files.is_dir() {
            tokio::fs::remove_dir_all(&state.game_files).await?;
        }

        tokio::fs::create_dir_all(&state.game_files).await?;

        // Wine files
        let wine_files = state.wine_dir()?;

        if wine_files.is_dir() {
            tokio::fs::remove_dir_all(wine_files).await?;
        }

        notify.notify(Event::DownloadingWine)?;

        fetch_wine(&state).await?;

        notify.notify(Event::FinishedDownloadingWine)?;

        // Wine prefix
        let prefix = state.wine_prefix()?;

        if prefix.is_dir() {
            tokio::fs::remove_dir_all(prefix).await?;
        }

        notify.notify(Event::InitPrefix)?;

        create_prefix(&state).await?;

        notify.notify(Event::FinishedInitPrefix)?;

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
    pub async fn init(_: EventNotify) -> Result<State, Error> {
        let base = PathBuf::from("A:\\");
        let game_files = base.clone();

        Ok(State {
            base,
            game_files,
            env: Environment::External,
        })
    }

    #[cfg(target_os = "windows")]
    pub fn init_raw() -> Result<State, Error> {
        let base = PathBuf::from("A:\\");
        let game_files = base.clone();

        Ok(State {
            base,
            game_files,
            env: Environment::External,
        })
    }

    pub fn base(&self) -> &Path {
        &self.base
    }

    pub fn game_dir(&self) -> &Path {
        &self.game_files
    }

    pub fn d2_dir(&self) -> PathBuf {
        self.game_files.join("Diablo II")
    }

    pub fn pd2_dir(&self) -> PathBuf {
        self.d2_dir().join("ProjectD2")
    }

    pub fn filter_dir_local(&self) -> PathBuf {
        self.pd2_dir().join("filters/local")
    }

    pub fn filter_dir_online(&self) -> PathBuf {
        self.pd2_dir().join("filters/online")
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

    pub async fn serialise_settings(state: Option<State>, settings: Settings) -> Result<(), Error> {
        let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

        let settings_file_path = state.base().join("settings.toml");

        let mut settings_file = tokio::fs::File::create(&settings_file_path).await?;

        let settings_serialised = toml::to_string_pretty(&settings)?;

        settings_file
            .write_all(settings_serialised.as_bytes())
            .await?;

        Ok(())
    }

    pub fn deserialise_settings(&self) -> Settings {
        let settings_file_path = self.base().join("settings.toml");

        // This can use normal File since it's never called in the GUI at all
        let Ok(mut settings_file) = std::fs::File::open(&settings_file_path) else {
            return Settings::default();
        };

        let mut buffer = Vec::new();

        let _ = settings_file.read_to_end(&mut buffer);

        let Ok(settings) = toml::from_slice(&buffer) else {
            return Settings::default();
        };

        settings
    }
}
