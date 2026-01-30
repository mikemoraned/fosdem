use std::collections::HashMap;

use chrono::{Duration, NaiveDate, NaiveTime};
use shared::model::Event;

// A Timetable is an allocation of Events into TimeSlots for a single day.
// invariants:
// * A Timetable never spans multiple days
// * `slots` is an ordered series of TimeSlots separated by `slot_duration`. In other words `slots` has no gaps
pub struct Timetable {
    day: NaiveDate,
    slots: Vec<TimeSlot>,
    slot_duration: Duration,
}

// A TimeSlot has a start and can overlap with different Events in different Streams
// invariants:
// * The same Event event may appear in an EventOverlap in multiple Streams
// * The same Event cannot have multiple occurrences via different EventOverlaps in the same Stream
pub struct TimeSlot {
    start: NaiveTime,
    overlaps: HashMap<Stream, EventOverlap>,
}

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

    // TODO: write tests that test the invariants
}
