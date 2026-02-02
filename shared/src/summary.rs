use std::collections::{BTreeMap, HashSet};

use chrono::Duration;

use crate::{model::PersonId, queryable::Queryable};

#[derive(Debug)]
pub struct Summary {
    pub events: usize,
    pub people: usize,
    pub rooms: usize,
    pub tracks: usize,
    pub videos: usize,
    pub video_duration: Duration,
    pub slides: usize,
    pub links: usize,
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
    let mut videos_by_year: BTreeMap<u32, usize> = BTreeMap::new();
    let mut video_duration_by_year: BTreeMap<u32, Duration> = BTreeMap::new();
    let mut slides_by_year: BTreeMap<u32, usize> = BTreeMap::new();
    let mut links_by_year: BTreeMap<u32, usize> = BTreeMap::new();

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

        if event.mp4_video_link().is_some() {
            *videos_by_year.entry(year).or_insert(0) += 1;
            *video_duration_by_year.entry(year).or_insert(Duration::zero()) +=
                Duration::minutes(event.duration.into());
        }

        *slides_by_year.entry(year).or_insert(0) += event.slides.len();
        *links_by_year.entry(year).or_insert(0) += event.links.len();
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
                videos: *videos_by_year.get(year).unwrap_or(&0),
                video_duration: *video_duration_by_year
                    .get(year)
                    .unwrap_or(&Duration::zero()),
                slides: *slides_by_year.get(year).unwrap_or(&0),
                links: *links_by_year.get(year).unwrap_or(&0),
            },
        );
    }

    Ok(DataSummary { by_year })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Event, EventId, Link, Person, PersonId};
    use chrono::NaiveDate;
    use url::Url;

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

    fn make_event(
        year: u32,
        room: &str,
        track: &str,
        presenters: Vec<PersonId>,
        slides: Vec<Url>,
        links: Vec<Link>,
    ) -> Event {
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
            slides,
            presenters: presenters
                .into_iter()
                .map(|id| Person {
                    id,
                    name: "Name".to_string(),
                })
                .collect(),
            links,
        }
    }

    fn make_video_link() -> Link {
        Link {
            name: "Video recording (mp4)".to_string(),
            url: "https://video.fosdem.org/2024/test.mp4".parse().unwrap(),
        }
    }

    fn make_slide() -> Url {
        "https://fosdem.org/slides/test.pdf".parse().unwrap()
    }

    fn make_link() -> Link {
        Link {
            name: "Some link".to_string(),
            url: "https://example.com".parse().unwrap(),
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
                make_event(2024, "Room1", "Track1", vec![PersonId::new(2024, 1)], vec![], vec![]),
                make_event(2024, "Room2", "Track1", vec![PersonId::new(2024, 2)], vec![], vec![]),
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
                make_event(2024, "Room1", "Track1", vec![PersonId::new(2024, 1)], vec![], vec![]),
                make_event(2025, "Room1", "Track2", vec![PersonId::new(2025, 1)], vec![], vec![]),
            ],
        };
        let summary = load_summary(&queryable).await.unwrap();

        assert_eq!(summary.by_year.len(), 2);
        assert_eq!(summary.by_year.get(&2024).unwrap().events, 1);
        assert_eq!(summary.by_year.get(&2025).unwrap().events, 1);
    }

    #[tokio::test]
    async fn test_videos_slides_links_counts() {
        let queryable = TestQueryable {
            events: vec![
                // Event with video, 2 slides, 3 links (including video link)
                make_event(
                    2024,
                    "Room1",
                    "Track1",
                    vec![],
                    vec![make_slide(), make_slide()],
                    vec![make_video_link(), make_link(), make_link()],
                ),
                // Event with no video, 1 slide, 1 link
                make_event(
                    2024,
                    "Room1",
                    "Track1",
                    vec![],
                    vec![make_slide()],
                    vec![make_link()],
                ),
            ],
        };
        let summary = load_summary(&queryable).await.unwrap();

        let year_2024 = summary.by_year.get(&2024).unwrap();
        assert_eq!(year_2024.videos, 1); // Only first event has video
        assert_eq!(year_2024.video_duration, Duration::minutes(30)); // 30 min event duration
        assert_eq!(year_2024.slides, 3); // 2 + 1
        assert_eq!(year_2024.links, 4);  // 3 + 1
    }

    #[tokio::test]
    async fn test_no_videos_slides_links() {
        let queryable = TestQueryable {
            events: vec![make_event(2024, "Room1", "Track1", vec![], vec![], vec![])],
        };
        let summary = load_summary(&queryable).await.unwrap();

        let year_2024 = summary.by_year.get(&2024).unwrap();
        assert_eq!(year_2024.videos, 0);
        assert_eq!(year_2024.video_duration, Duration::zero());
        assert_eq!(year_2024.slides, 0);
        assert_eq!(year_2024.links, 0);
    }
}
