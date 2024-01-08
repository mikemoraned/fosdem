use std::{collections::HashMap, fs::File, io::Write};

use chrono::{NaiveDate, NaiveTime};
use clap::Parser;
use dotenvy;

use shared::{cli::progress_bar, env::load_secret, queryable::Queryable};
use tracing::info;
use webapp::related::{D3Force, Link, Node};

/// generate related items
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// maximum number of related items to include per event
    #[arg(short, long)]
    limit: u8,

    /// output json file
    #[arg(short, long)]
    json: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv()?;
    let args = Args::parse();

    let openai_api_key = load_secret("OPENAI_API_KEY");
    let db_id = load_secret("DB_ID");
    let db_password = load_secret("DB_PASSWORD");

    info!("Loading all Events and converting to Nodes");
    let queryable = Queryable::connect(&db_id, &db_password, &openai_api_key).await?;
    let events = queryable.load_all_events().await?;
    let mut titles_covered: HashMap<String, usize> = HashMap::new();
    let mut nodes: Vec<Node> = vec![];
    let time_slots: Vec<(NaiveDate, NaiveTime)> =
        events.iter().map(|e| (e.date, e.start)).collect();
    let time_slot_ids = build_time_slot_ids(&time_slots);
    for event in events.iter() {
        let new_index = nodes.len();
        titles_covered.insert(event.title.clone(), new_index);
        let time_slot = (event.date, event.start);
        let time_slot_id = time_slot_ids.get(&time_slot).unwrap();
        nodes.push(Node {
            index: new_index,
            title: event.title.clone(),
            url: event.url.clone(),
            time_slot: *time_slot_id,
            day: event.date.format("%a").to_string(),
            start: event.start.format("%H:%M").to_string(),
        });
    }

    info!(
        "Loading all related Events (limit: {}) and converting to Links",
        args.limit
    );
    let mut links: Vec<Link> = vec![];
    let progress = progress_bar(events.len() as u64);
    for event in events.into_iter() {
        let related = queryable
            .find_related_events(&event.title, args.limit)
            .await?;
        let source = *titles_covered.get(&event.title).unwrap();
        for item in related.iter() {
            let target = *titles_covered.get(&item.event.title).unwrap();
            let distance = item.distance;
            links.push(Link {
                source,
                target,
                distance,
            });
        }
        progress.inc(1);
    }

    info!("Converting to JSON");
    let forces = D3Force { nodes, links };
    let json = serde_json::to_string(&forces)?;

    let json_file_path = args.json;
    info!("Saving to {}", json_file_path);
    let mut json_file = File::create(json_file_path)?;
    json_file.write_all(json.as_bytes())?;

    Ok(())
}

fn build_time_slot_ids(
    time_slots: &[(NaiveDate, NaiveTime)],
) -> HashMap<(NaiveDate, NaiveTime), usize> {
    let mut sorted = vec![];
    for time_slot in time_slots {
        sorted.push(time_slot);
    }
    sorted.sort();
    sorted.dedup();
    let mut time_slot_ids = HashMap::new();
    for (id, time_slot) in sorted.into_iter().enumerate() {
        time_slot_ids.insert(time_slot.clone(), id);
    }
    time_slot_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_time_slot_ids() {
        let day1 = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2024, 2, 4).unwrap();
        let slot1 = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
        let slot2 = NaiveTime::from_hms_opt(14, 10, 0).unwrap();

        let mut expected = HashMap::new();
        expected.insert((day1, slot1), 0);
        expected.insert((day1, slot2), 1);
        expected.insert((day2, slot1), 2);
        expected.insert((day2, slot2), 3);

        let time_slots = vec![(day2, slot2), (day2, slot1), (day1, slot1), (day1, slot2)];
        let actual = build_time_slot_ids(&time_slots);

        assert_eq!(expected, actual);
    }
}
