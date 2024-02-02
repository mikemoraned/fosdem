use std::collections::HashMap;

use shared::queryable::SearchItem;
use unicode_segmentation::UnicodeSegmentation;

fn count_graphemes(s: &str) -> usize {
    UnicodeSegmentation::graphemes(s, true).into_iter().count()
}

pub fn truncate_title(title: &String, max_size: usize) -> ::askama::Result<String> {
    if count_graphemes(&title) <= max_size {
        return Ok(title.clone());
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

pub fn distance_similarity(distance: &f64) -> ::askama::Result<String> {
    let similarity = 1.0 - distance;
    Ok(format!("{:.2}", similarity).into())
}

pub fn distance_icon(distance: &f64) -> ::askama::Result<String> {
    let similarity = 1.0 - distance;
    let assumed_max_typical_similarity = 0.60;
    let opacity = (similarity / assumed_max_typical_similarity).min(1.0f64);
    Ok(format!(
        "<i class=\"fa-solid fa-circle\" style=\"opacity: {}\"></i>",
        opacity
    )
    .into())
}

pub struct GroupedSearchItems {
    pub distance: f64,
    pub items: Vec<SearchItem>,
}

pub fn group_by_distance(items: &Vec<SearchItem>) -> ::askama::Result<Vec<GroupedSearchItems>> {
    let mut group_map: HashMap<String, Vec<SearchItem>> = HashMap::new();
    for item in items {
        let group_name = distance_similarity(&item.distance)?;
        group_map
            .entry(group_name)
            .and_modify(|e| e.push(item.clone()))
            .or_insert(vec![item.clone()]);
    }
    let mut grouped: Vec<GroupedSearchItems> = group_map
        .into_iter()
        .map(|(_, items)| GroupedSearchItems {
            distance: items[0].distance,
            items,
        })
        .collect();
    grouped.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    Ok(grouped)
}
