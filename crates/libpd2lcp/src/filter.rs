use serde::{Deserialize, Serialize};

use crate::error::Error;

pub const FILTER_GROUPS_LIST: &str =
    "https://raw.githubusercontent.com/Project-Diablo-2/LootFilters/refs/heads/main/filters.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterGroup {
    pub name: String,
    pub url: String,
    pub author: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GhResp {
    name: String,
    path: String,
    sha: String,
    size: u32,
    url: String,
    html_url: String,
    download_url: Option<String>,
    #[serde(rename = "type")]
    type_field: String,
    _links: GhLinksField,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GhLinksField {
    #[serde(rename = "self")]
    self_field: String,
    git: String,
    html: String,
}

#[derive(Clone, Debug)]
pub struct Filter {
    pub name: String,
    pub url: String,
}

pub async fn get_filter_authors() -> Result<Vec<FilterGroup>, Error> {
    let filter_groups_list_json = reqwest::get(FILTER_GROUPS_LIST).await?.text().await?;

    serde_json::from_str(&filter_groups_list_json).map_err(|e| e.into())
}

pub async fn get_filters(url: &str) -> Result<Vec<Filter>, Error> {
    let filters_json = reqwest::get(url).await?.text().await?;

    Ok(serde_json::from_str::<Vec<GhResp>>(&filters_json)?
        .iter()
        .filter(|i| i.type_field == "file" && i.name.ends_with(".filter"))
        .filter_map(|i| {
            i.download_url.clone().map(|url| Filter {
                name: i.name.clone(),
                url,
            })
        })
        .collect())
}
