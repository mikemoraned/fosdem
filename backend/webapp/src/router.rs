use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::Method,
    response::Html,
    routing::get,
    Router,
};
use axum_valid::Valid;

use serde::Deserialize;
use shared::{
    inmemory_openai::InMemoryOpenAIQueryable,
    postgres_openai::PostgresOpenAIQueryable,
    queryable::{NextEvents, NextEventsContext, SearchItem},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;
use validator::Validate;

use crate::filters;
use crate::related::related;
use crate::state::AppState;
use shared::queryable::Queryable;

#[derive(Deserialize, Validate, Debug)]
struct SearchParams {
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
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[tracing::instrument]
async fn index() -> Html<String> {
    let page = IndexTemplate {};
    let html = page.render().unwrap();
    Html(html)
}

#[tracing::instrument(skip(state))]
async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<SearchParams>>,
) -> axum::response::Result<Html<String>> {
    info!("search params: {:?}", params);
    match state.queryable.search(&params.q, params.limit, true).await {
        Ok(items) => {
            let page = SearchTemplate {
                query: params.q,
                items,
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("search failed".into()),
    }
}

#[derive(Deserialize, Validate, Debug)]
struct NextParams {
    #[validate(range(min = 1, max = 20000))]
    id: Option<u32>,
}

#[derive(Template, Debug)]
#[template(path = "now_and_next.html")]
struct NowAndNextTemplate {
    next: NextEvents,
}

#[tracing::instrument(skip(state))]
async fn next(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<NextParams>>,
) -> axum::response::Result<Html<String>> {
    let context = match params.id {
        Some(event_id) => NextEventsContext::EventId(event_id),
        None => NextEventsContext::Now,
    };
    match state.queryable.find_next_events(context).await {
        Ok(next) => {
            let page = NowAndNextTemplate { next };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("failed".into()),
    }
}

#[tracing::instrument(skip(state))]
async fn next_after_event(
    State(state): State<AppState>,
    Path(event_id): Path<u32>,
) -> axum::response::Result<Html<String>> {
    match state
        .queryable
        .find_next_events(NextEventsContext::Now)
        .await
    {
        Ok(next) => {
            let page = NowAndNextTemplate { next };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("failed".into()),
    }
}

pub async fn app_state(openai_api_key: &str, csv_data_dir: &std::path::Path) -> AppState {
    AppState {
        queryable: Arc::new(
            InMemoryOpenAIQueryable::connect(csv_data_dir, &openai_api_key)
                .await
                .unwrap(),
        ),
    }
}

pub async fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let router = Router::new()
        .route("/", get(index))
        .route("/search", get(search))
        .route("/connections/", get(related))
        .route("/next/", get(next))
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    router
}
