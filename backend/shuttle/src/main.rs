use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use shared::queryable::{Entry, Queryable};
use shuttle_secrets::SecretStore;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[derive(Clone, Debug)]
struct AppState {
    queryable: Arc<Queryable>,
}

#[derive(Deserialize, Debug)]
struct Params {
    q: String,
    limit: u8,
}

async fn search(State(state): State<AppState>, params: Query<Params>) -> Json<Vec<Entry>> {
    Json(state.queryable.find(&params.q, params.limit).await.unwrap())
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
        .route("/", get(hello_world))
        .route("/search", get(search))
        .with_state(state);

    Ok(router.into())
}
