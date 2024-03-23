use shared::{inmemory_openai::InMemoryOpenAIQueryable, model::SearchItem, queryable::Queryable};

pub struct Snapshotter {
    queryable: InMemoryOpenAIQueryable,
}

impl Snapshotter {
    pub async fn new(openai_api_key: &str, model_dir: &std::path::Path) -> Snapshotter {
        let queryable = InMemoryOpenAIQueryable::connect(model_dir, openai_api_key)
            .await
            .unwrap();
        Snapshotter { queryable }
    }

    pub async fn search(&self, title: &str) -> Vec<SearchItem> {
        self.queryable.search(title, 20, true).await.unwrap();
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use shared::env::load_secret;

    use super::*;

    #[tokio::test]
    async fn test_phrase() {
        let openai_api_key = load_secret("OPENAI_API_KEY").unwrap();
        let model_dir = PathBuf::from_str("../shared/data/model").unwrap();
        let snapshotter = Snapshotter::new(&openai_api_key, &model_dir).await;
        let similar = snapshotter.search("controversial").await;
        insta::assert_yaml_snapshot!(similar);
    }
}
