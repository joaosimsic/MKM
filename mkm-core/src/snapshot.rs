use crate::{
    coupling::CouplingState,
    edge::Edge,
    energy::EnergyBudget,
    id::VertexId,
    lifecycle::VertexLifecycle,
    state::VertexState,
};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

/// Snapshot of a single vertex — all fields required for full state restore.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VertexSnapshot {
    pub id: VertexId,
    pub mass: f32,
    pub state: VertexState,
    pub lifecycle: VertexLifecycle,
    pub coupling: CouplingState,
    pub energy: EnergyBudget,
}

/// Full simulation snapshot at a given tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshSnapshot {
    pub tick: u64,
    pub sim_time: f32,
    pub vertices: Vec<VertexSnapshot>,
    pub edges: Vec<Edge>,
}

impl MeshSnapshot {
    /// Serialize to msgpack bytes (in-memory, for hashing / comparison).
    pub fn to_bytes(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    /// Deserialize from msgpack bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }

    /// Write snapshot to a file at `path` using msgpack encoding.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let bytes = self
            .to_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(path, bytes)
    }

    /// Load a snapshot from a msgpack file at `path`.
    pub fn load(path: &Path) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
}
