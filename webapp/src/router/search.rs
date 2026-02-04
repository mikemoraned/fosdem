use std::{fmt, str::FromStr};

use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};
use axum_valid::Valid;

use serde::{de, Deserialize, Deserializer};
use shared::model::{Event, SearchItem};
use tracing::info;
use validator::Validate;

use crate::filters;
use crate::state::AppState;
use shared::queryable::Queryable;

#[derive(Deserialize, Validate, Debug)]
pub struct SearchParams {
    #[validate(length(min = 2, max = 100))]
    q: String,
    #[validate(range(min = 1, max = 20))]
    limit: u8,
    #[validate(range(min = 2024, max = 2026))]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    year: Option<u32>,
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Template, Debug)]
#[template(path = "search.html")]
struct SearchTemplate {
    query: String,
    year: Option<u32>,
    items: Vec<SearchItem>,
    has_videos: bool,
    current_event: Option<Event>, // TODO: remove this
    current_fosdem: shared::model::CurrentFosdem,
}

#[tracing::instrument(skip(state))]
pub async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<SearchParams>>,
) -> axum::response::Result<Html<String>> {
    info!("search params: {:?}", params);
    match state
        .queryable
        .search(&params.q, params.limit, true, params.year)
        .await
    {
        Ok(items) => {
            let has_videos = items.iter().any(|item| item.event.has_video());
            let page = SearchTemplate {
                query: params.q,
                year: params.year,
                items,
                has_videos,
                current_event: None,
                current_fosdem: state.current_fosdem.clone(),
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("search failed".into()),
    }
}
