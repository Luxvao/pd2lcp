use serde::{Deserialize, Serialize};

use crate::error::Error;

pub const METADATA_ENDPOINT: &str = "https://pd2-client-files.projectdiablo2.com";

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameFilesMetadataRaw {
    checksum: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameFileMetadata {
    pub checksum: String,
    pub name: String,
    pub url: String,
}

impl TryFrom<GameFilesMetadataRaw> for Vec<GameFileMetadata> {
    type Error = Error;

    fn try_from(value: GameFilesMetadataRaw) -> Result<Self, Self::Error> {
        value
            .checksum
            .iter()
            .map(|i| {
                let mut parts = i.split(" ");

                let checksum = parts
                    .next()
                    .ok_or(Error::InvalidMetadata)?
                    .trim()
                    .to_string();
                let name = parts.last().ok_or(Error::InvalidMetadata)?.trim();

                Ok(GameFileMetadata {
                    checksum,
                    name: name.to_string(),
                    url: format!("{}/{}", METADATA_ENDPOINT, name),
                })
            })
            .collect::<Result<Vec<GameFileMetadata>, Error>>()
    }
}

pub async fn get_metadata() -> Result<Vec<GameFileMetadata>, Error> {
    let metadata_url = format!("{}/metadata.json", METADATA_ENDPOINT);

    let text = reqwest::get(metadata_url).await?.text().await?;

    let metadata_raw: GameFilesMetadataRaw = serde_json::from_str(&text)?;

    metadata_raw.try_into()
}
