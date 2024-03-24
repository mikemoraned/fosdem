use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SubjectEmbedding {
    subject: EventArtefact,
}

impl SubjectEmbedding {
    pub fn new(subject: EventArtefact) -> SubjectEmbedding {
        SubjectEmbedding { subject }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum EventArtefact {
    Combined { event_id: EventId },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EventId(pub u32);
