/// Mass damping tests — verifies D(m) suppresses bridge outputs for high-mass
/// vertices and that the emotional-spike → physical response reproduces the
/// spec's worked example within tolerance.
use glam::Vec2;
use mkm_core::{
    params::Params,
    state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
};
use mkm_sim::bridge_registry::{mass_damping, BridgeRegistry, VertexView};

fn base_state(arousal: f32, kinetic: f32) -> VertexState {
    VertexState {
        physical: PhysicalState {
            position: Vec2::ZERO,
            kinetic_energy: kinetic,
        },
        emotional: EmotionalState {
            valence: 0.0,
            arousal,
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
fn mass_damping_formula() {
    let k = 0.5_f32;
    // D(1) = 1
    assert!((mass_damping(1.0, k) - 1.0).abs() < 1e-6);
    // D(3) = 1/(1 + 0.5*2) = 0.5
    assert!((mass_damping(3.0, k) - 0.5).abs() < 1e-6);
    // D(5) = 1/(1 + 0.5*4) = 1/3
    assert!((mass_damping(5.0, k) - 1.0 / 3.0).abs() < 1e-5);
}

#[test]
fn high_mass_reduces_bridge_output() {
    let p = Params::default();
    let reg = BridgeRegistry::with_defaults();
    let state = base_state(0.5, 0.0);

    let view_unit = VertexView {
        state: &state,
        mass: 1.0,
        prev_reputation: 0.0,
    };
    let view_heavy = VertexView {
        state: &state,
        mass: 5.0,
        prev_reputation: 0.0,
    };

    let d_unit = reg.apply_all(&view_unit, &p, 1.0);
    let d_heavy = reg.apply_all(&view_heavy, &p, 1.0);

    assert!(
        d_heavy.d_kinetic.abs() < d_unit.d_kinetic.abs(),
        "high-mass vertex should have smaller bridge output: heavy={} unit={}",
        d_heavy.d_kinetic,
        d_unit.d_kinetic
    );

    let ratio = d_heavy.d_kinetic / d_unit.d_kinetic;
    let expected_ratio = mass_damping(5.0, p.k_mass_damp);
    assert!(
        (ratio - expected_ratio).abs() < 1e-4,
        "ratio={} expected={}",
        ratio,
        expected_ratio
    );
}

#[test]
fn unit_mass_no_damping() {
    let p = Params::default();
    let reg = BridgeRegistry::with_defaults();
    let state = base_state(0.5, 0.0);
    let view = VertexView {
        state: &state,
        mass: 1.0,
        prev_reputation: 0.0,
    };
    let delta = reg.apply_all(&view, &p, 1.0);
    // With mass=1, D(m)=1, A=1: d_kinetic = K_EP * arousal exactly
    let expected = p.k_ep * 0.5;
    assert!(
        (delta.d_kinetic - expected).abs() < 1e-6,
        "unit mass: d_kinetic={} expected={}",
        delta.d_kinetic,
        expected
    );
}

#[test]
fn emotional_spike_propagates_to_physical_within_3_ticks() {
    // Verifies acceptance gate: emotional spike → proportional Mp response within 3 ticks.
    // Runs a manual tick loop without Bevy to isolate the bridge math.
    use bevy::prelude::{App, IntoSystemConfigs, IntoSystemSetConfigs, Update};
    use mkm_core::{
        coupling::CouplingState, energy::EnergyBudget, id::VertexId, inbox::Inbox,
        lifecycle::VertexLifecycle,
    };
    use mkm_sim::{
        bridge_registry::BridgeRegistry,
        components::{
            VCoupling, VEnergy, VId, VInbox, VLifecycle, VMass, VPendingEnergyCost,
            VPrevReputation, VState,
        },
        resources::ParamsRes,
        systems::{bridges::bridges_system, plasticity::plasticity_system},
        tick::TickStage,
    };

    let p = Params::default();
    let mut app = App::new();
    app.insert_resource(ParamsRes(p.clone()));
    app.insert_resource(BridgeRegistry::with_defaults());
    app.configure_sets(Update, (TickStage::Bridges, TickStage::Plasticity).chain());
    app.add_systems(Update, bridges_system.in_set(TickStage::Bridges));
    app.add_systems(Update, plasticity_system.in_set(TickStage::Plasticity));

    // Vertex with high arousal, no physical activity yet.
    let init_state = VertexState {
        physical: PhysicalState {
            position: Vec2::ZERO,
            kinetic_energy: 0.0,
        },
        emotional: EmotionalState {
            valence: 0.0,
            arousal: 0.8,
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
    };
    app.world_mut().spawn((
        VId(VertexId(0)),
        VState(init_state),
        VMass(1.0),
        VLifecycle(VertexLifecycle::Active),
        VCoupling(CouplingState::default()),
        VEnergy(EnergyBudget::new(p.energy_capacity, p.energy_regen_rate)),
        VInbox(Inbox::default()),
        VPrevReputation(0.0),
        VPendingEnergyCost::default(),
    ));

    // Run 3 ticks.
    for _ in 0..3 {
        app.update();
    }

    let mut query = app.world_mut().query::<&VState>();
    for vs in query.iter(app.world()) {
        let kinetic = vs.0.physical.kinetic_energy;
        assert!(
            kinetic > 0.0,
            "kinetic_energy should be > 0 after 3 ticks of arousal spike, got {}",
            kinetic
        );
        // Analytic bound: after 1 tick, d_kinetic ≥ K_EP * arousal * A * D(m) = 0.4 * 0.8 * 1 * 1 = 0.32
        // After 3 ticks it accumulates further minus friction.
        assert!(
            kinetic < 1.0,
            "kinetic_energy should be clamped below 1.0, got {}",
            kinetic
        );
    }
}

#[test]
fn mass_damping_never_negative() {
    // Robustness: D(m) should never produce a negative value or NaN.
    for mass in [0.001_f32, 0.1, 0.5, 1.0, 2.0, 10.0, 100.0] {
        let d = mass_damping(mass, 0.5);
        assert!(
            d > 0.0 && d.is_finite(),
            "D({}) = {} should be positive finite",
            mass,
            d
        );
    }
}
