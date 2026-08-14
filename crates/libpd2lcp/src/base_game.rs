use std::path::PathBuf;

use tokio::process::Command;

use crate::{error::Error, state::State};

#[cfg(target_os = "linux")]
pub async fn install_d2(state: Option<State>, d2_installer: PathBuf) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

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
pub async fn install_d2_lod(state: Option<State>, d2_lod_installer: PathBuf) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

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
pub async fn install_d2(_: Option<State>, d2_installer: PathBuf) -> Result<(), Error> {
    let status_d2 = Command::new(d2_installer).status().await?;

    if !status_d2.success() {
        return Err(Error::FailedToInstallD2);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn install_d2_lod(_: Option<State>, d2_lod_installer: PathBuf) -> Result<(), Error> {
    let status_d2_lod = Command::new(d2_lod_installer).status().await?;

    if !status_d2_lod.success() {
        return Err(Error::FailedToInstallD2LOD);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn relocate_files(state: Option<State>) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let prefix = state.wine_prefix()?;

    let output = Command::new(state.wine_exe("wine")?)
        .env("WINEPREFIX", prefix)
        .args([
            "reg",
            "query",
            "HKCU\\Software\\Blizzard Entertainment\\Diablo II",
            "/v",
            "InstallPath",
        ])
        .output()
        .await?;

    let output_contents = output.stdout.iter().map(|i| *i as char).collect::<String>();

    if output_contents.trim() == "reg: Unable to find the specified registry key" {
        return Err(Error::D2InstalledIncorrectly);
    }

    let install_path = output_contents
        .split("    ")
        .last()
        .ok_or(Error::D2InstalledIncorrectly)?
        .trim();

    if install_path == "A:\\Diablo II\\" {
        return Ok(());
    }

    let drive_letter = install_path
        .chars()
        .take(2)
        .collect::<String>()
        .to_ascii_lowercase();

    if drive_letter.len() != 2 {
        return Err(Error::D2InstalledIncorrectly);
    }

    let relative_path = install_path.get(3..).unwrap_or_default().replace("\\", "/");

    let full_path = state.wine_dosdevice(&drive_letter)?.join(relative_path);

    let d2_dir = state.d2_dir();

    if d2_dir.is_dir() {
        tokio::fs::remove_dir_all(&d2_dir).await?;
    }

    tokio::fs::symlink(full_path, d2_dir).await?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn relocate_files(_: Option<State>) -> Result<(), Error> {
    use crate::error::ResultContextExt;

    use winreg::enums::{KEY_READ, KEY_WRITE};

    use crate::utils::copy_dir_all;

    let target_path = PathBuf::from("A:\\Diablo II");

    let hkcu_d2 = winreg::HKCU.open_subkey_with_flags(
        "Software\\Blizzard Entertainment\\Diablo II",
        KEY_READ | KEY_WRITE,
    )?;
    let hklm_d2 = winreg::HKLM
        .open_subkey_with_flags(
            "Software\\Blizzard Entertainment\\Diablo II",
            KEY_READ | KEY_WRITE,
        )
        .or_else(|_| {
            winreg::HKLM.open_subkey_with_flags(
                "Software\\Wow6432Node\\Blizzard Entertainment\\Diablo II",
                KEY_READ | KEY_WRITE,
            )
        })?;

    let current_path: String = hklm_d2.get_value("InstallPath")?;

    if current_path.as_str() == "A:\\Diablo II\\" {
        return Ok(());
    }

    if target_path.is_dir() {
        tokio::fs::remove_dir_all(&target_path).await?;
    }

    // Can't rename because it's on different mounts usually
    copy_dir_all(&current_path, target_path)
        .await
        .context("Failed to copy d2 files")?;
    tokio::fs::remove_dir_all(current_path)
        .await
        .context("Failed to delete original directory")?;

    hkcu_d2.set_value("InstallPath", &"A:\\Diablo II".to_string())?;
    hkcu_d2.set_value("GamePath", &"A:\\Diablo II\\Game.exe".to_string())?;

    hklm_d2.set_value("InstallPath", &"A:\\Diablo II".to_string())?;
    hklm_d2.set_value("GamePath", &"A:\\Diablo II\\Game.exe".to_string())?;

    Ok(())
}
