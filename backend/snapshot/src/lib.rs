use serde::Serialize;
use shared::{inmemory_openai::InMemoryOpenAIQueryable, model::SearchItem, queryable::Queryable};

#[derive(Debug, PartialEq, Serialize)]
pub struct DistanceSummary {
    distance: f64,
    event_id: u32,
    event_title: String,
}

impl DistanceSummary {
    fn from_search_item(item: &SearchItem) -> DistanceSummary {
        DistanceSummary {
            distance: item.distance,
            event_id: item.event.id,
            event_title: item.event.title.clone(),
        }
    }
}

pub struct Snapshotter {
    queryable: InMemoryOpenAIQueryable,
}

impl Snapshotter {
    pub async fn new(
        openai_api_key: &str,
        model_dir: &std::path::Path,
    ) -> Result<Snapshotter, Box<dyn std::error::Error>> {
        let queryable = InMemoryOpenAIQueryable::connect(model_dir, openai_api_key)
            .await
            .unwrap();
        Ok(Snapshotter { queryable })
    }

    pub async fn search(
        &self,
        title: &str,
    ) -> Result<Vec<DistanceSummary>, Box<dyn std::error::Error>> {
        Ok(Snapshotter::summarise(
            &self.queryable.search(title, 20, true).await.unwrap(),
        ))
    }

    fn summarise(items: &[SearchItem]) -> Vec<DistanceSummary> {
        items
            .iter()
            .map(DistanceSummary::from_search_item)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use shared::env::load_secret;

    use super::*;

    static PHRASES: [&str; 1] = ["controversial"];

    #[tokio::test]
    async fn test_phrase_search() {
        let openai_api_key = load_secret("OPENAI_API_KEY").unwrap();
        let model_dir = PathBuf::from_str("../shared/data/model").unwrap();
        let snapshotter = Snapshotter::new(&openai_api_key, &model_dir).await.unwrap();
        for phrase in PHRASES {
            let similar = snapshotter.search(phrase).await.unwrap();
            insta::assert_yaml_snapshot!(similar);
        }
    }
}
