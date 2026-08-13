use std::sync::PoisonError;

use thiserror::Error;

pub trait ResultContextExt<T> {
    fn context(self, ctx: &str) -> Result<T, Error>;
    fn with_context<F: FnOnce(&Error) -> String>(self, f: F) -> Result<T, Error>;
}

impl<T, E: Into<Error>> ResultContextExt<T> for Result<T, E> {
    fn context(self, ctx: &str) -> Result<T, Error> {
        self.map_err(|e| {
            let e = e.into();

            Error::Context {
                message: format!("Context: {ctx}\nError:{}", e.to_string()),
                source: Box::new(e),
            }
        })
    }

    fn with_context<F: FnOnce(&Error) -> String>(self, f: F) -> Result<T, Error> {
        self.map_err(|e| {
            let e = e.into();

            Error::Context {
                message: f(&e),
                source: Box::new(e),
            }
        })
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{message}")]
    Context {
        message: String,
        #[source]
        source: Box<Error>,
    },
    #[error("PD2 is not initialised")]
    Pd2lcpNotInitialised,
    #[error("Installation is corrupted")]
    InstallCorrupted,
    #[error("Invalid game file data metadata")]
    InvalidMetadata,
    #[error("Wine initialisation failed")]
    WineInitFailed,
    #[error("No home directory")]
    NoHomeDir,
    #[error("Provided state is invalid")]
    InvalidState,
    #[error("Failed to install d2")]
    FailedToInstallD2,
    #[error("Failed to install d2 lod")]
    FailedToInstallD2LOD,
    #[error("Unable to untar the archive. Is tar installed?")]
    FailedToUntarArchive,
    #[error("This operation does not exist on this platform")]
    InvalidPlatform,
    #[error("Failed to parse filename")]
    FailedToParseFilename,
    #[error("Mutex poisoned")]
    PoisonError,
    #[error("D2 is not installed correctly, reinstall")]
    D2InstalledIncorrectly,
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("{0}")]
    VarError(#[from] std::env::VarError),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("{0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("{0}")]
    TomlSerError(#[from] toml::ser::Error),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::PoisonError
    }
}
