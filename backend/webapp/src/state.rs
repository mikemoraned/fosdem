use shared::postgres_openai::PostgresOpenAIQueryable;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub queryable: Arc<PostgresOpenAIQueryable>,
}
