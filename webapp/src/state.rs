use std::sync::Arc;

use blog::BlogIndex;
use content::video_index::VideoIndex;
use shared::{inmemory_openai::InMemoryOpenAIQueryable, model::CurrentFosdem};

#[derive(Clone, Debug)]
pub struct AppState {
    pub queryable: Arc<InMemoryOpenAIQueryable>,
    pub video_index: Arc<VideoIndex>,
    pub current_fosdem: CurrentFosdem,
    pub blog_index: Arc<BlogIndex>,
}
