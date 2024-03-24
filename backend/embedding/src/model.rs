use nalgebra::DVector;
use serde::{Deserialize, Serialize};
use shared::model::EventArtefact;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SubjectEmbedding {
    pub subject: EventArtefact,
    pub embedding: Embedding,
}

impl SubjectEmbedding {
    pub fn new(subject: EventArtefact, embedding: Embedding) -> SubjectEmbedding {
        SubjectEmbedding { subject, embedding }
    }
}

pub type OpenAIVector = DVector<f64>;
pub fn distance(lhs: &OpenAIVector, rhs: &OpenAIVector) -> f64 {
    lhs.metric_distance(rhs)
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Embedding {
    OpenAIAda2 { vector: OpenAIVector },
}
