use dotenv;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::embedding::EmbeddingParameters;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let api_key_name = "OPENAI_API_KEY";
    let api_key = dotenv::var(api_key_name).expect(&format!("{} is not set", api_key_name));

    let client = Client::new(api_key);

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: "The food was delicious and the waiter...".to_string(),
        encoding_format: None,
        user: None,
    };

    let result = client.embeddings().create(parameters).await.unwrap();

    println!("{:#?}", result);

    Ok(())
}
