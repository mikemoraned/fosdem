use std::collections::{BTreeMap, HashSet};

use crate::{model::PersonId, queryable::Queryable};

#[derive(Debug)]
pub struct Summary {
    pub events: usize,
    pub people: usize,
    pub rooms: usize,
    pub tracks: usize,
}

#[derive(Debug)]
pub struct DataSummary {
    pub by_year: BTreeMap<u32, Summary>,
}

pub async fn load_summary<Q: Queryable>(
    queryable: &Q,
) -> Result<DataSummary, Box<dyn std::error::Error>> {
    let events = queryable.load_all_events().await?;

    let mut events_by_year: BTreeMap<u32, usize> = BTreeMap::new();
    let mut people_by_year: BTreeMap<u32, HashSet<PersonId>> = BTreeMap::new();
    let mut rooms_by_year: BTreeMap<u32, HashSet<String>> = BTreeMap::new();
    let mut tracks_by_year: BTreeMap<u32, HashSet<String>> = BTreeMap::new();

    for event in events {
        let year = event.year;
        *events_by_year.entry(year).or_insert(0) += 1;

        rooms_by_year
            .entry(year)
            .or_default()
            .insert(event.room.clone());

        tracks_by_year
            .entry(year)
            .or_default()
            .insert(event.track.clone());

        for person in &event.presenters {
            people_by_year.entry(year).or_default().insert(person.id);
        }
    }

    let mut by_year = BTreeMap::new();
    for (year, events) in &events_by_year {
        by_year.insert(
            *year,
            Summary {
                events: *events,
                people: people_by_year.get(year).map_or(0, |s| s.len()),
                rooms: rooms_by_year.get(year).map_or(0, |s| s.len()),
                tracks: tracks_by_year.get(year).map_or(0, |s| s.len()),
            },
        );
    }

    Ok(DataSummary { by_year })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Event, EventId, Person, PersonId};
    use chrono::NaiveDate;

    struct TestQueryable {
        events: Vec<Event>,
    }

    impl crate::queryable::Queryable for TestQueryable {
        async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
            Ok(self.events.clone())
        }

        async fn find_event_by_id(
            &self,
            _: EventId,
        ) -> Result<Option<Event>, Box<dyn std::error::Error>> {
            unimplemented!()
        }

        async fn find_related_events(
            &self,
            _: &str,
            _: u8,
            _: Option<u32>,
        ) -> Result<Vec<crate::model::SearchItem>, Box<dyn std::error::Error>> {
            unimplemented!()
        }

        async fn search(
            &self,
            _: &str,
            _: u8,
            _: bool,
            _: Option<u32>,
        ) -> Result<Vec<crate::model::SearchItem>, Box<dyn std::error::Error>> {
            unimplemented!()
        }

        async fn find_next_events(
            &self,
            _: crate::model::NextEventsContext,
        ) -> Result<crate::model::NextEvents, Box<dyn std::error::Error>> {
            unimplemented!()
        }
    }

    fn make_event(year: u32, room: &str, track: &str, presenters: Vec<PersonId>) -> Event {
        Event {
            id: EventId::new(year, 1),
            guid: "guid".to_string(),
            year,
            date: NaiveDate::from_ymd_opt(year as i32, 1, 1).unwrap(),
            start: chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            duration: 30,
            room: room.to_string(),
            track: track.to_string(),
            title: "Title".to_string(),
            slug: "slug".to_string(),
            url: "https://example.com".parse().unwrap(),
            r#abstract: "Abstract".to_string(),
            slides: vec![],
            presenters: presenters
                .into_iter()
                .map(|id| Person {
                    id,
                    name: "Name".to_string(),
                })
                .collect(),
            links: vec![],
        }
    }

    #[tokio::test]
    async fn test_empty_summary() {
        let queryable = TestQueryable { events: vec![] };
        let summary = load_summary(&queryable).await.unwrap();

        assert!(summary.by_year.is_empty());
    }

    #[tokio::test]
    async fn test_single_year_summary() {
        let queryable = TestQueryable {
            events: vec![
                make_event(2024, "Room1", "Track1", vec![PersonId::new(2024, 1)]),
                make_event(2024, "Room2", "Track1", vec![PersonId::new(2024, 2)]),
            ],
        };
        let summary = load_summary(&queryable).await.unwrap();

        assert_eq!(summary.by_year.len(), 1);
        let year_2024 = summary.by_year.get(&2024).unwrap();
        assert_eq!(year_2024.events, 2);
        assert_eq!(year_2024.people, 2);
        assert_eq!(year_2024.rooms, 2);
        assert_eq!(year_2024.tracks, 1);
    }

    #[tokio::test]
    async fn test_multi_year_summary() {
        let queryable = TestQueryable {
            events: vec![
                make_event(2024, "Room1", "Track1", vec![PersonId::new(2024, 1)]),
                make_event(2025, "Room1", "Track2", vec![PersonId::new(2025, 1)]),
            ],
        };
        let summary = load_summary(&queryable).await.unwrap();

        assert_eq!(summary.by_year.len(), 2);
        assert_eq!(summary.by_year.get(&2024).unwrap().events, 1);
        assert_eq!(summary.by_year.get(&2025).unwrap().events, 1);
    }
}
