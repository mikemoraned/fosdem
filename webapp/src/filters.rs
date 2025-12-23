use shared::model::Event;
use unicode_segmentation::UnicodeSegmentation;

fn count_graphemes(s: &str) -> usize {
    UnicodeSegmentation::graphemes(s, true).count()
}

pub fn truncate_title(
    title: &str,
    _: &dyn askama::Values,
    max_size: usize,
) -> ::askama::Result<String> {
    if count_graphemes(title) <= max_size {
        return Ok(title.to_string());
    }

    let suffix = " …";
    let suffix_length = count_graphemes(suffix);
    let mut available = max_size - suffix_length;

    let chunks = title.split_word_bounds().collect::<Vec<&str>>();
    let mut limited = vec![];
    for chunk in chunks {
        let chunk_length = count_graphemes(chunk);
        if available >= chunk_length {
            limited.push(chunk);
            available -= chunk_length;
        } else {
            break;
        }
    }
    limited.push(suffix);
    Ok(limited.join(""))
}

pub fn distance_similarity(distance: &f64, _: &dyn askama::Values) -> ::askama::Result<String> {
    let similarity = 1.0 - distance;
    Ok(format!("{:.2}", similarity))
}

pub fn distance_icon(distance: &f64, _: &dyn askama::Values) -> ::askama::Result<String> {
    let similarity = 1.0 - distance;
    let assumed_max_typical_similarity = 0.60;
    let opacity = (similarity / assumed_max_typical_similarity).min(1.0f64);
    Ok(format!(
        "<i class=\"fa-solid fa-circle\" style=\"opacity: {}\"></i>",
        opacity
    ))
}

pub fn order_event_by_time_then_place(
    events: &[Event],
    _: &dyn askama::Values,
) -> ::askama::Result<Vec<Event>> {
    let mut ordered = Vec::from(events);
    ordered.sort_by(|a, b| {
        if a.starting_time() == b.starting_time() {
            a.room.cmp(&b.room)
        } else {
            a.starting_time().cmp(&b.starting_time())
        }
    });
    Ok(ordered)
}
