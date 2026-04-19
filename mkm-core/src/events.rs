use crate::id::VertexId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerDelta {
    pub physical: f32,
    pub emotional: f32,
    pub economic: f32,
    pub social: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventTarget {
    Global,
    Vertex(VertexId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TensorImpact {
    pub target: EventTarget,
    pub delta: LayerDelta,
    pub tick: u64,
}

/// A queue of pending `TensorImpact` events, ordered by tick.
///
/// This is a plain data struct (no Bevy dependency). In mkm-sim it is wrapped
/// in a `#[derive(Resource)]` newtype.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EventQueue {
    pub pending: Vec<TensorImpact>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drain all events whose `tick` is ≤ `current_tick`, returning them in
    /// order. Remaining events stay in the queue.
    pub fn drain_tick(&mut self, current_tick: u64) -> Vec<TensorImpact> {
        let mut due = Vec::new();
        let mut remaining = Vec::new();
        for event in self.pending.drain(..) {
            if event.tick <= current_tick {
                due.push(event);
            } else {
                remaining.push(event);
            }
        }
        self.pending = remaining;
        due
    }
}
