use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TickStage {
    Ingest,
    Propagate,
    Bridges,
    Crisis,
    Plasticity,
    History,
    Output,
}
