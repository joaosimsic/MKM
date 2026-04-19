use std::f32::consts::PI;

use glam::Vec2;
use mkm_core::{
    coupling::CouplingState,
    edge::Edge,
    energy::EnergyBudget,
    id::{EdgeId, VertexId},
    inbox::Inbox,
    layer::Layer,
    lifecycle::VertexLifecycle,
    params::{InitDistribution, Params, SimConfig},
    state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
};
use rand::Rng;
use rand_chacha::ChaCha20Rng;

use crate::{
    components::{
        VCoupling, VEnergy, VId, VInbox, VLifecycle, VMass, VPendingEnergyCost, VPrevReputation,
        VState, VertexBundle,
    },
    rng::SimRng,
};

pub struct InitResult {
    pub vertex_bundles: Vec<VertexBundle>,
    pub edges: Vec<Edge>,
}

pub fn init_world(config: &SimConfig, params: &Params, rng: &mut SimRng) -> InitResult {
    let mut pos_rng = rng.fork(0);
    let mut state_rng = rng.fork(1);
    let mut edge_rng = rng.fork(2);

    let n = config.n_vertices;
    let positions = init_positions(n, config, &mut pos_rng);
    let vertex_bundles = init_vertex_bundles(n, config, params, &positions, &mut state_rng);
    let edges = init_edges(n, config, &vertex_bundles, &positions, &mut edge_rng);

    InitResult {
        vertex_bundles,
        edges,
    }
}

fn box_muller(rng: &mut ChaCha20Rng, mean: f32, std: f32) -> f32 {
    let u1: f32 = rng.gen::<f32>().max(1e-10);
    let u2: f32 = rng.gen::<f32>();
    mean + std * (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

fn lerp_range(t: f32, lo: f32, hi: f32) -> f32 {
    lo + t * (hi - lo)
}

fn init_positions(n: usize, config: &SimConfig, rng: &mut ChaCha20Rng) -> Vec<Vec2> {
    let ext = config.spatial_extent;
    match &config.init_distribution {
        InitDistribution::Uniform => (0..n)
            .map(|_| Vec2::new(rng.gen::<f32>() * ext, rng.gen::<f32>() * ext))
            .collect(),

        InitDistribution::Clustered => {
            let k = ((n as f64).sqrt().ceil()) as usize;
            let sigma = 0.05 * ext;
            let centroids: Vec<Vec2> = (0..k)
                .map(|_| Vec2::new(rng.gen::<f32>() * ext, rng.gen::<f32>() * ext))
                .collect();
            (0..n)
                .map(|_| {
                    let c = centroids[rng.gen_range(0..k)];
                    let dx = box_muller(rng, 0.0, sigma);
                    let dy = box_muller(rng, 0.0, sigma);
                    Vec2::new((c.x + dx).clamp(0.0, ext), (c.y + dy).clamp(0.0, ext))
                })
                .collect()
        }

        InitDistribution::PowerLaw => {
            let n_hubs = ((n as f64 * 0.05).ceil() as usize).max(1);
            let hubs: Vec<Vec2> = (0..n_hubs)
                .map(|_| Vec2::new(rng.gen::<f32>() * ext, rng.gen::<f32>() * ext))
                .collect();
            // Zipf weights w_i = 1/i^1.5
            let total: f64 = (1..=n_hubs).map(|i| 1.0 / (i as f64).powf(1.5)).sum();
            let cum: Vec<f64> = (1..=n_hubs)
                .scan(0.0, |acc, i| {
                    *acc += (1.0 / (i as f64).powf(1.5)) / total;
                    Some(*acc)
                })
                .collect();
            let sigma = ext * 0.02;
            (0..n)
                .map(|_| {
                    let r: f64 = rng.gen();
                    let idx = cum.partition_point(|&c| c < r).min(n_hubs - 1);
                    let h = hubs[idx];
                    let dx = box_muller(rng, 0.0, sigma);
                    let dy = box_muller(rng, 0.0, sigma);
                    Vec2::new((h.x + dx).clamp(0.0, ext), (h.y + dy).clamp(0.0, ext))
                })
                .collect()
        }
    }
}

fn init_vertex_bundles(
    n: usize,
    config: &SimConfig,
    params: &Params,
    positions: &[Vec2],
    rng: &mut ChaCha20Rng,
) -> Vec<VertexBundle> {
    let clustered = matches!(config.init_distribution, InitDistribution::Clustered);
    (0..n)
        .map(|i| {
            let pos = positions[i];
            let valence = lerp_range(rng.gen::<f32>(), -0.2, 0.2);
            let arousal = lerp_range(rng.gen::<f32>(), 0.0, 0.2);
            let resources = lerp_range(rng.gen::<f32>(), 0.4, 0.6);
            let (reputation, trust) = if clustered {
                let mu_rep = lerp_range(rng.gen::<f32>(), -0.1, 0.1);
                let mu_trust = lerp_range(rng.gen::<f32>(), 0.3, 0.7);
                (
                    (box_muller(rng, mu_rep, 0.1)).clamp(-1.0, 1.0),
                    (box_muller(rng, mu_trust, 0.1)).clamp(0.0, 1.0),
                )
            } else {
                (
                    lerp_range(rng.gen::<f32>(), -0.1, 0.1),
                    lerp_range(rng.gen::<f32>(), 0.3, 0.7),
                )
            };
            let mass = box_muller(rng, 1.0, 0.2).clamp(0.1, 10.0);

            let state = VertexState {
                physical: PhysicalState {
                    position: pos,
                    kinetic_energy: 0.0,
                },
                emotional: EmotionalState { valence, arousal },
                economic: EconomicState {
                    resources,
                    flow_rate: 0.0,
                },
                social: SocialState {
                    reputation,
                    hierarchy_rank: 0,
                    trust,
                },
            };
            VertexBundle {
                id: VId(VertexId(i as u64)),
                state: VState(state.clone()),
                mass: VMass(mass),
                lifecycle: VLifecycle(VertexLifecycle::Active),
                coupling: VCoupling(CouplingState::default()),
                energy: VEnergy(EnergyBudget::new(
                    params.energy_capacity,
                    params.energy_regen_rate,
                )),
                inbox: VInbox(Inbox::default()),
                prev_reputation: VPrevReputation(state.social.reputation),
                pending_energy: VPendingEnergyCost::default(),
            }
        })
        .collect()
}

fn make_edge(id: &mut u64, src: usize, tgt: usize, layer: Layer, rng: &mut ChaCha20Rng) -> Edge {
    let eid = *id;
    *id += 1;
    let mut e = Edge::new(
        EdgeId(eid),
        VertexId(src as u64),
        VertexId(tgt as u64),
        layer,
    );
    e.weight = lerp_range(rng.gen::<f32>(), -0.5, 0.5);
    e
}

fn init_edges(
    n: usize,
    config: &SimConfig,
    bundles: &[VertexBundle],
    positions: &[Vec2],
    rng: &mut ChaCha20Rng,
) -> Vec<Edge> {
    let target = ((n * n) as f64 * config.edge_density as f64).floor() as usize;
    let k = (target / n.max(1)).max(1);
    let mut eid = 0u64;
    let mut edges = Vec::new();

    // Mp: k-nearest spatial neighbors
    for i in 0..n {
        let mut dists: Vec<(f32, usize)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| (positions[i].distance_squared(positions[j]), j))
            .collect();
        dists.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for &(_, j) in dists.iter().take(k) {
            edges.push(make_edge(&mut eid, i, j, Layer::Physical, rng));
        }
    }

    // Ms: accept ~ 1 - |rep_A - rep_B|
    sample_random_edges(
        n,
        target,
        Layer::Social,
        &mut eid,
        rng,
        &mut edges,
        |a, b| {
            let ra = bundles[a].state.0.social.reputation;
            let rb = bundles[b].state.0.social.reputation;
            1.0 - (ra - rb).abs()
        },
    );

    // Me: accept ~ 1 - |val_A - val_B|
    sample_random_edges(
        n,
        target,
        Layer::Emotional,
        &mut eid,
        rng,
        &mut edges,
        |a, b| {
            let va = bundles[a].state.0.emotional.valence;
            let vb = bundles[b].state.0.emotional.valence;
            1.0 - (va - vb).abs()
        },
    );

    // Mc: accept ~ |res_A - res_B| (complementarity)
    sample_random_edges(
        n,
        target,
        Layer::Economic,
        &mut eid,
        rng,
        &mut edges,
        |a, b| {
            let ra = bundles[a].state.0.economic.resources;
            let rb = bundles[b].state.0.economic.resources;
            (ra - rb).abs()
        },
    );

    edges
}

fn sample_random_edges(
    n: usize,
    target: usize,
    layer: Layer,
    eid: &mut u64,
    rng: &mut ChaCha20Rng,
    edges: &mut Vec<Edge>,
    accept_prob: impl Fn(usize, usize) -> f32,
) {
    let max_attempts = target * 20;
    let mut count = 0usize;
    let mut attempts = 0usize;
    while count < target && attempts < max_attempts {
        attempts += 1;
        let a = rng.gen_range(0..n);
        let b = rng.gen_range(0..n);
        if a == b {
            continue;
        }
        let p = accept_prob(a, b).clamp(0.0, 1.0);
        if rng.gen::<f32>() < p {
            edges.push(make_edge(eid, a, b, layer, rng));
            count += 1;
        }
    }
}
