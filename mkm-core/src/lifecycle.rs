use serde::{Deserialize, Serialize};

/// Lifecycle state for a vertex. `Zombie` retains `decay_since` tick for
/// re-entry memory semantics — decayed state is preserved.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VertexLifecycle {
    Active,
    Strained,
    Zombie { decay_since: u64 },
}

/// Lifecycle state for an edge.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EdgeLifecycle {
    Active,
    Strained,
    Snapped,
}
