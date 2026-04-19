use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
};

use bevy::prelude::*;
use mkm_core::{
    lifecycle::VertexLifecycle,
    snapshot::{MeshSnapshot, VertexSnapshot},
};
use serde_json::json;

use crate::{
    components::{VCoupling, VEnergy, VId, VLifecycle, VMass, VState},
    resources::{EdgeStore, ParamsRes, SimClock, SimConfigRes, TickMetrics},
};

pub fn output_system(
    mut clock: ResMut<SimClock>,
    config: Res<SimConfigRes>,
    params: Res<ParamsRes>,
    edges: Res<EdgeStore>,
    mut metrics: ResMut<TickMetrics>,
    query: Query<(&VId, &VState, &VMass, &VLifecycle, &VCoupling, &VEnergy)>,
) {
    let tick = clock.tick;

    // Collect metrics
    let mut active_v = 0usize;
    let mut zombie_v = 0usize;
    for (_, _, _, lc, _, _) in query.iter() {
        match lc.0 {
            VertexLifecycle::Zombie { .. } => zombie_v += 1,
            _ => active_v += 1,
        }
    }
    let active_e = edges.0.iter().filter(|e| {
        e.lifecycle != mkm_core::lifecycle::EdgeLifecycle::Snapped
    }).count();

    metrics.active_vertices = active_v;
    metrics.zombie_count = zombie_v;
    metrics.active_edges = active_e;

    // stdout summary
    println!(
        "tick={} sim_time={:.2} vertices={} zombies={} edges={}",
        tick, clock.sim_time, active_v, zombie_v, active_e
    );

    // JSONL output
    if let Some(ref path_str) = config.0.output_path {
        let line = json!({
            "tick": tick,
            "sim_time": clock.sim_time,
            "active_vertices": active_v,
            "zombie_count": zombie_v,
            "active_edges": active_e,
        });
        let path = PathBuf::from(path_str).join("metrics.jsonl");
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(f, "{}", line);
        }
    }

    // Snapshot flush
    if config.0.snapshot_interval > 0 && tick % config.0.snapshot_interval == 0 {
        flush_snapshot(tick, clock.sim_time, &config, &edges, &query);
    }

    // Advance clock at end of tick
    clock.sim_time += params.0.dt;
    clock.tick += 1;
}

fn flush_snapshot(
    tick: u64,
    sim_time: f32,
    config: &SimConfigRes,
    edges: &EdgeStore,
    query: &Query<(&VId, &VState, &VMass, &VLifecycle, &VCoupling, &VEnergy)>,
) {
    let vertices: Vec<VertexSnapshot> = query
        .iter()
        .map(|(id, state, mass, lc, coupling, energy)| VertexSnapshot {
            id: id.0,
            mass: mass.0,
            state: state.0.clone(),
            lifecycle: lc.0.clone(),
            coupling: coupling.0.clone(),
            energy: energy.0.clone(),
        })
        .collect();

    let snapshot = MeshSnapshot {
        tick,
        sim_time,
        vertices,
        edges: edges.0.clone(),
    };

    if let Some(ref path_str) = config.0.output_path {
        let dir = PathBuf::from(path_str);
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join(format!("tick_{tick:08}.msgpack"));
        if let Err(e) = snapshot.save(&file) {
            eprintln!("snapshot write error at {}: {e}", file.display());
        }
    }
}
