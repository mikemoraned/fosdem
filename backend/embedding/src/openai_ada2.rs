use content::{slide_index::SlideIndex, video_index::VideoIndex};
use openai_dive::v1::{
    api::Client,
    resources::embedding::{EmbeddingParameters, EmbeddingResponse},
};
use shared::model::Event;
use subtp::vtt::VttBlock;
use tracing::debug;

use crate::{
    input::{format_basic_input, trim_input},
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
) -> Result<Embedding, Box<dyn std::error::Error>> {
    let mut preferred_input = String::new();
    use std::fmt::Write;

    writeln!(preferred_input, "{}", format_basic_input(event))?;
    if let Some(slide_content) = slide_index.entries.get(&event.id) {
        writeln!(preferred_input, "Slides:{}", slide_content)?;
    }
    if let Some(video_content) = video_index.webvtt_for_event_id(event.id) {
        let mut block_content: Vec<_> = video_content
            .blocks
            .iter()
            .map(|b| match b {
                VttBlock::Que(cue) => cue.payload.join("\n"),
                _ => "".into(),
            })
            .collect();
        block_content.dedup();
        debug!("[{}] blocks: {:?}", event.id, block_content);
        writeln!(preferred_input, "Subtitles:{}", block_content.join("\n"))?;
    }

    let trimmed_input = trim_input(&preferred_input);

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: trimmed_input,
        encoding_format: None,
        user: None,
    };

    match client.embeddings().create(parameters).await {
        Ok(response) => {
            let embedding = Embedding::OpenAIAda2 {
                vector: OpenAIVector::from(response.data[0].embedding.clone()),
            };
            Ok(embedding)
        }
        Err(e) => Err(format!("[{}] error: \'{}\'", event.id, e).into()),
    }
}
