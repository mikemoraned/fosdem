use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Query, State},
    http::Method,
    response::Html,
    routing::get,
    Router,
};
use axum_valid::Valid;
use serde::Deserialize;
use shared::queryable::{Queryable, SearchItem};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use validator::Validate;

use crate::related::related;
use crate::state::AppState;

#[derive(Deserialize, Validate, Debug)]
struct Params {
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

mod filters {
    pub fn distance_similarity(distance: &f64) -> ::askama::Result<String> {
        let similarity = 1.0 - distance;
        Ok(format!("{:.2}", similarity).into())
    }

    pub fn distance_icon(distance: &f64) -> ::askama::Result<String> {
        let similarity = 1.0 - distance;
        Ok((if similarity <= 0.20 {
            "fa-thin fa-circle"
        } else if similarity <= 0.40 {
            "fa-duotone fa-circle"
        } else {
            "fa-solid fa-circle"
        })
        .into())
    }
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

#[tracing::instrument]
async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<Params>>,
) -> axum::response::Result<Html<String>> {
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

pub async fn router(openai_api_key: &str, db_host: &str, db_key: &str) -> Router {
    let state = AppState {
        queryable: Arc::new(
            Queryable::connect(&db_host, &db_key, &openai_api_key)
                .await
                .unwrap(),
        ),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let router = Router::new()
        .route("/", get(index))
        .route("/search", get(search))
        .route("/related/", get(related))
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    router
}
