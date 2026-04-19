use std::collections::HashSet;

use mkm_core::{
    edge::Edge,
    id::{EdgeId, VertexId},
    invariants::*,
    layer::Layer,
    lifecycle::EdgeLifecycle,
    state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
};

fn good_state() -> VertexState {
    VertexState {
        physical: PhysicalState {
            position: glam::Vec2::ZERO,
            kinetic_energy: 0.5,
        },
        emotional: EmotionalState {
            valence: 0.0,
            arousal: 0.5,
        },
        economic: EconomicState {
            resources: 0.5,
            flow_rate: 0.0,
        },
        social: SocialState {
            reputation: 0.0,
            hierarchy_rank: 0,
            trust: 0.5,
        },
    }
}

#[test]
fn inv1_valid_state() {
    assert!(inv1_state_bounds(&good_state()));
}

#[test]
fn inv1_out_of_range() {
    let mut s = good_state();
    s.physical.kinetic_energy = 1.5;
    assert!(!inv1_state_bounds(&s));

    let mut s = good_state();
    s.emotional.valence = -2.0;
    assert!(!inv1_state_bounds(&s));

    let mut s = good_state();
    s.social.trust = -0.1;
    assert!(!inv1_state_bounds(&s));
}

#[test]
fn inv2_no_self_loops_pass() {
    let e = Edge::new(EdgeId(0), VertexId(0), VertexId(1), Layer::Physical);
    assert!(inv2_no_self_loops(&[e]));
}

#[test]
fn inv2_self_loop_fails() {
    let e = Edge::new(EdgeId(0), VertexId(5), VertexId(5), Layer::Social);
    assert!(!inv2_no_self_loops(&[e]));
}

#[test]
fn inv3_always_true() {
    let e = Edge::new(EdgeId(0), VertexId(0), VertexId(1), Layer::Economic);
    assert!(inv3_layer_isolation(&[e]));
}

#[test]
fn inv4_live_vertices_pass() {
    let e = Edge::new(EdgeId(0), VertexId(0), VertexId(1), Layer::Social);
    let live: HashSet<VertexId> = [VertexId(0), VertexId(1)].into();
    assert!(inv4_no_dangling(&[e], &live));
}

#[test]
fn inv4_dangling_fails() {
    let e = Edge::new(EdgeId(0), VertexId(0), VertexId(99), Layer::Social);
    let live: HashSet<VertexId> = [VertexId(0), VertexId(1)].into();
    assert!(!inv4_no_dangling(&[e], &live));
}

#[test]
fn inv4_snapped_edge_ignored() {
    let mut e = Edge::new(EdgeId(0), VertexId(0), VertexId(99), Layer::Physical);
    e.lifecycle = EdgeLifecycle::Snapped;
    let live: HashSet<VertexId> = [VertexId(0)].into();
    assert!(inv4_no_dangling(&[e], &live));
}

#[test]
fn test_inv5_energy_bounds() {
    assert!(inv5_energy_bounds(0.5, 1.0));
    assert!(inv5_energy_bounds(0.0, 1.0));
    assert!(inv5_energy_bounds(1.0, 1.0));
    assert!(!inv5_energy_bounds(-0.1, 1.0));
    assert!(!inv5_energy_bounds(1.1, 1.0));
}

#[test]
fn test_inv6_mass_monotonicity() {
    assert!(inv6_mass_monotonicity(1.0, 1.005, 0.01));
    assert!(!inv6_mass_monotonicity(1.0, 1.1, 0.01));
}
