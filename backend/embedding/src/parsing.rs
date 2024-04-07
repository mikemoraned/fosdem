use std::{fs::File, io::BufReader, path::PathBuf};

use tracing::debug;

use crate::model::SubjectEmbedding;

pub fn parse_all_subject_embeddings(
    embeddings_paths: &Vec<PathBuf>,
) -> Result<Vec<SubjectEmbedding>, Box<dyn std::error::Error>> {
    let mut all_embeddings = Vec::new();

    for embeddings_path in embeddings_paths {
        debug!("Loading embeddings from {:?}", embeddings_path);

        let reader = BufReader::new(File::open(embeddings_path)?);
        let mut embeddings: Vec<SubjectEmbedding> = serde_json::from_reader(reader)?;
        debug!(
            "Loaded {} embeddings from {:?}",
            embeddings.len(),
            embeddings_path
        );

        all_embeddings.append(&mut embeddings);
    }

    all_embeddings.sort_by(|a, b| a.partial_cmp(b).unwrap());

    Ok(all_embeddings)
}
