use askama::Template;
use axum::{extract::State, response::Html};
use chrono::{Datelike, NaiveDate, NaiveTime};
use std::collections::{HashMap, HashSet};

use shared::{model::Event, queryable::Queryable};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct Period {
    pub duration_minutes: u32,
}

impl Period {
    pub fn slot_count(&self, slot_duration_minutes: &u32) -> usize {
        ((self.duration_minutes + slot_duration_minutes - 1) / slot_duration_minutes) as usize
    }
}

#[derive(Debug, Clone)]
pub struct TimetableCell {
    pub event: Event,
    pub rowspan: usize,
}

#[derive(Debug, Clone)]
pub struct TimetableRow {
    pub time_slot: NaiveTime,
    pub cells: Vec<Option<TimetableCell>>,
}

#[derive(Debug)]
pub struct DayTimetable {
    pub date: NaiveDate,
    pub day_name: String,
    pub rooms: Vec<String>,
    pub rows: Vec<TimetableRow>,
}

#[derive(Template, Debug)]
#[template(path = "bookmarks2.html")]
struct Bookmarks2Template {
    timetables: Vec<DayTimetable>,
    current_event: Option<Event>,
    current_fosdem: shared::model::CurrentFosdem,
}

#[tracing::instrument(skip(state))]
pub async fn bookmarks2(State(state): State<AppState>) -> axum::response::Result<Html<String>> {
    let mut events = state.queryable.load_all_events().await.unwrap();

    // Filter to current year only
    let current_year = state.current_fosdem.year;
    events.retain(|e| e.year == current_year);

    events.sort_by_key(|e| e.starting_time());

    // Build timetables
    let timetables = build_timetables(events);

    let page = Bookmarks2Template {
        timetables,
        current_event: None,
        current_fosdem: state.current_fosdem.clone(),
    };
    let html = page.render().unwrap();
    Ok(Html(html))
}

fn build_timetables(events: Vec<Event>) -> Vec<DayTimetable> {
    // Group events by date
    let mut events_by_date: HashMap<NaiveDate, Vec<Event>> = HashMap::new();
    for event in events {
        events_by_date
            .entry(event.date)
            .or_default()
            .push(event);
    }

    // Build a timetable for each day
    let mut timetables = Vec::new();
    let mut dates: Vec<_> = events_by_date.keys().copied().collect();
    dates.sort();

    for date in dates {
        let day_events = events_by_date.get(&date).unwrap();
        let timetable = build_day_timetable(date, day_events);
        timetables.push(timetable);
    }

    timetables
}

fn build_day_timetable(date: NaiveDate, events: &[Event]) -> DayTimetable {
    let slot_duration_minutes = 5;

    // Collect all unique rooms
    let rooms: HashSet<String> = events.iter().map(|e| e.room.clone()).collect();
    let mut rooms: Vec<String> = rooms.into_iter().collect();
    rooms.sort();

    // Generate time slots (5-minute intervals)
    let mut time_slots = Vec::new();
    let mut cell_map = HashMap::new();

    if !events.is_empty() {
        // Find earliest and latest times
        let earliest = events.iter().map(|e| e.start).min().unwrap();
        let latest = events.iter().map(|e| e.ending_time().time()).max().unwrap();

        let mut current_time = earliest;
        while current_time <= latest {
            time_slots.push(current_time);
            current_time = current_time
                .overflowing_add_signed(chrono::Duration::minutes(slot_duration_minutes as i64))
                .0;
        }

        // Place events in cells
        for event in events {
            let period = Period {
                duration_minutes: event.duration,
            };

            let rowspan = period.slot_count(&slot_duration_minutes);

            cell_map.insert(
                (event.start, event.room.clone()),
                TimetableCell {
                    event: event.clone(),
                    rowspan,
                },
            );
        }
    }

    // Build rows
    let mut rows = Vec::new();
    for time_slot in time_slots {
        let mut cells = Vec::new();
        for room in &rooms {
            let cell = cell_map.get(&(time_slot, room.clone())).cloned();
            cells.push(cell);
        }
        rows.push(TimetableRow { time_slot, cells });
    }

    let day_name = match date.weekday() {
        chrono::Weekday::Sat => "Saturday".to_string(),
        chrono::Weekday::Sun => "Sunday".to_string(),
        _ => date.format("%A").to_string(),
    };

    DayTimetable {
        date,
        day_name,
        rooms,
        rows,
    }
}
