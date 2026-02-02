use std::collections::BTreeMap;

use crate::queryable::Queryable;

#[derive(Debug)]
pub struct DataSummary {
    pub events_by_year: BTreeMap<u32, usize>,
}

impl DataSummary {
    pub fn total_events(&self) -> usize {
        self.events_by_year.values().sum()
    }
}

pub async fn load_summary<Q: Queryable>(
    queryable: &Q,
) -> Result<DataSummary, Box<dyn std::error::Error>> {
    let events = queryable.load_all_events().await?;

    let mut events_by_year: BTreeMap<u32, usize> = BTreeMap::new();
    for event in events {
        *events_by_year.entry(event.year).or_insert(0) += 1;
    }

    Ok(DataSummary { events_by_year })
}
