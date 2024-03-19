use openai_dive::v1::{
    api::Client,
    resources::embedding::{EmbeddingParameters, EmbeddingResponse},
};

#[tracing::instrument(skip(client))]
pub async fn get_embedding(
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
