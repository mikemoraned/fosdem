use std::{path::Path, sync::Arc};

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
use shared::{
    inmemory_openai::InMemoryOpenAIQueryable, postgres_openai::PostgresOpenAIQueryable,
    queryable::SearchItem,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;
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
        let assumed_max_typical_similarity = 0.60;
        let opacity = (similarity / assumed_max_typical_similarity).min(1.0f64);
        Ok(format!(
            "<i class=\"fa-solid fa-circle\" style=\"opacity: {}\"></i>",
            opacity
        )
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

#[tracing::instrument(skip(state))]
async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<Params>>,
) -> axum::response::Result<Html<String>> {
    use shared::queryable::Queryable;
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

// pub async fn app_state(openai_api_key: &str, db_host: &str, db_key: &str) -> AppState {
//     AppState {
//         queryable: Arc::new(
//             PostgresOpenAIQueryable::connect(&db_host, &db_key, &openai_api_key)
//                 .await
//                 .unwrap(),
//         ),
//     }
// }

pub async fn app_state(openai_api_key: &str, csv_data_dir: &Path) -> AppState {
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
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    router
}
