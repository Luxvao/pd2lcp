use crate::{error::Error, settings::Settings, state::State};
use tokio::process::Command;

#[cfg(target_os = "linux")]
pub async fn launch(state: Option<State>, settings: Settings) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let game_path = state.pd2_dir().join("Game.exe");
    let args = settings.compose_args();

    Command::new(state.wine_exe("wine")?)
        .env("WINEPREFIX", state.wine_prefix()?)
        .current_dir(state.pd2_dir())
        .arg(game_path)
        .args(args)
        .status()
        .await?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn launch(state: Option<State>, settings: Settings) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let game_path = state.pd2_dir().join("Game.exe");
    let args = settings.compose_args();

    Command::new(game_path)
        .args(args)
        .current_dir(state.pd2_dir())
        .status()
        .await?;

    Ok(())
}
