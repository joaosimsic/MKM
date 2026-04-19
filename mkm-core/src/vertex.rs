use crate::{id::VertexId, state::VertexState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub id: VertexId,
    pub state: VertexState,
    pub influence_mass: f32,
}
