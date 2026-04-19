use std::collections::HashSet;

use crate::{edge::Edge, id::VertexId, lifecycle::EdgeLifecycle, state::VertexState};

/// Invariant 1: all scalar state values within declared ranges.
pub fn inv1_state_bounds(state: &VertexState) -> bool {
    let p = &state.physical;
    let e = &state.emotional;
    let c = &state.economic;
    let s = &state.social;
    (0.0..=1.0).contains(&p.kinetic_energy)
        && (-1.0..=1.0).contains(&e.valence)
        && (0.0..=1.0).contains(&e.arousal)
        && (0.0..=1.0).contains(&c.resources)
        && (-1.0..=1.0).contains(&c.flow_rate)
        && (-1.0..=1.0).contains(&s.reputation)
        && (0.0..=1.0).contains(&s.trust)
}

/// Invariant 2: no self-loop edges (source != target).
pub fn inv2_no_self_loops(edges: &[Edge]) -> bool {
    edges.iter().all(|e| e.source != e.target)
}

/// Invariant 3: layer isolation — each edge belongs to exactly one layer.
/// Structurally guaranteed by the type system.
pub fn inv3_layer_isolation(_edges: &[Edge]) -> bool {
    true
}

/// Invariant 4: no dangling edges — every non-snapped edge references a live vertex.
pub fn inv4_no_dangling(edges: &[Edge], live_vertices: &HashSet<VertexId>) -> bool {
    edges
        .iter()
        .filter(|e| e.lifecycle != EdgeLifecycle::Snapped)
        .all(|e| live_vertices.contains(&e.source) && live_vertices.contains(&e.target))
}

/// Invariant 5: energy budget current is within [0, capacity].
pub fn inv5_energy_bounds(current: f32, capacity: f32) -> bool {
    current >= 0.0 && current <= capacity
}

/// Invariant 6: |Δmass| per tick ≤ MASS_DELTA_MAX.
pub fn inv6_mass_monotonicity(prev_mass: f32, next_mass: f32, mass_delta_max: f32) -> bool {
    (next_mass - prev_mass).abs() <= mass_delta_max + f32::EPSILON
}

/// Invariant 7: determinism is verified externally (SHA-256 comparison across runs).
pub fn inv7_determinism_marker() -> bool {
    true
}
