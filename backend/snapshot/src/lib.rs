use shared::{inmemory_openai::InMemoryOpenAIQueryable, model::SearchItem, queryable::Queryable};

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

    pub async fn search(&self, title: &str) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        self.queryable.search(title, 20, true).await.unwrap();
        Ok(vec![])
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
        let snapshotter = Snapshotter::new(&openai_api_key, &model_dir).await.unwrap();
        let similar = snapshotter.search("controversial").await.unwrap();
        insta::assert_yaml_snapshot!(similar);
    }
}
