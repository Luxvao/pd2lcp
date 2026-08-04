use std::path::PathBuf;

use tokio::runtime::Runtime;

use crate::{
    base_game::{install_d2, install_d2_lod},
    event::EventNotify,
    pd2_updater::install_pd2,
    state::State,
};

pub mod base_game;
pub mod error;
pub mod event;
pub mod launch;
pub mod metadata;
pub mod pd2_updater;
pub mod settings;
pub mod state;
pub mod utils;
pub mod wine_manager;

pub fn setup_test_run() {
    let runtime = Runtime::new().unwrap();

    runtime.block_on(async {
        let event_notify = EventNotify::default();
        let state = State::init(event_notify.clone()).await.unwrap();

        install_d2(
            &state,
            PathBuf::from("/home/bor/Downloads/Downloader_Diablo2_enUS.exe").as_path(),
        )
        .await
        .unwrap();

        install_d2_lod(
            &state,
            PathBuf::from("/home/bor/Downloads/Downloader_Diablo2_Lord_of_Destruction_enUS.exe")
                .as_path(),
        )
        .await
        .unwrap();

        install_pd2(state, event_notify.clone()).await.unwrap();
    });
}
