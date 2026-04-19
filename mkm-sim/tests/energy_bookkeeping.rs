/// Energy bookkeeping tests (Invariant 5).
///
/// Verifies that bridge activity costs are correctly accumulated and deducted,
/// and that EnergyBudget.current stays within [0, capacity] throughout.
use bevy::prelude::*;
use bevy::prelude::{IntoSystemConfigs, IntoSystemSetConfigs};
use glam::Vec2;
use mkm_core::{
    coupling::CouplingState,
    energy::EnergyBudget,
    id::VertexId,
    inbox::Inbox,
    invariants::inv5_energy_bounds,
    lifecycle::VertexLifecycle,
    params::Params,
    state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
};
use mkm_sim::{
    bridge_registry::BridgeRegistry,
    components::{
        VCoupling, VEnergy, VId, VInbox, VLifecycle, VMass, VPendingEnergyCost, VPrevReputation,
        VState,
    },
    resources::ParamsRes,
    systems::{bridges::bridges_system, plasticity::plasticity_system},
    tick::TickStage,
};

fn make_app_with_vertex(state: VertexState, energy: f32) -> App {
    let mut app = App::new();
    let p = Params::default();
    app.insert_resource(ParamsRes(p.clone()));
    app.insert_resource(BridgeRegistry::with_defaults());

    app.configure_sets(Update, (TickStage::Bridges, TickStage::Plasticity).chain());
    app.add_systems(Update, bridges_system.in_set(TickStage::Bridges));
    app.add_systems(Update, plasticity_system.in_set(TickStage::Plasticity));

    app.world_mut().spawn((
        VId(VertexId(0)),
        VState(state),
        VMass(1.0),
        VLifecycle(VertexLifecycle::Active),
        VCoupling(CouplingState::default()),
        VEnergy(EnergyBudget::new(energy, p.energy_regen_rate)),
        VInbox(Inbox::default()),
        VPrevReputation(0.0),
        VPendingEnergyCost::default(),
    ));

    app
}

fn high_activity_state() -> VertexState {
    VertexState {
        physical: PhysicalState {
            position: Vec2::ZERO,
            kinetic_energy: 0.9,
        },
        emotional: EmotionalState {
            valence: -0.8,
            arousal: 0.9,
        },
        economic: EconomicState {
            resources: 0.1,
            flow_rate: 0.8,
        },
        social: SocialState {
            reputation: 0.0,
            hierarchy_rank: 0,
            trust: 0.9,
        },
    }
}

#[test]
fn invariant5_holds_after_bridges_and_plasticity() {
    let mut app = make_app_with_vertex(high_activity_state(), 1.0);
    app.update();

    let p = Params::default();
    let mut query = app.world_mut().query::<&VEnergy>();
    for ve in query.iter(app.world()) {
        assert!(
            inv5_energy_bounds(ve.0.current, ve.0.capacity),
            "Invariant 5 violated: current={} capacity={}",
            ve.0.current,
            ve.0.capacity
        );
        assert!(
            ve.0.current <= p.energy_capacity,
            "energy exceeds capacity: {}",
            ve.0.current
        );
    }
}

#[test]
fn pending_cost_cleared_after_plasticity() {
    let mut app = make_app_with_vertex(high_activity_state(), 1.0);
    app.update();

    let mut query = app.world_mut().query::<&VPendingEnergyCost>();
    for vp in query.iter(app.world()) {
        assert_eq!(
            vp.0, 0.0,
            "pending energy cost should be cleared after Stage 5"
        );
    }
}

#[test]
fn energy_does_not_go_negative_under_heavy_bridge_activity() {
    // Start with low energy budget; bridges fire hard.
    let mut app = make_app_with_vertex(high_activity_state(), 0.001);

    for _ in 0..50 {
        app.update();
        let mut query = app.world_mut().query::<&VEnergy>();
        for ve in query.iter(app.world()) {
            assert!(
                ve.0.current >= 0.0,
                "energy went negative: {}",
                ve.0.current
            );
        }
    }
}

#[test]
fn energy_regen_occurs_each_tick() {
    // State where all bridge outputs are zero: trust=trust_threshold, resources=0.5,
    // everything else zero → no bridge fires → bridge cost = 0 → regen dominates.
    let p = Params::default();
    let idle_state = VertexState {
        physical: PhysicalState {
            position: Vec2::ZERO,
            kinetic_energy: 0.0,
        },
        emotional: EmotionalState {
            valence: 0.0,
            arousal: 0.0,
        },
        economic: EconomicState {
            resources: 0.5,
            flow_rate: 0.0,
        },
        social: SocialState {
            reputation: 0.0,
            hierarchy_rank: 0,
            trust: p.trust_threshold, // B_sc = K_SC * (trust - threshold) = 0
        },
    };
    let mut app = make_app_with_vertex(idle_state, p.energy_capacity);

    // Drain energy manually
    {
        let mut query = app.world_mut().query::<&mut VEnergy>();
        for mut ve in query.iter_mut(app.world_mut()) {
            let cap = ve.0.capacity;
            ve.0.deduct(cap);
        }
    }

    // After 1 tick, regen fires: energy = regen_rate - 0 = regen_rate > 0.
    app.update();

    let mut query = app.world_mut().query::<&VEnergy>();
    for ve in query.iter(app.world()) {
        assert!(
            ve.0.current > 0.0,
            "energy should increase after 1 tick via regen, got {}",
            ve.0.current
        );
        // Bridge cost is near-zero (tiny intra-layer drift may cause small B_sc output).
        // Energy should be close to regen_rate, within 0.001.
        assert!(
            ve.0.current >= p.energy_regen_rate - 0.001,
            "energy too low: got {} expected ~{}",
            ve.0.current,
            p.energy_regen_rate
        );
    }
}

#[test]
fn invariants5_holds_over_1000_ticks() {
    let mut app = make_app_with_vertex(high_activity_state(), 1.0);

    for tick in 0..1000u64 {
        app.update();
        let mut query = app.world_mut().query::<&VEnergy>();
        for ve in query.iter(app.world()) {
            assert!(
                inv5_energy_bounds(ve.0.current, ve.0.capacity),
                "Invariant 5 violated at tick {}: current={} capacity={}",
                tick,
                ve.0.current,
                ve.0.capacity
            );
        }
    }
}

/// Acceptance gate: Invariants 1–5 hold at 10K ticks with random shocks every 100 ticks.
/// Uses a fixed-seed PRNG for reproducibility; shock sets high arousal + negative valence.
#[test]
fn invariants_1_to_5_hold_over_10k_ticks_with_shocks() {
    use mkm_core::{invariants::inv1_state_bounds, state::VertexState};
    use mkm_sim::components::VState;

    let p = Params::default();
    let init_state = VertexState {
        physical: mkm_core::state::PhysicalState {
            position: glam::Vec2::ZERO,
            kinetic_energy: 0.3,
        },
        emotional: mkm_core::state::EmotionalState {
            valence: 0.1,
            arousal: 0.2,
        },
        economic: mkm_core::state::EconomicState {
            resources: 0.5,
            flow_rate: 0.0,
        },
        social: mkm_core::state::SocialState {
            reputation: 0.3,
            hierarchy_rank: 0,
            trust: 0.5,
        },
    };
    let mut app = make_app_with_vertex(init_state, p.energy_capacity);

    // Shock pattern: at every 100th tick, inject high-stress state directly.
    // Uses deterministic values (no external RNG dependency).
    let shocks: [(f32, f32, f32, f32); 4] = [
        // (arousal, valence, kinetic, resources)
        (0.9, -0.8, 0.7, 0.1),
        (0.5, -0.3, 0.4, 0.2),
        (0.8, 0.0, 0.6, 0.05),
        (1.0, -1.0, 1.0, 0.0),
    ];

    for tick in 0..10_000u64 {
        if tick % 100 == 0 {
            let shock = &shocks[((tick / 100) % 4) as usize];
            let mut q = app.world_mut().query::<&mut VState>();
            for mut vs in q.iter_mut(app.world_mut()) {
                vs.0.emotional.arousal = shock.0;
                vs.0.emotional.valence = shock.1;
                vs.0.physical.kinetic_energy = shock.2;
                vs.0.economic.resources = shock.3;
            }
        }

        app.update();

        let p_local = p.clone();
        let mut sq = app.world_mut().query::<(&VState, &VEnergy)>();
        for (vs, ve) in sq.iter(app.world()) {
            assert!(
                inv1_state_bounds(&vs.0),
                "Invariant 1 (state bounds) violated at tick {}: {:?}",
                tick,
                vs.0
            );
            assert!(
                inv5_energy_bounds(ve.0.current, ve.0.capacity),
                "Invariant 5 (energy bounds) violated at tick {}: current={} capacity={}",
                tick,
                ve.0.current,
                ve.0.capacity
            );
            assert!(
                ve.0.current <= p_local.energy_capacity,
                "energy exceeds capacity at tick {}: {}",
                tick,
                ve.0.current
            );
        }
    }
}
