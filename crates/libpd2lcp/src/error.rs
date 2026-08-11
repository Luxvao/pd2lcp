use std::sync::PoisonError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("pd2 is not initialised")]
    Pd2lcpNotInitialised,
    #[error("installation is corrupted")]
    InstallCorrupted,
    #[error("invalid game file data metadata")]
    InvalidMetadata,
    #[error("wine initialisation failed")]
    WineInitFailed,
    #[error("no home directory")]
    NoHomeDir,
    #[error("provided state is invalid")]
    InvalidState,
    #[error("failed to install d2")]
    FailedToInstallD2,
    #[error("failed to install d2 lod")]
    FailedToInstallD2LOD,
    #[error("unable to untar the archive. Is tar installed?")]
    FailedToUntarArchive,
    #[error("this operation does not exist on this platform")]
    InvalidPlatform,
    #[error("failed to parse filename")]
    FailedToParseFilename,
    #[error("mutex poisoned")]
    PoisonError,
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
