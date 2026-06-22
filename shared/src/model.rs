use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use nalgebra::DVector;
use openai_dive::v1::resources::embedding::{EmbeddingOutput, EmbeddingResponse};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Clone)]
pub struct CurrentFosdem {
    pub year: u32,
    pub selectable_years: Vec<u32>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub event: Event,
    pub distance: f64,
    pub related: Option<Vec<SearchItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Hash, Copy)]
pub struct EventId {
    year: u32,
    id: u32,
}

impl EventId {
    pub const fn new(year: u32, id: u32) -> EventId {
        EventId { year, id }
    }

    pub const fn year(&self) -> u32 {
        self.year
    }

    pub const fn event_in_year(&self) -> u32 {
        self.id
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.year, self.id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Event {
    pub id: EventId,
    pub guid: String,
    pub year: u32,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub duration: u32,
    pub room: String,
    pub track: String,
    pub title: String,
    pub slug: String,
    pub url: Url,
    pub r#abstract: String,
    pub slides: Vec<Url>,
    pub presenters: Vec<Person>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Hash, Copy)]
pub struct PersonId {
    year: u32,
    id: u32,
}

impl PersonId {
    pub fn new(year: u32, id: u32) -> PersonId {
        PersonId { year, id }
    }
}

impl Display for PersonId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.year, self.id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Link {
    pub url: Url,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum VideoLink {
    Mp4(Url),
    Webm(Url),
}

impl VideoLink {
    pub fn url(&self) -> &Url {
        match self {
            VideoLink::Mp4(url) => url,
            VideoLink::Webm(url) => url,
        }
    }

    pub fn mime_type(&self) -> &str {
        match self {
            VideoLink::Mp4(_) => "video/mp4",
            VideoLink::Webm(_) => "video/webm",
        }
    }

    pub fn codecs(&self) -> Option<&str> {
        match self {
            VideoLink::Mp4(_) => None,
            VideoLink::Webm(_) => Some("av01.0.08M.08.0.110.01.01.01.0"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RoomId(String); // TODO: use RoomId instead of String in Event

impl RoomId {
    pub fn new(id: String) -> RoomId {
        RoomId(id)
    }

    pub fn nav_url(&self) -> Url {
        let location_base_url = Url::parse("https://nav.fosdem.org/l/").unwrap();
        location_base_url.join(&self.nav_room()).unwrap()
    }

    pub fn nav_room(&self) -> String {
        if self.0.contains(' ') {
            let parts: Vec<_> = self.0.split(' ').collect();
            let start = parts[0];
            start.to_lowercase().replace('.', "")
        } else {
            self.0.to_lowercase().replace('.', "")
        }
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Event {
    pub fn starting_time(&self) -> NaiveDateTime {
        self.date.and_time(self.start)
    }

    pub fn ending_time(&self) -> NaiveDateTime {
        self.starting_time() + Duration::minutes(self.duration.into())
    }

    pub fn sojourner_url(&self) -> Url {
        let base_url =
            Url::parse(format!("https://fosdem.sojourner.rocks/{}/event/", self.year).as_str())
                .unwrap();
        base_url.join(&self.guid.to_string()).unwrap()
    }

    pub fn nav_url(&self, current_event: &Option<Event>) -> Url {
        match current_event {
            Some(event) => Url::parse(&format!(
                "https://nav.fosdem.org/r/{}/{}",
                event.nav_room(),
                self.nav_room()
            ))
            .unwrap(),
            None => {
                let location_base_url = Url::parse("https://nav.fosdem.org/l/").unwrap();
                location_base_url.join(&self.nav_room()).unwrap()
            }
        }
    }

    pub fn nav_room(&self) -> String {
        if self.room.contains(' ') {
            let parts: Vec<_> = self.room.split(' ').collect();
            let start = parts[0];
            start.to_lowercase().replace('.', "")
        } else {
            self.room.to_lowercase().replace('.', "")
        }
    }

    fn find_video_url(&self, extension: &str) -> Option<Url> {
        self.links
            .iter()
            .filter(|l| {
                l.name.to_lowercase().contains("video recording")
                    && l.url.to_string().ends_with(extension)
            })
            .map(|l| l.url.clone())
            .next()
    }

    pub fn mp4_video_link(&self) -> Option<VideoLink> {
        self.find_video_url(".mp4").map(VideoLink::Mp4)
    }

    pub fn webm_video_link(&self) -> Option<VideoLink> {
        self.find_video_url(".webm").map(VideoLink::Webm)
    }

    pub fn video_links(&self) -> Vec<VideoLink> {
        let mut links = Vec::new();
        if let Some(webm) = self.webm_video_link() {
            links.push(webm);
        }
        if let Some(mp4) = self.mp4_video_link() {
            links.push(mp4);
        }
        links
    }

    pub fn has_video(&self) -> bool {
        !self.video_links().is_empty()
    }
}

pub type OpenAIVector = DVector<f64>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIEmbedding {
    pub title: String,
    pub embedding: OpenAIVector,
}

impl OpenAIEmbedding {
    pub fn embedding_from_response(
        response: &EmbeddingResponse,
    ) -> Result<OpenAIVector, Box<dyn std::error::Error>> {
        let output = response.data[0].embedding.clone();
        match output {
            EmbeddingOutput::Float(parts) => Ok(OpenAIVector::from(parts)),
            EmbeddingOutput::Base64(base64) => {
                Err(format!("Base64 encoding not supported: {}", base64).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_event_with_links(links: Vec<Link>) -> Event {
        Event {
            id: EventId::new(2024, 1),
            guid: "guid".to_string(),
            year: 2024,
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            duration: 30,
            room: "Room".to_string(),
            track: "Track".to_string(),
            title: "Title".to_string(),
            slug: "slug".to_string(),
            url: "https://example.com".parse().unwrap(),
            r#abstract: "Abstract".to_string(),
            slides: vec![],
            presenters: vec![],
            links,
        }
    }

    #[test]
    fn test_mp4_video_link_exact_name_match() {
        let name = "Video recording";
        let event = make_event_with_links(vec![Link {
            name: name.to_string(),
            url: "https://video.fosdem.org/2024/test.mp4".parse().unwrap(),
        }]);

        assert!(event.mp4_video_link().is_some(), "name: {:?}", name);
    }

    #[test]
    fn test_mp4_video_link_partial_name_match() {
        let names = [
            "Video recording (MP4) - 357.1 MB",
            "Video recording (MP4; for legacy systems) - 994.0 MB",
        ];
        for name in names.iter() {
            let event = make_event_with_links(vec![Link {
                name: name.to_string(),
                url: "https://video.fosdem.org/2024/test.mp4".parse().unwrap(),
            }]);
            assert!(event.mp4_video_link().is_some(), "name: {:?}", name);
        }
    }

    #[test]
    fn test_mp4_video_link_no_match() {
        let event = make_event_with_links(vec![Link {
            name: "Some other link".to_string(),
            url: "https://example.com".parse().unwrap(),
        }]);

        assert!(event.mp4_video_link().is_none(), "event: {:?}", event);
    }

    #[test]
    fn test_mp4_video_link_wrong_extension() {
        let event = make_event_with_links(vec![Link {
            name: "Video recording".to_string(),
            url: "https://video.fosdem.org/2024/test.webm".parse().unwrap(),
        }]);

        assert!(event.mp4_video_link().is_none(), "event: {:?}", event);
    }

    #[test]
    fn test_webm_video_link_exact_name_match() {
        let name = "Video recording";
        let event = make_event_with_links(vec![Link {
            name: name.to_string(),
            url: "https://video.fosdem.org/2024/test.webm".parse().unwrap(),
        }]);

        assert!(event.webm_video_link().is_some(), "name: {:?}", name);
    }

    #[test]
    fn test_webm_video_link_partial_name_match() {
        let names = ["Video recording (AV1/opus)", "Video recording (webm)"];
        for name in names.iter() {
            let event = make_event_with_links(vec![Link {
                name: name.to_string(),
                url: "https://video.fosdem.org/2024/test.webm".parse().unwrap(),
            }]);
            assert!(event.webm_video_link().is_some(), "name: {:?}", name);
        }
    }

    #[test]
    fn test_webm_video_link_no_match() {
        let event = make_event_with_links(vec![Link {
            name: "Some other link".to_string(),
            url: "https://example.com".parse().unwrap(),
        }]);

        assert!(event.webm_video_link().is_none(), "event: {:?}", event);
    }

    #[test]
    fn test_webm_video_link_wrong_extension() {
        let event = make_event_with_links(vec![Link {
            name: "Video recording".to_string(),
            url: "https://video.fosdem.org/2024/test.mp4".parse().unwrap(),
        }]);

        assert!(event.webm_video_link().is_none(), "event: {:?}", event);
    }

    #[test]
    fn test_video_links_returns_both_formats_webm_first() {
        let event = make_event_with_links(vec![
            Link {
                name: "Video recording (mp4)".to_string(),
                url: "https://video.fosdem.org/2024/test.mp4".parse().unwrap(),
            },
            Link {
                name: "Video recording (AV1/opus)".to_string(),
                url: "https://video.fosdem.org/2024/test.webm".parse().unwrap(),
            },
        ]);

        let links = event.video_links();
        assert_eq!(links.len(), 2);
        assert!(matches!(links[0], VideoLink::Webm(_)));
        assert!(matches!(links[1], VideoLink::Mp4(_)));
    }

    #[test]
    fn test_video_links_returns_mp4_only() {
        let event = make_event_with_links(vec![Link {
            name: "Video recording (mp4)".to_string(),
            url: "https://video.fosdem.org/2024/test.mp4".parse().unwrap(),
        }]);

        let links = event.video_links();
        assert_eq!(links.len(), 1);
        assert!(matches!(links[0], VideoLink::Mp4(_)));
    }

    #[test]
    fn test_video_links_returns_webm_only() {
        let event = make_event_with_links(vec![Link {
            name: "Video recording (AV1/opus)".to_string(),
            url: "https://video.fosdem.org/2024/test.webm".parse().unwrap(),
        }]);

        let links = event.video_links();
        assert_eq!(links.len(), 1);
        assert!(matches!(links[0], VideoLink::Webm(_)));
    }

    #[test]
    fn test_video_links_returns_empty_when_no_videos() {
        let event = make_event_with_links(vec![Link {
            name: "Some other link".to_string(),
            url: "https://example.com".parse().unwrap(),
        }]);

        let links = event.video_links();
        assert!(links.is_empty());
    }
}
