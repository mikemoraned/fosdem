use query::{inmemory_openai::InMemoryOpenAIQueryable, queryable::Queryable};
use serde::Serialize;
use shared::model::SearchItem;

#[derive(Debug, PartialEq, Serialize)]
pub struct RankSummary {
    distance: f64,
    rounded_distance: f64,
    rank_in_search: usize,
    event_id: u32,
    event_title: String,
}

impl RankSummary {
    fn from_ranked_search_item(ranked_item: (usize, &SearchItem)) -> RankSummary {
        let (rank, item) = ranked_item;
        RankSummary {
            distance: item.distance,
            rounded_distance: (item.distance * 100.0).round() / 100.0,
            rank_in_search: rank,
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

    pub async fn find_related_events(
        &self,
        title: &str,
    ) -> Result<Vec<RankSummary>, Box<dyn std::error::Error>> {
        Ok(Snapshotter::summarise(
            &self.queryable.find_related_events(title, 20).await?,
        ))
    }

    fn summarise(items: &[SearchItem]) -> Vec<RankSummary> {
        let mut summaries: Vec<RankSummary> = items
            .iter()
            .enumerate()
            .map(RankSummary::from_ranked_search_item)
            .collect();

        summaries.sort_by(|a, b| a.event_id.cmp(&b.event_id));

        summaries
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use shared::env::load_secret;

    use super::*;

    static TITLES: [&str; 3] = [
        "Best practices for research in open source ecosystems",
        "Where Did All the Fun Go?  And How to Bring it Back with FOSS!",
        "Staying Ahead of the Game: JavaScript Security",
    ];

    #[tokio::test]
    async fn test_find_related_events() {
        let openai_api_key = load_secret("OPENAI_API_KEY").unwrap();
        let model_dir = PathBuf::from_str("../shared/data/model").unwrap();
        let snapshotter = Snapshotter::new(&openai_api_key, &model_dir).await.unwrap();
        for title in TITLES {
            let similar = snapshotter.find_related_events(title).await.unwrap();
            insta::with_settings!({
                info => &model_dir,
                description => title,
                omit_expression => true
            }, {
                insta::assert_yaml_snapshot!(similar);
            });
        }
    }
}
