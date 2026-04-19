use crate::state::VertexState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layer {
    Physical,
    Emotional,
    Economic,
    Social,
}

impl Layer {
    pub const ALL: [Layer; 4] = [
        Layer::Physical,
        Layer::Emotional,
        Layer::Economic,
        Layer::Social,
    ];
}

/// Extract the scalar signal for a given layer from a vertex's composite state.
///
/// - Physical  → `kinetic_energy`
/// - Emotional → `arousal * sign(valence)`
/// - Economic  → `flow_rate`
/// - Social    → `trust * reputation`
pub fn signal_extractor(layer: Layer, state: &VertexState) -> f32 {
    match layer {
        Layer::Physical => state.physical.kinetic_energy,
        Layer::Emotional => state.emotional.arousal * state.emotional.valence.signum(),
        Layer::Economic => state.economic.flow_rate,
        Layer::Social => state.social.trust * state.social.reputation,
    }
}
