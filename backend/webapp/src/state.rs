use shared::queryable::Queryable;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub queryable: Arc<Queryable>,
}
