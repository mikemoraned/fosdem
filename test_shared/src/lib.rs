use const_format::formatcp;
use shared::model::EventId;

// common theme here is a search on "gnome" which should find these related events in the different years

pub const SEARCH_TERM: &str = "gnome";

pub const EVENT_ID_2025: EventId = EventId::new(2025, 5649);
pub const EVENT_ID_2025_BACKWARDS_COMPATIBLE_PATH: &str =
    formatcp!("/event/{}/", EVENT_ID_2025.event_in_year());
pub const EVENT_ID_2025_CANONICAL_PATH: &str = formatcp!(
    "/{}/event/{}/",
    EVENT_ID_2025.year(),
    EVENT_ID_2025.event_in_year()
);
pub const EVENT_ID_2025_ABSTRACT_PATH: &str = formatcp!(
    "/{}/event/{}/abstract/",
    EVENT_ID_2025.year(),
    EVENT_ID_2025.event_in_year()
);
pub const EVENT_ID_2025_CARD_PATH: &str = formatcp!(
    "/{}/event/{}/card/",
    EVENT_ID_2025.year(),
    EVENT_ID_2025.event_in_year()
);
pub const EVENT_ID_2025_CONTENT_SAMPLE: &str =
    "The GNOME project has some end-to-end tests running with openQA and os-autoinst";

pub const EVENT_ID_2026: EventId = EventId::new(2026, 8816);
pub const EVENT_ID_2026_CANONICAL_PATH: &str = formatcp!(
    "/{}/event/{}/",
    EVENT_ID_2026.year(),
    EVENT_ID_2026.event_in_year()
);
pub const EVENT_ID_2026_ABSTRACT_PATH: &str = formatcp!(
    "/{}/event/{}/abstract/",
    EVENT_ID_2026.year(),
    EVENT_ID_2026.event_in_year()
);
pub const EVENT_ID_2026_CARD_PATH: &str = formatcp!(
    "/{}/event/{}/card/",
    EVENT_ID_2026.year(),
    EVENT_ID_2026.event_in_year()
);
pub const EVENT_ID_2026_CONTENT_SAMPLE: &str =
    "In this talk, I'm going to present recent efforts to run GNOME OS on phones";
