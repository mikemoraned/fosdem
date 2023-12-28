use shuttle_secrets::SecretStore;
use webapp::router::router;

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let openai_api_key = secret_store.get("OPENAI_API_KEY").unwrap();
    let db_host = secret_store.get("DB_HOST").unwrap();
    let db_key = secret_store.get("DB_KEY").unwrap();

    let router = router(&openai_api_key, &db_host, &db_key).await;

    Ok(router.into())
}
