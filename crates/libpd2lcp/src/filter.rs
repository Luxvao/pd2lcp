use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::{
    error::Error,
    event::{Event, EventNotify},
    state::State,
};

pub const FILTER_GROUPS_LIST: &str =
    "https://raw.githubusercontent.com/Project-Diablo-2/LootFilters/refs/heads/main/filters.json";

pub const LOCAL_FILTER_GROUP: LazyLock<FilterGroup> = LazyLock::new(FilterGroup::local);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterGroup {
    pub name: String,
    pub url: String,
    pub author: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    pub grouping: FilterGroup,
    pub name: String,
    pub url: String,
    pub sha: String,
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

impl FilterGroup {
    pub fn local() -> FilterGroup {
        FilterGroup {
            name: "Local".to_string(),
            url: "n/a".to_string(),
            author: "local".to_string(),
        }
    }
}

impl Filter {
    pub fn get_filename(&self) -> String {
        format!(
            "{}-{}",
            self.grouping.name.replace(" ", "_"),
            self.name.replace(" ", "_"),
        )
    }
}

pub async fn get_filter_authors() -> Result<Vec<FilterGroup>, Error> {
    let filter_groups_list_json = reqwest::get(FILTER_GROUPS_LIST).await?.text().await?;

    let mut filters = vec![LOCAL_FILTER_GROUP.clone()];

    // If nothing else we might as well show the local ones
    let mut online_filters: Vec<FilterGroup> =
        serde_json::from_str(&filter_groups_list_json).unwrap_or_default();

    filters.append(&mut online_filters);

    Ok(filters)
}

pub async fn get_filters(state: Option<State>, group: FilterGroup) -> Result<Vec<Filter>, Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let local = &group == &*LOCAL_FILTER_GROUP;

    if local {
        let mut filters = Vec::new();

        let filter_dir_local = state.filter_dir_local();

        tokio::fs::create_dir_all(&filter_dir_local).await?;

        let mut read_dir = tokio::fs::read_dir(&filter_dir_local).await?;

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let entry = entry.path();

            let entry_filename = entry
                .file_name()
                .ok_or(Error::FailedToParseFilename)?
                .to_str()
                .ok_or(Error::FailedToParseFilename)?
                .to_string();

            if entry.is_file() && entry_filename.ends_with(".filter") {
                filters.push(Filter {
                    grouping: LOCAL_FILTER_GROUP.clone(),
                    name: entry_filename,
                    url: "n/a".to_string(),
                    sha: "n/a".to_string(),
                });
            }
        }

        return Ok(filters);
    }

    let client = reqwest::Client::builder()
        .user_agent("pd2lcp/0.1.0 (+https://github.com/Luxvao/pd2lcp)")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "Authorization",
                format!(
                    "Bearer {}",
                    std::env::var("GITHUB_TOKEN").unwrap_or_default()
                )
                .parse()
                .unwrap(),
            );
            headers
        })
        .build()?;

    let filters_json = client.get(&group.url).send().await?.text().await?;

    Ok(serde_json::from_str::<Vec<GhResp>>(&filters_json)
        .unwrap_or_default()
        .iter()
        .filter(|i| i.type_field == "file" && i.name.ends_with(".filter"))
        .filter_map(|i| {
            i.download_url.clone().map(|url| Filter {
                grouping: group.clone(),
                name: i.name.clone().replace("_", "-"),
                url,
                sha: i.sha.clone(),
            })
        })
        .collect())
}

pub async fn download_filter(state: Option<State>, filter: Filter) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let filter_dir_online = state.filter_dir_online();

    tokio::fs::create_dir_all(&filter_dir_online).await?;

    let filter_contents = reqwest::get(&filter.url).await?.bytes().await?;

    let filter_file_path = filter_dir_online.join(filter.get_filename());

    let mut filter_file = tokio::fs::File::create(&filter_file_path).await?;

    filter_file.write_all(&filter_contents).await?;

    Ok(())
}

pub async fn delete_filter(state: Option<State>, filter: Filter) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let local = &filter.grouping == &*LOCAL_FILTER_GROUP;

    if local {
        let filter_dir_local = state.filter_dir_local();

        tokio::fs::create_dir_all(&filter_dir_local).await?;

        tokio::fs::remove_file(filter_dir_local.join(filter.name)).await?;

        return Ok(());
    }

    let filter_dir_online = state.filter_dir_online();

    tokio::fs::create_dir_all(&filter_dir_online).await?;

    tokio::fs::remove_file(filter_dir_online.join(filter.get_filename())).await?;

    Ok(())
}

pub async fn check_for_update(state: Option<State>, filter: Filter) -> Result<bool, Error> {
    let filters = get_filters(state, filter.grouping.clone()).await?;

    if filters
        .iter()
        .any(|online_filter| online_filter.name == filter.name && online_filter.sha != filter.sha)
    {
        // If we're here then there's an update
        return Ok(true);
    }

    Ok(false)
}

pub async fn activate_filter(
    state: Option<State>,
    filter: Option<Filter>,
    event_notify: EventNotify,
) -> Result<(), Error> {
    let state = state.ok_or(Error::Pd2lcpNotInitialised)?;

    let loot_filter_path = state.pd2_dir().join("loot.filter");

    if loot_filter_path.exists() {
        tokio::fs::remove_file(&loot_filter_path).await?;
    }

    if let Some(filter) = filter {
        if &filter.grouping == &*LOCAL_FILTER_GROUP {
            let selected_filter_path = state.filter_dir_local().join(filter.name);

            tokio::fs::copy(selected_filter_path, loot_filter_path).await?;

            return Ok(());
        }

        if check_for_update(Some(state.clone()), filter.clone()).await? {
            // Update available
            event_notify.notify(Event::UpdatingFilter)?;

            download_filter(Some(state.clone()), filter.clone()).await?;

            event_notify.notify(Event::DoneUpdatingFilter)?;
        }

        let selected_filter_path = state.filter_dir_online().join(filter.get_filename());

        tokio::fs::copy(selected_filter_path, loot_filter_path).await?;
    }

    Ok(())
}
