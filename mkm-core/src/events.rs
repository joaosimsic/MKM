use crate::id::VertexId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDelta {
    pub physical: f32,
    pub emotional: f32,
    pub economic: f32,
    pub social: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTarget {
    Global,
    Vertex(VertexId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorImpact {
    pub target: EventTarget,
    pub delta: LayerDelta,
    pub tick: u64,
}
