use shared::{inmemory_openai::InMemoryOpenAIQueryable, postgres_openai::PostgresOpenAIQueryable};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    // pub queryable: Arc<PostgresOpenAIQueryable>,
    pub queryable: Arc<InMemoryOpenAIQueryable>,
}
