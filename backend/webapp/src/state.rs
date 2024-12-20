use std::sync::Arc;

use content::video_index::VideoIndex;
use shared::inmemory_openai::InMemoryOpenAIQueryable;

#[derive(Clone, Debug)]
pub struct AppState {
    pub queryable: Arc<InMemoryOpenAIQueryable>,
    pub video_index: Arc<VideoIndex>,
}
