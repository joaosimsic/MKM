use bevy::prelude::*;
use mkm_core::{
    edge::Edge,
    events::EventQueue,
    params::{Params, SimConfig},
};

use crate::rng::SimRng;

#[derive(Resource)]
pub struct EdgeStore(pub Vec<Edge>);

#[derive(Resource)]
pub struct SimClock {
    pub tick: u64,
    pub sim_time: f32,
}

impl SimClock {
    pub fn new() -> Self {
        Self {
            tick: 0,
            sim_time: 0.0,
        }
    }
}

impl Default for SimClock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource)]
pub struct SimRngRes(pub SimRng);

#[derive(Resource)]
pub struct SimConfigRes(pub SimConfig);

#[derive(Resource)]
pub struct ParamsRes(pub Params);

#[derive(Resource, Default)]
pub struct EventQueueRes(pub EventQueue);

#[derive(Resource, Default)]
pub struct TickMetrics {
    pub active_vertices: usize,
    pub zombie_count: usize,
    pub active_edges: usize,
}
