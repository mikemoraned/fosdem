
use askama::Template;
use axum::{
    extract::State,
    response::Html,
};
use axum::extract::Path;
use serde::Deserialize;
use shared::{model::Event, queryable::Queryable};
use shared::model::RoomId;
use crate::state::AppState;

#[derive(Template, Debug)]
#[template(path = "room.html")]
struct RoomTemplate {
    room: RoomId,
    events: Vec<Event>,
    current_event: Option<Event>, // TODO: remove this
}

#[derive(Deserialize, Debug)]
pub struct RoomIdParam(String);

#[tracing::instrument(skip(state))]
pub async fn room(
    State(state): State<AppState>,
    Path(RoomIdParam(room_id)): Path<RoomIdParam>,
) -> axum::response::Result<Html<String>> {
    let all_events = state.queryable.load_all_events().await.unwrap();
    let mut events: Vec<Event> = all_events.into_iter().filter(|e| e.room == room_id).collect();
    events.sort_by_key(|e| e.starting_time());
    let room = RoomId::new(room_id);
    let page = RoomTemplate {
        room,
        events,
        current_event: None,
    };
    let html = page.render().unwrap();
    Ok(Html(html))
}
