use crate::{id::{EdgeId, VertexId}, layer::Layer, lifecycle::LifecycleState, ringbuffer::RingBuffer};
use serde::{Deserialize, Serialize};

pub const HISTORY_WINDOW: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source: VertexId,
    pub target: VertexId,
    pub layer: Layer,
    pub weight: f32,
    pub resistance: f32,
    pub lifecycle: LifecycleState,
    pub history: RingBuffer<f32>,
}

impl Edge {
    pub fn new(id: EdgeId, source: VertexId, target: VertexId, layer: Layer) -> Self {
        Self {
            id,
            source,
            target,
            layer,
            weight: 1.0,
            resistance: 0.0,
            lifecycle: LifecycleState::Active,
            history: RingBuffer::new(HISTORY_WINDOW),
        }
    }

    pub fn conductance(&self) -> f32 {
        1.0 - self.resistance.clamp(0.0, 1.0)
    }
}
