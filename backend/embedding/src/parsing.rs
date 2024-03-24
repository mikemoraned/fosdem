use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use shared::model::{EventArtefact, EventId};
use tracing::debug;

use crate::model::{Embedding, OpenAIVector, SubjectEmbedding};

pub fn parse_all_embeddings_into_index(
    embeddings_path: &Path,
) -> Result<HashMap<EventId, OpenAIVector>, Box<dyn std::error::Error>> {
    debug!("Loading embeddings data from {:?}", embeddings_path);

    let mut index: HashMap<EventId, OpenAIVector> = HashMap::new();

    let reader = BufReader::new(File::open(embeddings_path)?);
    let embeddings: Vec<SubjectEmbedding> = serde_json::from_reader(reader)?;

    for embedding in embeddings {
        let EventArtefact::Combined { event_id } = embedding.subject;
        let Embedding::OpenAIAda2 { vector } = embedding.embedding;

        if let Some(_) = index.get(&event_id) {
            return Err(format!("event_id is not unique: {:?}", event_id).into());
        }

        index.insert(event_id, vector);
    }

    Ok(index)
}
