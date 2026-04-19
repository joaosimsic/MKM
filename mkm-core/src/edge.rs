use crate::{
    id::{EdgeId, VertexId},
    layer::Layer,
    lifecycle::EdgeLifecycle,
    ringbuffer::RingBuffer,
};
use serde::{Deserialize, Serialize};

pub const HISTORY_WINDOW: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub source: VertexId,
    pub target: VertexId,
    pub layer: Layer,
    pub weight: f32,
    pub resistance: f32,
    pub lifecycle: EdgeLifecycle,
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
            resistance: 0.1,
            lifecycle: EdgeLifecycle::Active,
            history: RingBuffer::new(HISTORY_WINDOW),
        }
    }

    /// g = 1 / (1 + r)  per docs §2
    pub fn conductance(&self) -> f32 {
        1.0 / (1.0 + self.resistance)
    }

    /// Hysteresis update: r_{t+1} = α·r_t + (1-α)·mean(|history|)
    pub fn update_resistance(&mut self, alpha: f32) {
        let hist_mean = self.history.mean();
        self.resistance = (alpha * self.resistance + (1.0 - alpha) * hist_mean).clamp(0.0, 1.0);
    }
}
