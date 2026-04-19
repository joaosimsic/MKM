use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalState {
    pub position: Vec2,
    pub velocity: Vec2,
    pub kinetic_energy: f32,
    pub mass: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    pub valence: f32,
    pub arousal: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicState {
    pub resources: f32,
    pub flow_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialState {
    pub trust: f32,
    pub reputation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexState {
    pub physical: PhysicalState,
    pub emotional: EmotionalState,
    pub economic: EconomicState,
    pub social: SocialState,
}
