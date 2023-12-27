use std::{path::PathBuf, sync::Arc};

use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use axum_valid::Valid;
use serde::Deserialize;
use shared::queryable::{Entry, Queryable};
use shuttle_secrets::SecretStore;
use tower_http::services::ServeDir;
use validator::Validate;

#[derive(Clone, Debug)]
struct AppState {
    queryable: Arc<Queryable>,
}

#[derive(Deserialize, Validate, Debug)]
struct Params {
    #[validate(length(min = 3, max = 20))]
    q: String,
    #[validate(range(min = 1, max = 20))]
    limit: u8,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    entries: Vec<Entry>,
}

async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<Params>>,
) -> axum::response::Result<Html<String>> {
    match state.queryable.find(&params.q, params.limit).await {
        Ok(entries) => {
            let page = SearchTemplate { entries };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("search failed".into()),
    }
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let openai_api_key = secret_store.get("OPENAI_API_KEY").unwrap();
    let db_host = secret_store.get("DB_HOST").unwrap();
    let db_key = secret_store.get("DB_KEY").unwrap();

    let state = AppState {
        queryable: Arc::new(
            Queryable::connect(&db_host, &db_key, &openai_api_key)
                .await
                .unwrap(),
        ),
    };

    let router = Router::new()
        .route("/search", get(search))
        .nest_service("/", ServeDir::new(PathBuf::from("html")))
        .with_state(state);

    Ok(router.into())
}
