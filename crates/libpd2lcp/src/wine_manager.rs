use tokio::{
    fs::{File, create_dir, create_dir_all, read_dir, rename},
    io::AsyncWriteExt,
    process::Command,
};

use std::io::Write;

use crate::{error::Error, state::State};

pub const WINE_URL: &str = "https://github.com/Kron4ek/Wine-Builds/releases/download/11.14/wine-11.14-staging-tkg-amd64.tar.xz";

pub async fn fetch_wine(state: &State) -> Result<(), Error> {
    let install_path = state.wine_dir()?;

    create_dir(install_path).await?;

    let wine_bytes = reqwest::get(WINE_URL).await?.bytes().await?;

    let wine_archive_path = install_path.join("wine_tarball.tar.xz");

    let mut wine_archive_file = File::create(&wine_archive_path).await?;

    wine_archive_file.write_all(&wine_bytes).await?;

    // Now we untar it
    let status = Command::new("tar")
        .arg("-xf")
        .arg(wine_archive_path)
        .arg("-C")
        .arg(install_path)
        .status()
        .await?;

    if !status.success() {
        return Err(Error::FailedToUntarArchive);
    }

    while let Ok(Some(file)) = read_dir(install_path.join("wine-11.14-staging-tkg-amd64"))
        .await?
        .next_entry()
        .await
    {
        rename(file.path(), install_path.join(file.file_name())).await?;
    }

    Ok(())
}

pub async fn create_prefix(state: &State) -> Result<(), Error> {
    let prefix = state.wine_prefix()?;

    create_dir_all(prefix).await?;

    let status = Command::new(state.wine_exe("wineboot")?)
        .env("WINEPREFIX", prefix)
        .arg("-i")
        .status()
        .await?;

    if !status.success() {
        return Err(Error::WineInitFailed);
    }

    // wine reg add "HKCU\\Software\\Wine" /v Version /d win7 /f
    let status_set_win7 = Command::new(state.wine_exe("wine")?)
        .env("WINEPREFIX", prefix)
        .args([
            "reg",
            "add",
            "HKCU\\Software\\Wine",
            "/v",
            "Version",
            "/d",
            "win7",
            "/f",
        ])
        .status()
        .await?;

    if !status_set_win7.success() {
        return Err(Error::WineInitFailed);
    }

    std::os::unix::fs::symlink(state.d2_dir(), state.wine_dosdevice("a:")?)?;

    // Now we kill wineserver to update dosdevices
    let status_wineserver = Command::new(state.wine_exe("wineserver")?)
        .env("WINEPREFIX", prefix)
        .arg("-k")
        .status()
        .await?;

    if !status_wineserver.success() {
        return Err(Error::WineInitFailed);
    }

    Ok(())
}
