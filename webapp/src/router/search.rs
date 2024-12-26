
use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};
use axum_valid::Valid;

use serde::Deserialize;
use shared::
    model::{Event, SearchItem}
;
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
}

#[derive(Template, Debug)]
#[template(path = "search.html")]
struct SearchTemplate {
    query: String,
    items: Vec<SearchItem>,
    current_event: Option<Event>,
}

#[tracing::instrument(skip(state))]
pub async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<SearchParams>>,
) -> axum::response::Result<Html<String>> {
    info!("search params: {:?}", params);
    match state.queryable.search(&params.q, params.limit, true).await {
        Ok(items) => {
            let page = SearchTemplate {
                query: params.q,
                items,
                current_event: None,
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("search failed".into()),
    }
}
