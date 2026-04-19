use mkm_core::{
    coupling::{CouplingMode, CouplingState},
    edge::Edge,
    energy::EnergyBudget,
    id::{EdgeId, VertexId},
    layer::Layer,
    lifecycle::VertexLifecycle,
    snapshot::{MeshSnapshot, VertexSnapshot},
    state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
};

fn sample_vertex_snapshot(id: u64) -> VertexSnapshot {
    VertexSnapshot {
        id: VertexId(id),
        mass: 1.0 + id as f32 * 0.1,
        state: VertexState {
            physical: PhysicalState {
                position: glam::Vec2::new(1.0, 2.0),
                kinetic_energy: 0.5,
            },
            emotional: EmotionalState {
                valence: 0.3,
                arousal: 0.7,
            },
            economic: EconomicState {
                resources: 0.6,
                flow_rate: 0.1,
            },
            social: SocialState {
                reputation: 0.2,
                hierarchy_rank: 0,
                trust: 0.8,
            },
        },
        lifecycle: VertexLifecycle::Active,
        coupling: CouplingState {
            mode: CouplingMode::Loose,
            level: 0.0,
            ema: 0.0,
        },
        energy: EnergyBudget::new(1.0, 0.01),
    }
}

fn sample_edge(id: u64) -> Edge {
    Edge::new(EdgeId(id), VertexId(0), VertexId(1), Layer::Physical)
}

fn sample_snapshot() -> MeshSnapshot {
    MeshSnapshot {
        tick: 42,
        sim_time: 42.0,
        vertices: (0..4).map(sample_vertex_snapshot).collect(),
        edges: (0..2).map(sample_edge).collect(),
    }
}

#[test]
fn msgpack_roundtrip() {
    let original = sample_snapshot();
    let bytes = original.to_bytes().expect("serialize");
    let restored = MeshSnapshot::from_bytes(&bytes).expect("deserialize");
    assert_eq!(original, restored);
}

#[test]
fn double_roundtrip_identical() {
    let snap = sample_snapshot();
    let bytes1 = snap.to_bytes().expect("serialize 1");
    let snap2 = MeshSnapshot::from_bytes(&bytes1).expect("deserialize");
    let bytes2 = snap2.to_bytes().expect("serialize 2");
    assert_eq!(bytes1, bytes2, "save→load→save must be bit-identical");
}
