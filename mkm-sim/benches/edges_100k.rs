use criterion::{criterion_group, criterion_main, Criterion};
use mkm_core::{
    edge::Edge,
    id::{EdgeId, VertexId},
    layer::{signal_extractor, Layer},
    lifecycle::EdgeLifecycle,
    state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
};

fn make_state(ke: f32) -> VertexState {
    VertexState {
        physical: PhysicalState {
            position: glam::Vec2::ZERO,
            kinetic_energy: ke,
        },
        emotional: EmotionalState {
            valence: 0.5,
            arousal: 0.3,
        },
        economic: EconomicState {
            resources: 0.5,
            flow_rate: 0.2,
        },
        social: SocialState {
            trust: 0.7,
            reputation: 0.4,
            hierarchy_rank: 0,
        },
    }
}

fn bench_propagate_100k(c: &mut Criterion) {
    const N_VERTICES: usize = 1_000;
    const EDGES_PER_VERTEX: usize = 100; // 100K total edges
    const N_EDGES: usize = N_VERTICES * EDGES_PER_VERTEX;

    let states: Vec<VertexState> = (0..N_VERTICES)
        .map(|i| make_state(i as f32 / N_VERTICES as f32))
        .collect();
    let masses: Vec<f32> = vec![1.0; N_VERTICES];

    let edges: Vec<Edge> = (0..N_EDGES)
        .map(|i| {
            let mut e = Edge::new(
                EdgeId(i as u64),
                VertexId((i % N_VERTICES) as u64),
                VertexId(((i + 1) % N_VERTICES) as u64),
                Layer::Physical,
            );
            e.resistance = 0.1;
            e
        })
        .collect();

    c.bench_function("propagate_100k_edges", |b| {
        b.iter(|| {
            let mut inbox_totals = vec![0.0_f32; N_VERTICES];
            for edge in &edges {
                if edge.lifecycle != EdgeLifecycle::Active {
                    continue;
                }
                let src = edge.source.0 as usize;
                let tgt = edge.target.0 as usize;
                let s_in = signal_extractor(edge.layer, &states[src]);
                let g = edge.conductance();
                let mw = masses[src] / (1.0 + masses[src]);
                let s_out = edge.weight * g * s_in * mw;
                inbox_totals[tgt] += s_out;
            }
            std::hint::black_box(inbox_totals)
        });
    });
}

fn bench_history_100k(c: &mut Criterion) {
    const N_EDGES: usize = 100_000;
    let mut edges: Vec<Edge> = (0..N_EDGES)
        .map(|i| {
            let mut e = Edge::new(
                EdgeId(i as u64),
                VertexId((i % 1000) as u64),
                VertexId(((i + 1) % 1000) as u64),
                Layer::Physical,
            );
            e.resistance = 0.3;
            e
        })
        .collect();
    let alpha = 0.8_f32;

    c.bench_function("history_100k_edges", |b| {
        b.iter(|| {
            for edge in &mut edges {
                edge.history.push(0.25_f32);
                edge.update_resistance(alpha);
            }
        });
    });
}

criterion_group!(benches, bench_propagate_100k, bench_history_100k);
criterion_main!(benches);
