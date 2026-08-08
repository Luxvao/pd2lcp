use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

use crate::{error::Error, state::State};

#[cfg(target_os = "linux")]
pub async fn install_d2(state: State, d2_installer: PathBuf) -> Result<(), Error> {
    use tokio::process::Command;

    let status_d2 = Command::new(state.wine_exe("wine")?)
        .env("WINEPREFIX", state.wine_prefix()?)
        .arg(d2_installer)
        .status()
        .await?;

    if !status_d2.success() {
        return Err(Error::FailedToInstallD2);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn install_d2_lod(state: State, d2_lod_installer: PathBuf) -> Result<(), Error> {
    use tokio::process::Command;

    let status_d2_lod = Command::new(state.wine_exe("wine")?)
        .env("WINEPREFIX", state.wine_prefix()?)
        .arg(d2_lod_installer)
        .status()
        .await?;

    if !status_d2_lod.success() {
        return Err(Error::FailedToInstallD2LOD);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn install_d2(
    state: &State,
    d2_installer: &Path,
    d2_lod_installer: &Path,
) -> Result<(), Error> {
    let status_d2 = Command::new(d2_installer).status()?;

    if !status_d2.success() {
        return Err(Error::FailedToInstallD2);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn install_d2_lod(state: &State, d2_lod_installer: &Path) -> Result<(), Error> {
    let status_d2_lod = Command::new(d2_lod_installer).status()?;

    if !status_d2_lod.success() {
        return Err(Error::FailedToInstallD2LOD);
    }

    Ok(())
}
