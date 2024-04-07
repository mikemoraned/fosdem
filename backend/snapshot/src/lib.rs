use query::{
    inmemory_openai::InMemoryOpenAIQueryable,
    queryable::{Queryable, SearchKind},
};
use serde::Serialize;
use shared::model::SearchItem;

#[derive(Debug, PartialEq, Serialize)]
pub struct Summary {
    items: Vec<ItemSummary>,
    ranks: Vec<RankSummary>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ItemSummary {
    event_id: u32,
    event_title: String,
}

impl ItemSummary {
    fn from_search_item(item: &SearchItem) -> ItemSummary {
        ItemSummary {
            event_id: item.event.id,
            event_title: item.event.title.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct RankSummary {
    event_id: u32,
    rounded_distance: f64,
    rank_in_search: usize,
}

impl RankSummary {
    fn from_ranked_search_item(ranked_item: (usize, &SearchItem)) -> RankSummary {
        let (rank, item) = ranked_item;
        RankSummary {
            rounded_distance: item.distance.clone().into(),
            rank_in_search: rank,
            event_id: item.event.id,
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
        kind: &SearchKind,
    ) -> Result<Summary, Box<dyn std::error::Error>> {
        Ok(Snapshotter::summarise(
            &self.queryable.find_related_events(title, kind, 20).await?,
        ))
    }

    fn summarise(items: &[SearchItem]) -> Summary {
        let mut item_summaries: Vec<ItemSummary> =
            items.iter().map(ItemSummary::from_search_item).collect();

        item_summaries.sort_by(|a, b| a.event_id.cmp(&b.event_id));

        let mut rank_sumamries: Vec<RankSummary> = items
            .iter()
            .enumerate()
            .map(RankSummary::from_ranked_search_item)
            .collect();

        rank_sumamries.sort_by(|a, b| a.event_id.cmp(&b.event_id));

        Summary {
            items: item_summaries,
            ranks: rank_sumamries,
        }
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

    #[derive(Serialize)]
    struct TestInfo {
        model_dir: PathBuf,
        search_kind: SearchKind,
    }

    #[tokio::test]
    async fn test_find_related_events_videoonly_searchkind() {
        let kind = &SearchKind::VideoOnly;

        let openai_api_key = load_secret("OPENAI_API_KEY").unwrap();
        let model_dir = PathBuf::from_str("../shared/data/model").unwrap();
        let snapshotter = Snapshotter::new(&openai_api_key, &model_dir).await.unwrap();
        for title in TITLES {
            let similar = snapshotter.find_related_events(title, kind).await.unwrap();
            insta::with_settings!({
                info => &TestInfo {
                    model_dir: model_dir.clone(),
                    search_kind: kind.clone(),
                },
                description => title,
                omit_expression => true
            }, {
                insta::assert_yaml_snapshot!(similar);
            });
        }
    }

    #[tokio::test]
    async fn test_find_related_events_combined_searchkind() {
        let kind = &SearchKind::Combined;

        let openai_api_key = load_secret("OPENAI_API_KEY").unwrap();
        let model_dir = PathBuf::from_str("../shared/data/model").unwrap();
        let snapshotter = Snapshotter::new(&openai_api_key, &model_dir).await.unwrap();
        for title in TITLES {
            let similar = snapshotter.find_related_events(title, kind).await.unwrap();
            insta::with_settings!({
                info => &TestInfo {
                    model_dir: model_dir.clone(),
                    search_kind: kind.clone(),
                },
                description => title,
                omit_expression => true
            }, {
                insta::assert_yaml_snapshot!(similar);
            });
        }
    }
}
