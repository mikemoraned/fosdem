use askama::Template;
use axum::{extract::State, response::Html};

use planning::Timetable;
use shared::queryable::Queryable;

use crate::state::AppState;

#[derive(Template, Debug)]
#[template(path = "timetables.html")]
struct TimetablesTemplate {
    timetables: Vec<Timetable>,
    current_fosdem: shared::model::CurrentFosdem,
}

#[tracing::instrument(skip(state))]
pub async fn timetables(State(state): State<AppState>) -> axum::response::Result<Html<String>> {
    let all_events = state.queryable.load_all_events().await.unwrap();

    // Filter events for the current FOSDEM year
    let current_year = state.current_fosdem.year;
    let events_for_year: Vec<_> = all_events
        .into_iter()
        .filter(|e| e.year == current_year)
        .collect();

    // Allocate events into timetables
    let timetables = planning::allocate(&events_for_year)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let page = TimetablesTemplate {
        timetables,
        current_fosdem: state.current_fosdem.clone(),
    };
    let html = page.render().unwrap();
    Ok(Html(html))
}
