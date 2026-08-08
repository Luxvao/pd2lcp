use std::{
    fs::{File, OpenOptions},
    io::Write,
};

use crate::{
    error::Error,
    event::{Event, EventNotify},
    metadata::{GameFileMetadata, get_metadata},
    state::State,
    utils::compute_hash,
};

pub async fn update_available(state: State) -> Result<bool, Error> {
    let remote_metadata = get_metadata().await?;

    let local_metadata_path = state.pd2_dir().join("local_metadata.json");

    if !local_metadata_path.is_file() {
        return Ok(true);
    }

    let local_metadata_file = File::open(&local_metadata_path)?;

    let local_metadata: Vec<GameFileMetadata> = serde_json::from_reader(local_metadata_file)?;

    if local_metadata != remote_metadata {
        return Ok(true);
    }

    Ok(false)
}

pub async fn install_pd2(state: State, notify: EventNotify) -> Result<(), Error> {
    let metadata_game_files = get_metadata().await?;

    let pd2_files = state.pd2_dir();

    if !pd2_files.is_dir() {
        std::fs::create_dir_all(&pd2_files)?;
    }

    let total = metadata_game_files.len() as u32;

    let mut done = 0;

    for file_metadata in metadata_game_files.iter() {
        let output_file_path = pd2_files.join(&file_metadata.name);

        if !output_file_path.exists() {
            let data = reqwest::get(&file_metadata.url).await?.bytes().await?;

            let mut output_file = File::create(&output_file_path)?;

            output_file.write_all(&data)?;

            done += 1;

            notify.notify(Event::UpdatingPD2 { done, total })?;

            continue;
        }

        let mut output_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&output_file_path)?;

        if file_metadata.checksum != compute_hash(&output_file)? {
            let data = reqwest::get(&file_metadata.url).await?.bytes().await?;

            output_file.write_all(&data)?;
        }

        done += 1;

        notify.notify(Event::UpdatingPD2 { done, total })?;
    }

    // Now we write to the local_metadata.json file
    let local_metadata_path = pd2_files.join("local_metadata.json");

    let local_metadata_file = File::create(local_metadata_path)?;

    serde_json::to_writer_pretty(local_metadata_file, &metadata_game_files)?;

    notify.notify(Event::DoneUpdating)?;

    Ok(())
}
