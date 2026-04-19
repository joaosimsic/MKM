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
