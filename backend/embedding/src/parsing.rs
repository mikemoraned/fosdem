use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use shared::model::{EventArtefact, EventId};
use tracing::debug;

use crate::model::SubjectEmbedding;

pub fn parse_all_subject_embeddings_into_index(
    embeddings_paths: &Vec<PathBuf>,
) -> Result<HashMap<EventId, Vec<SubjectEmbedding>>, Box<dyn std::error::Error>> {
    let mut index: HashMap<EventId, Vec<SubjectEmbedding>> = HashMap::new();

    for embeddings_path in embeddings_paths {
        debug!("Loading embeddings data from {:?}", embeddings_path);
        let mut entry_count = 0;

        let reader = BufReader::new(File::open(embeddings_path)?);
        let embeddings: Vec<SubjectEmbedding> = serde_json::from_reader(reader)?;

        for embedding in embeddings {
            let event_id = match &embedding.subject {
                EventArtefact::Combined { event_id } => event_id,
                EventArtefact::Video { event_id, file: _ } => event_id,
            };
            if let Some(entries) = index.get_mut(&event_id) {
                entries.push(embedding.clone());
            } else {
                index.insert(event_id.clone(), vec![embedding]);
            }
            entry_count += 1
        }
        debug!(
            "Loaded {} embeddings data from {:?}",
            entry_count, embeddings_path
        );
    }

    Ok(index)
}
