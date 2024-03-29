use content::{slide_index::SlideIndex, video_index::VideoIndex};
use shared::model::{Event, EventId};
use subtp::vtt::{VttBlock, WebVtt};

pub struct InputBuilder {
    event_id: EventId,
    event: Option<Event>,
    slide_content: Option<String>,
    video_content: Option<WebVtt>,
}

#[derive(Debug, PartialEq)]
pub struct FormatStatistics {
    used_character_count: usize,
    discarded_character_count: usize,
}
impl FormatStatistics {
    pub fn accumulate(&mut self, statistics: FormatStatistics) {
        self.used_character_count += statistics.used_character_count;
        self.discarded_character_count += statistics.discarded_character_count;
    }
}

impl Default for FormatStatistics {
    fn default() -> Self {
        Self {
            used_character_count: 0,
            discarded_character_count: 0,
        }
    }
}

impl InputBuilder {
    pub fn new(event_id: EventId) -> InputBuilder {
        InputBuilder {
            event_id,
            event: None,
            slide_content: None,
            video_content: None,
        }
    }

    pub fn with_event_source(self, event: &Event) -> Self {
        InputBuilder {
            event: Some(event.clone()),
            ..self
        }
    }

    pub fn with_slide_source(self, slide_index: &SlideIndex) -> Self {
        InputBuilder {
            slide_content: slide_index.entries.get(&self.event_id.0).cloned(),
            ..self
        }
    }

    pub fn with_video_source(self, video_index: &VideoIndex) -> Self {
        InputBuilder {
            video_content: video_index.webvtt_for_event_id(self.event_id.0),
            ..self
        }
    }

    pub fn format_with_statistics(
        &self,
        max_tokens: usize,
    ) -> Result<(String, FormatStatistics), Box<dyn std::error::Error>> {
        let mut preferred_input = String::new();
        use std::fmt::Write;

        if let Some(e) = &self.event {
            writeln!(preferred_input, "{}", format_basic_input(e))?;
        }

        if let Some(s) = &self.slide_content {
            writeln!(preferred_input, "Slides: {}", s)?;
        }

        if let Some(video_content) = &self.video_content {
            let mut block_content: Vec<_> = video_content
                .blocks
                .iter()
                .map(|b| match b {
                    VttBlock::Que(cue) => cue.payload.join("\n"),
                    _ => "".into(),
                })
                .collect();
            block_content.dedup();
            writeln!(preferred_input, "Subtitles: {}", block_content.join("\n"))?;
        }

        let (used, discarded) = split_input(&preferred_input, max_tokens);
        let statistics = FormatStatistics {
            used_character_count: used.len(),
            discarded_character_count: discarded.len(),
        };

        Ok((used, statistics))
    }

    pub fn format(&self, max_tokens: usize) -> Result<String, Box<dyn std::error::Error>> {
        let (formatted, _) = self.format_with_statistics(max_tokens)?;
        Ok(formatted)
    }
}

fn split_input(input: &str, max_tokens: usize) -> (String, String) {
    use tiktoken_rs::cl100k_base;
    let token_estimator = cl100k_base().unwrap();

    let tokens = token_estimator.split_by_token(input, false).unwrap();
    let head: Vec<String> = tokens.iter().take(max_tokens).map(|s| s.clone()).collect();
    let tail: Vec<String> = tokens.iter().skip(max_tokens).map(|s| s.clone()).collect();

    (head.join(""), tail.join(""))
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use chrono::{NaiveDate, NaiveTime};
    use content::{
        slide_index::SlideIndex,
        video_index::{VideoIndex, VideoIndexEntry},
    };
    use shared::model::{Event, EventId, Person, VideoFile};
    use subtp::vtt::{VttBlock, VttCue, VttHeader, VttTimestamp, VttTimings, WebVtt};
    use url::Url;

    use crate::input::{FormatStatistics, InputBuilder};

    use super::split_input;

    fn example_event() -> Event {
        Event {
            id: 1,
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            duration: 0,
            room: "Room 1".into(),
            track: "Track 1".into(),
            title: "Title 1".into(),
            slug: "slug1".into(),
            url: Url::parse("http://example.com/foop").unwrap(),
            r#abstract: "Abstract 1".into(),
            slides: vec![],
            presenters: vec![Person {
                id: 1,
                name: "Person 1".into(),
            }],
            links: vec![],
        }
    }

    fn example_slide_index() -> SlideIndex {
        let mut entries: HashMap<u32, String> = HashMap::new();
        entries.insert(1u32, "Slide Content 1".into());
        SlideIndex { entries }
    }

    fn example_video_index() -> VideoIndex {
        let mut entries: HashMap<u32, VideoIndexEntry> = HashMap::new();
        let webvtt = WebVtt {
            header: VttHeader { description: None },
            blocks: vec![VttBlock::Que(VttCue {
                identifier: None,
                timings: VttTimings {
                    start: VttTimestamp {
                        hours: 0,
                        minutes: 0,
                        seconds: 0,
                        milliseconds: 0,
                    },
                    end: VttTimestamp {
                        hours: 1,
                        minutes: 0,
                        seconds: 0,
                        milliseconds: 0,
                    },
                },
                settings: None,
                payload: vec!["Some Speech 1".into()],
            })],
        };
        let entry = VideoIndexEntry {
            webvtt,
            file: VideoFile {
                name: "video1".into(),
            },
        };
        entries.insert(1u32, entry);
        VideoIndex { entries }
    }

    #[test]
    fn test_basic_input() {
        let builder = InputBuilder::new(EventId(1)).with_event_source(&example_event());
        let max_tokens = 10000;
        let actual = builder.format(max_tokens).unwrap();
        let expected = "FOSDEM Conference Event 2024\n\
                              Title: Title 1\n\
                              Track: Track 1\n\
                              Abstract: Abstract 1\n\
                              Presenter: Person 1\n";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_slide_input() {
        let builder = InputBuilder::new(EventId(1)).with_slide_source(&example_slide_index());
        let max_tokens = 10000;
        let actual = builder.format(max_tokens).unwrap();
        let expected = "Slides: Slide Content 1\n";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_video_input() {
        let builder = InputBuilder::new(EventId(1)).with_video_source(&example_video_index());
        let max_tokens = 10000;
        let actual = builder.format(max_tokens).unwrap();
        let expected = "Subtitles: Some Speech 1\n";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_combined_input() {
        let builder = InputBuilder::new(EventId(1))
            .with_event_source(&example_event())
            .with_slide_source(&example_slide_index())
            .with_video_source(&example_video_index());
        let max_tokens = 10000;
        let actual = builder.format(max_tokens).unwrap();
        let expected = "FOSDEM Conference Event 2024\n\
                              Title: Title 1\n\
                              Track: Track 1\n\
                              Abstract: Abstract 1\n\
                              Presenter: Person 1\n\
                              Slides: Slide Content 1\n\
                              Subtitles: Some Speech 1\n";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_token_limits() {
        let builder = InputBuilder::new(EventId(1)).with_event_source(&example_event());
        let max_tokens = 8;
        let actual = builder.format(max_tokens).unwrap();
        let expected = "FOSDEM Conference Event 2024";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_format_with_statistics() {
        let builder = InputBuilder::new(EventId(1)).with_event_source(&example_event());
        let max_tokens = 8;
        let (actual_used, actual_statistics) = builder.format_with_statistics(max_tokens).unwrap();
        let expected_used = "FOSDEM Conference Event 2024";
        let expected_statistics = FormatStatistics {
            used_character_count: 28,
            discarded_character_count: 72,
        };
        assert_eq!(expected_used, actual_used);
        assert_eq!(expected_statistics, actual_statistics);
    }

    #[test]
    fn test_split_input() {
        let input = "some multiple amount of tokens";
        let max_tokens = 2;
        let (head, tail) = split_input(input, max_tokens);
        assert_eq!("some multiple", head);
        assert_eq!(" amount of tokens", tail);
    }
}

fn format_basic_input(event: &Event) -> String {
    let lines: Vec<String> = vec![
        "FOSDEM Conference Event 2024".into(),
        format!("Title: {}", event.title),
        format!("Track: {}", event.track),
        format!("Abstract: {}", event.r#abstract),
        format!(
            "Presenter: {}",
            event
                .presenters
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ];
    lines.join("\n")
}