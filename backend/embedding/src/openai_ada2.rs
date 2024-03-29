use content::{slide_index::SlideIndex, video_index::VideoIndex};
use openai_dive::v1::{
    api::Client,
    resources::embedding::{EmbeddingParameters, EmbeddingResponse},
};
use shared::model::{Event, EventId};

use crate::{
    input::{FormatStatistics, InputBuilder},
    model::{Embedding, OpenAIVector},
};

#[tracing::instrument(skip(client))]
pub async fn get_phrase_embedding(
    client: &Client,
    input: &str,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: input.to_string(),
        encoding_format: None,
        user: None,
    };

    let response = client.embeddings().create(parameters).await.unwrap();

    Ok(response)
}

pub async fn get_event_embedding(
    client: &Client,
    event: &Event,
    slide_index: &SlideIndex,
    video_index: &VideoIndex,
) -> Result<(Embedding, FormatStatistics), Box<dyn std::error::Error>> {
    let builder = InputBuilder::new(EventId(event.id))
        .with_event_source(&event)
        .with_slide_source(&slide_index)
        .with_video_source(&video_index);

    let max_tokens = 8192 - 100;
    let (input, statistics) = builder.format_with_statistics(max_tokens)?;

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input,
        encoding_format: None,
        user: None,
    };

    match client.embeddings().create(parameters).await {
        Ok(response) => {
            let embedding = Embedding::OpenAIAda2 {
                vector: OpenAIVector::from(response.data[0].embedding.clone()),
            };
            Ok((embedding, statistics))
        }
        Err(e) => Err(format!("[{}] error: \'{}\'", event.id, e).into()),
    }
}

pub async fn get_video_embedding(
    client: &Client,
    event_id: &EventId,
    video_index: &VideoIndex,
) -> Result<(Embedding, FormatStatistics), Box<dyn std::error::Error>> {
    let builder = InputBuilder::new(event_id.clone()).with_video_source(&video_index);

    let max_tokens = 8192 - 100;
    let (input, statistics) = builder.format_with_statistics(max_tokens)?;

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input,
        encoding_format: None,
        user: None,
    };

    match client.embeddings().create(parameters).await {
        Ok(response) => {
            let embedding = Embedding::OpenAIAda2 {
                vector: OpenAIVector::from(response.data[0].embedding.clone()),
            };
            Ok((embedding, statistics))
        }
        Err(e) => Err(format!("[{:?}] error: \'{}\'", event_id, e).into()),
    }
}
