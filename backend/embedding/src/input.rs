use shared::model::{Event, EventId};

pub struct InputBuilder {
    event_id: EventId,
    event: Option<Event>,
}

impl InputBuilder {
    pub fn new(event_id: EventId) -> InputBuilder {
        InputBuilder {
            event_id,
            event: None,
        }
    }

    pub fn format(&self, _max_tokens: u32) -> String {
        if let Some(e) = &self.event {
            format_basic_input(e)
        } else {
            "".into()
        }
    }

    pub fn with_event_source(self, event: &Event) -> Self {
        InputBuilder {
            event: Some(event.clone()),
            ..self
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::{NaiveDate, NaiveTime};
    use shared::model::{Event, EventId, Person};
    use url::Url;

    use crate::input::InputBuilder;

    #[test]
    fn test_basic_input() {
        let event = Event {
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
        };
        let builder = InputBuilder::new(EventId(1)).with_event_source(&event);
        let max_tokens = 10000;
        let actual = builder.format(max_tokens);
        let expected = "FOSDEM Conference Event 2024\n\
                              Title: Title 1\n\
                              Track: Track 1\n\
                              Abstract: Abstract 1\n\
                              Presenter: Person 1";
        assert_eq!(expected, actual);
    }
}

pub fn format_basic_input(event: &Event) -> String {
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

pub fn trim_input(input: &str) -> String {
    use tiktoken_rs::cl100k_base;
    let max_tokens = 8192 - 100;
    let token_estimator = cl100k_base().unwrap();

    let tokens = token_estimator.split_by_token(input, false).unwrap();
    let trimmed: Vec<_> = tokens.into_iter().take(max_tokens).collect();
    trimmed.join("")
}
