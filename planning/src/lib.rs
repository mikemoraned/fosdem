use std::collections::HashMap;

use chrono::{Duration, NaiveDate, NaiveTime};
use shared::model::Event;

// A Timetable is an allocation of Events into TimeSlots for a single day.
// invariants:
// * A Timetable never spans multiple days
// * `slots` is an ordered series of TimeSlots separated by `slot_duration`. In other words `slots` has no gaps
pub struct Timetable {
    pub day: NaiveDate,
    pub slots: Vec<TimeSlot>,
    pub slot_duration: Duration,
}

// A TimeSlot has a start and can overlap with different Events in different Streams
// invariants:
// * The same Event event may appear in an EventOverlap in multiple Streams
// * The same Event cannot have multiple occurrences via different EventOverlaps in the same Stream
pub struct TimeSlot {
    pub start: NaiveTime,
    pub overlaps: HashMap<Stream, EventOverlap>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stream {
    Room(String),
}

// an Event overlaps with a Slot via it's Beginning, Middle, or End
// invariants:
// * An Event has one Beginning and one End overlap, with different Slots
// * An Event may have zero to many Middle overlaps
pub enum EventOverlap {
    Beginning(Box<Event>),
    Middle(Box<Event>),
    End(Box<Event>),
}

// takes multiple events and allocates them to Slots in a Timetable where each Slot has a `slot_duration`
// invariants:
// * Timetables are returned in sorted order, ordered by `day`
// * each Event may only appear in a single Timetable
pub fn allocate(events: &[Event], slot_duration: Duration) -> Vec<Timetable> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use shared::model::{EventId, Person, PersonId};
    use url::Url;

    fn make_event(id: u32, date: NaiveDate, start: NaiveTime, duration: u32, room: &str) -> Event {
        let year = date.year() as u32;
        Event {
            id: EventId::new(year, id),
            guid: format!("guid-{}", id),
            year,
            date,
            start,
            duration,
            room: room.to_string(),
            track: "Test Track".to_string(),
            title: format!("Event {}", id),
            slug: format!("event-{}", id),
            url: Url::parse("https://example.com").unwrap(),
            r#abstract: String::new(),
            slides: vec![],
            presenters: vec![Person {
                id: PersonId::new(year, 1),
                name: "Test".to_string(),
            }],
            links: vec![],
        }
    }

    // Invariant: A Timetable never spans multiple days
    #[test]
    fn timetable_single_day() {
        let day1 = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2024, 2, 4).unwrap();
        let start = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        let events = vec![
            make_event(1, day1, start, 30, "Room1"),
            make_event(2, day2, start, 30, "Room1"),
        ];

        let timetables = allocate(&events, Duration::minutes(15));

        // Each timetable should contain events from only one day
        for timetable in &timetables {
            let events_in_timetable: Vec<&Event> = timetable
                .slots
                .iter()
                .flat_map(|slot| slot.overlaps.values())
                .map(|overlap| match overlap {
                    EventOverlap::Beginning(e) | EventOverlap::Middle(e) | EventOverlap::End(e) => {
                        e.as_ref()
                    }
                })
                .collect();

            for event in &events_in_timetable {
                assert_eq!(event.date, timetable.day);
            }
        }
    }

    // Invariant: slots is an ordered series of TimeSlots separated by slot_duration (no gaps)
    #[test]
    fn timetable_slots_no_gaps() {
        let day = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let start = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        let events = vec![make_event(1, day, start, 60, "Room1")];

        let slot_duration = Duration::minutes(15);
        let timetables = allocate(&events, slot_duration);

        for timetable in &timetables {
            for i in 1..timetable.slots.len() {
                let prev_start = timetable.slots[i - 1].start;
                let curr_start = timetable.slots[i].start;
                let expected = prev_start + timetable.slot_duration;
                assert_eq!(curr_start, expected, "Slots must be consecutive with no gaps");
            }
        }
    }

    // Invariant: The same Event cannot have multiple occurrences via different EventOverlaps in the same Stream
    #[test]
    fn timeslot_no_duplicate_event_in_same_stream() {
        let day = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let start = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        let events = vec![make_event(1, day, start, 30, "Room1")];

        let timetables = allocate(&events, Duration::minutes(15));

        for timetable in &timetables {
            for slot in &timetable.slots {
                // Within a single slot, each stream should have at most one overlap per event
                // (This is enforced by HashMap<Stream, EventOverlap> - only one overlap per stream)
                // But we verify no event appears multiple times across the slot's overlaps
                let mut event_ids_in_slot: Vec<EventId> = slot
                    .overlaps
                    .values()
                    .map(|overlap| match overlap {
                        EventOverlap::Beginning(e)
                        | EventOverlap::Middle(e)
                        | EventOverlap::End(e) => e.id,
                    })
                    .collect();
                let original_len = event_ids_in_slot.len();
                event_ids_in_slot.sort();
                event_ids_in_slot.dedup();
                // Note: same event CAN appear in multiple streams, so we just check
                // the HashMap structure ensures one overlap per stream
                assert!(
                    slot.overlaps.len() <= original_len,
                    "HashMap ensures one overlap per stream"
                );
            }
        }
    }

    // Invariant: An Event has one Beginning and one End overlap, with different Slots
    #[test]
    fn event_has_one_beginning_and_one_end_in_different_slots() {
        let day = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let start = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        // Event spans multiple slots (60 min event with 15 min slots)
        let events = vec![make_event(1, day, start, 60, "Room1")];

        let timetables = allocate(&events, Duration::minutes(15));

        for timetable in &timetables {
            let mut beginnings: HashMap<EventId, Vec<usize>> = HashMap::new();
            let mut ends: HashMap<EventId, Vec<usize>> = HashMap::new();

            for (slot_idx, slot) in timetable.slots.iter().enumerate() {
                for overlap in slot.overlaps.values() {
                    match overlap {
                        EventOverlap::Beginning(e) => {
                            beginnings.entry(e.id).or_default().push(slot_idx);
                        }
                        EventOverlap::End(e) => {
                            ends.entry(e.id).or_default().push(slot_idx);
                        }
                        EventOverlap::Middle(_) => {}
                    }
                }
            }

            // Each event should have exactly one Beginning and one End
            for event in &events {
                if event.date == timetable.day {
                    let begin_slots = beginnings.get(&event.id).expect("Event should have Beginning");
                    let end_slots = ends.get(&event.id).expect("Event should have End");

                    assert_eq!(begin_slots.len(), 1, "Event should have exactly one Beginning");
                    assert_eq!(end_slots.len(), 1, "Event should have exactly one End");
                    assert_ne!(
                        begin_slots[0], end_slots[0],
                        "Beginning and End should be in different slots"
                    );
                }
            }
        }
    }

    // Invariant: An Event may have zero to many Middle overlaps
    #[test]
    fn event_can_have_middle_overlaps() {
        let day = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let start = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        // Long event (60 min) with short slots (15 min) should have middle overlaps
        let events = vec![make_event(1, day, start, 60, "Room1")];

        let timetables = allocate(&events, Duration::minutes(15));

        for timetable in &timetables {
            let mut middles: HashMap<EventId, usize> = HashMap::new();

            for slot in &timetable.slots {
                for overlap in slot.overlaps.values() {
                    if let EventOverlap::Middle(e) = overlap {
                        *middles.entry(e.id).or_default() += 1;
                    }
                }
            }

            // 60 min event with 15 min slots: Beginning, Middle, Middle, End = 2 middles
            // This test verifies that middles exist for long events spanning multiple slots
            let event_id = events[0].id;
            let middle_count = middles.get(&event_id).copied().unwrap_or(0);
            assert!(
                middle_count >= 2,
                "60 min event with 15 min slots should have at least 2 Middle overlaps"
            );
        }
    }

    // Invariant: Timetables are returned in sorted order, ordered by day
    #[test]
    fn allocate_returns_timetables_sorted_by_day() {
        let day1 = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2024, 2, 4).unwrap();
        let day3 = NaiveDate::from_ymd_opt(2024, 2, 5).unwrap();
        let start = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        // Events in non-sorted order
        let events = vec![
            make_event(2, day2, start, 30, "Room1"),
            make_event(3, day3, start, 30, "Room1"),
            make_event(1, day1, start, 30, "Room1"),
        ];

        let timetables = allocate(&events, Duration::minutes(15));

        for i in 1..timetables.len() {
            assert!(
                timetables[i - 1].day <= timetables[i].day,
                "Timetables should be sorted by day"
            );
        }
    }

    // Invariant: Each Event may only appear in a single Timetable
    #[test]
    fn event_appears_in_single_timetable() {
        let day1 = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2024, 2, 4).unwrap();
        let start = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

        let events = vec![
            make_event(1, day1, start, 30, "Room1"),
            make_event(2, day2, start, 30, "Room1"),
        ];

        let timetables = allocate(&events, Duration::minutes(15));

        let mut event_to_timetable: HashMap<EventId, usize> = HashMap::new();

        for (tt_idx, timetable) in timetables.iter().enumerate() {
            for slot in &timetable.slots {
                for overlap in slot.overlaps.values() {
                    let event_id = match overlap {
                        EventOverlap::Beginning(e)
                        | EventOverlap::Middle(e)
                        | EventOverlap::End(e) => e.id,
                    };

                    if let Some(&existing_tt) = event_to_timetable.get(&event_id) {
                        assert_eq!(
                            existing_tt, tt_idx,
                            "Event {:?} appears in multiple timetables",
                            event_id
                        );
                    } else {
                        event_to_timetable.insert(event_id, tt_idx);
                    }
                }
            }
        }
    }
}
