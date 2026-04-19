use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Physical layer state. `kinetic_energy` ∈ [0, 1]; `position` is unbounded
/// (clamped per `spatial_extent` on init, not enforced as an invariant).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhysicalState {
    pub position: Vec2,
    pub kinetic_energy: f32,
}

/// Emotional layer state. `valence` ∈ [-1, 1]; `arousal` ∈ [0, 1].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmotionalState {
    pub valence: f32,
    pub arousal: f32,
}

/// Economic layer state. `resources` ∈ [0, 1]; `flow_rate` ∈ [-1, 1].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EconomicState {
    pub resources: f32,
    pub flow_rate: f32,
}

/// Social layer state. `reputation` ∈ [-1, 1]; `trust` ∈ [0, 1];
/// `hierarchy_rank` is unbounded (ordinal).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialState {
    pub reputation: f32,
    pub hierarchy_rank: u32,
    pub trust: f32,
}

/// Composite per-vertex state across all four layers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VertexState {
    pub physical: PhysicalState,
    pub emotional: EmotionalState,
    pub economic: EconomicState,
    pub social: SocialState,
}

impl VertexState {
    /// Clamp all scalar fields to their declared valid ranges.
    pub fn clamp_all(&mut self) {
        self.physical.kinetic_energy = self.physical.kinetic_energy.clamp(0.0, 1.0);
        self.emotional.valence = self.emotional.valence.clamp(-1.0, 1.0);
        self.emotional.arousal = self.emotional.arousal.clamp(0.0, 1.0);
        self.economic.resources = self.economic.resources.clamp(0.0, 1.0);
        self.economic.flow_rate = self.economic.flow_rate.clamp(-1.0, 1.0);
        self.social.reputation = self.social.reputation.clamp(-1.0, 1.0);
        self.social.trust = self.social.trust.clamp(0.0, 1.0);
    }
}
