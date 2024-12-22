use openai_dive::v1::{
    api::Client,
    resources::embedding::{EmbeddingInput, EmbeddingParameters, EmbeddingResponse},
};

#[tracing::instrument(skip(client))]
pub async fn get_embedding(
    client: &Client,
    input: &str,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: EmbeddingInput::String(input.to_string()),
        encoding_format: None,
        user: None,
        dimensions: None,
    };

    let response = client.embeddings().create(parameters).await.unwrap();

    Ok(response)
}
