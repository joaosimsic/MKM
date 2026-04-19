use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyBudget {
    pub current: f32,
    pub capacity: f32,
    pub regen_rate: f32,
}

impl EnergyBudget {
    pub fn new(capacity: f32, regen_rate: f32) -> Self {
        Self {
            current: capacity,
            capacity,
            regen_rate,
        }
    }

    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= cost
    }

    pub fn deduct(&mut self, cost: f32) {
        self.current = (self.current - cost).max(0.0);
    }

    pub fn regen(&mut self) {
        self.current = (self.current + self.regen_rate).min(self.capacity);
    }
}
