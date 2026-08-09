use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

use crate::{
    error::Error,
    event::{Event, EventNotify},
    metadata::{GameFileMetadata, get_metadata},
    state::State,
    utils::compute_hash,
};

pub async fn update_available(state: Option<State>) -> Result<bool, Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let remote_metadata = get_metadata().await?;

    let local_metadata_path = state.pd2_dir().join("local_metadata.json");

    if !local_metadata_path.is_file() {
        return Ok(true);
    }

    let mut local_metadata_file = File::open(&local_metadata_path).await?;

    let mut buffer = Vec::new();

    local_metadata_file.read_to_end(&mut buffer).await?;

    let local_metadata: Vec<GameFileMetadata> = serde_json::from_slice(&buffer)?;

    if local_metadata != remote_metadata {
        return Ok(true);
    }

    Ok(false)
}

pub async fn install_pd2(state: Option<State>, notify: EventNotify) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let metadata_game_files = get_metadata().await?;

    let pd2_files = state.pd2_dir();

    if !pd2_files.is_dir() {
        std::fs::create_dir_all(&pd2_files)?;
    }

    let total = metadata_game_files.len() as u32;

    let mut done: i64 = -1;

    for file_metadata in metadata_game_files.iter() {
        done += 1;

        notify.notify(Event::UpdatingPD2 {
            done: done as u32,
            total,
        })?;

        let output_file_path = pd2_files.join(&file_metadata.name);

        if !output_file_path.exists() {
            let data = reqwest::get(&file_metadata.url).await?.bytes().await?;

            let mut output_file = File::create(&output_file_path).await?;

            output_file.write_all(&data).await?;

            continue;
        }

        let mut output_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&output_file_path)
            .await?;

        let mut buffer = Vec::new();

        output_file.read_to_end(&mut buffer).await?;

        let local_hash = compute_hash(&buffer)?;

        if file_metadata.checksum != local_hash {
            let data = reqwest::get(&file_metadata.url).await?.bytes().await?;

            // We seek to 0 and write everything
            output_file.seek(std::io::SeekFrom::Start(0)).await?;
            output_file.set_len(0).await?;

            output_file.write_all(&data).await?;
        }
    }

    // Now we write to the local_metadata.json file
    let local_metadata_path = pd2_files.join("local_metadata.json");

    let mut local_metadata_file = File::create(local_metadata_path).await?;

    let serialised = serde_json::to_vec_pretty(&metadata_game_files)?;

    local_metadata_file.write_all(&serialised).await?;

    notify.notify(Event::DoneUpdating)?;

    Ok(())
}
