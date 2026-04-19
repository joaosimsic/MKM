use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CouplingMode {
    Loose,
    Tight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouplingState {
    pub mode: CouplingMode,
    pub level: f32,
    pub ema: f32,
}

impl Default for CouplingState {
    fn default() -> Self {
        Self {
            mode: CouplingMode::Loose,
            level: 0.0,
            ema: 0.0,
        }
    }
}
