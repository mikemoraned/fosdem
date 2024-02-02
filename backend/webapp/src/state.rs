use std::sync::Arc;

use shared::inmemory_openai::InMemoryOpenAIQueryable;

#[derive(Clone, Debug)]
pub struct AppState {
    pub queryable: Arc<InMemoryOpenAIQueryable>,
}
