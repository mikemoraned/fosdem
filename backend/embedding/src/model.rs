use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SubjectEmbedding {
    subject: EventArtefact,
    embedding: Embedding,
}

impl SubjectEmbedding {
    pub fn new(subject: EventArtefact, embedding: Embedding) -> SubjectEmbedding {
        SubjectEmbedding { subject, embedding }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Embedding {
    OpenAIAda2 { vector: DVector<f64> },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum EventArtefact {
    Combined { event_id: EventId },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EventId(pub u32);
