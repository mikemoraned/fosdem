use std::path::Path;

use shared::{inmemory_openai::InMemoryOpenAIQueryable, queryable::Queryable};
use test_shared::EVENT_ID_2025;

#[tokio::test]
async fn test_can_find_expected_content() {
    let model_dir = Path::new("./data/model");
    let openai_api_key = "dummy";
    let queryable = InMemoryOpenAIQueryable::connect(model_dir, openai_api_key)
        .await
        .unwrap();

    let expected_event_ids_found = vec![EVENT_ID_2025, EVENT_ID_2025];
    let mut actual_event_ids_found = vec![];
    for event_id in expected_event_ids_found.iter() {
        if let Some(_) = queryable.find_event_by_id(event_id.clone()).await.unwrap() {
            actual_event_ids_found.push(event_id.clone());
        }
    }

    assert_eq!(expected_event_ids_found, actual_event_ids_found);
}
