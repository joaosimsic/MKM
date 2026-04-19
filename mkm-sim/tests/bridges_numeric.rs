/// Numeric integration tests for the 8 default bridge functions.
///
/// Each test verifies that a single bridge produces a hand-computed delta
/// within 1e-4 at a fixed input, with A=1 and D(m)=1 (mass=1, coupling=0).
use glam::Vec2;
use mkm_core::{
    params::Params,
    state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
};
use mkm_sim::bridge_registry::{BridgeRegistry, VertexView};

fn default_state() -> VertexState {
    VertexState {
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
            trust: 0.5,
        },
    }
}

fn apply_all(state: &VertexState, prev_rep: f32) -> mkm_sim::bridge_registry::BridgeDelta {
    let p = Params::default();
    let reg = BridgeRegistry::with_defaults();
    let view = VertexView {
        state,
        mass: 1.0,
        prev_reputation: prev_rep,
    };
    // A=1 (coupling level=0), D(m)=1 (mass=1)
    reg.apply_all(&view, &p, 1.0)
}

// ── B_ep ─────────────────────────────────────────────────────────────────────

#[test]
fn b_ep_delta_kinetic() {
    let p = Params::default();
    let mut state = default_state();
    state.emotional.arousal = 0.6;
    let delta = apply_all(&state, 0.0);
    let expected = p.k_ep * 0.6;
    assert!(
        (delta.d_kinetic - expected).abs() < 1e-4,
        "B_ep: d_kinetic={} expected={}",
        delta.d_kinetic,
        expected
    );
}

#[test]
fn b_ep_zero_at_zero_arousal() {
    let state = default_state();
    let delta = apply_all(&state, 0.0);
    assert!(
        delta.d_kinetic.abs() < 1e-6,
        "B_ep should be zero at zero arousal"
    );
}

// ── B_pe ─────────────────────────────────────────────────────────────────────

#[test]
fn b_pe_delta_arousal() {
    let p = Params::default();
    let mut state = default_state();
    state.physical.kinetic_energy = 0.4;
    let delta = apply_all(&state, 0.0);
    let expected = p.k_pe * 0.4;
    assert!(
        (delta.d_arousal - expected).abs() < 1e-4,
        "B_pe: d_arousal={} expected={}",
        delta.d_arousal,
        expected
    );
}

// ── B_ec ─────────────────────────────────────────────────────────────────────

#[test]
fn b_ec_negative_flow_on_fear() {
    let p = Params::default();
    let mut state = default_state();
    state.emotional.arousal = 0.8;
    state.emotional.valence = -0.5; // fear
                                    // default_state has trust=0.5, kinetic=0 → B_sc and B_pc also fire.
    let b_ec = -p.k_ec * 0.8 * 0.5;
    let b_sc = p.k_sc * (state.social.trust - p.trust_threshold); // 0.3*(0.5-0.4)=0.03
    let b_pc = -p.k_pc * state.physical.kinetic_energy; // 0
    let expected = b_ec + b_sc + b_pc;
    let delta = apply_all(&state, 0.0);
    assert!(
        (delta.d_flow_rate - expected).abs() < 1e-4,
        "B_ec+B_sc+B_pc: d_flow={} expected={}",
        delta.d_flow_rate,
        expected
    );
    // Verify fear-driven component is negative and dominates
    assert!(
        delta.d_flow_rate < 0.0,
        "net flow should be negative on strong fear"
    );
}

#[test]
fn b_ec_zero_on_positive_valence() {
    // When valence >= 0, fear term is zero — B_ec should not fire.
    let mut state = default_state();
    state.emotional.arousal = 0.8;
    state.emotional.valence = 0.3;
    let delta = apply_all(&state, 0.0);
    // B_ec contributes 0; only other bridges contribute to flow.
    // B_pc also 0 (kinetic=0), B_sc = k_sc * (0.5 - trust_threshold).
    let p = Params::default();
    let b_sc = p.k_sc * (0.5 - p.trust_threshold);
    assert!(
        (delta.d_flow_rate - b_sc).abs() < 1e-4,
        "d_flow should only reflect B_sc, got {}",
        delta.d_flow_rate
    );
}

// ── B_ce ─────────────────────────────────────────────────────────────────────

#[test]
fn b_ce_positive_at_high_resources() {
    let p = Params::default();
    let mut state = default_state();
    state.economic.resources = 0.9;
    let delta = apply_all(&state, 0.0);
    // Only B_ce affects valence (other bridges don't touch valence at default state).
    // B_cs could... let me compute full expected.
    let expected_ce = p.k_ce * (0.9 - 0.5); // = K_CE * 0.4
    assert!(
        (delta.d_valence - expected_ce).abs() < 1e-4,
        "B_ce: d_valence={} expected={}",
        delta.d_valence,
        expected_ce
    );
}

#[test]
fn b_ce_negative_at_low_resources() {
    let p = Params::default();
    let mut state = default_state();
    state.economic.resources = 0.1;
    let delta = apply_all(&state, 0.0);
    let expected_ce = p.k_ce * (0.1 - 0.5);
    assert!(
        delta.d_valence < 0.0,
        "B_ce should decrease valence at low resources"
    );
    assert!(
        (delta.d_valence - expected_ce).abs() < 1e-4,
        "B_ce magnitude: got {} expected {}",
        delta.d_valence,
        expected_ce
    );
}

// ── B_pc ─────────────────────────────────────────────────────────────────────

#[test]
fn b_pc_flow_contraction() {
    let p = Params::default();
    let mut state = default_state();
    state.physical.kinetic_energy = 0.5;
    let delta = apply_all(&state, 0.0);
    // B_pc = -K_PC * 0.5; B_sc = K_SC * (0.5 - threshold)
    let b_pc = -p.k_pc * 0.5;
    let b_sc = p.k_sc * (state.social.trust - p.trust_threshold);
    let expected = b_pc + b_sc;
    assert!(
        (delta.d_flow_rate - expected).abs() < 1e-4,
        "B_pc+B_sc: d_flow={} expected={}",
        delta.d_flow_rate,
        expected
    );
}

// ── B_se ─────────────────────────────────────────────────────────────────────

#[test]
fn b_se_fires_on_rep_drop() {
    let p = Params::default();
    let mut state = default_state();
    state.social.reputation = 0.2; // dropped from 0.5
    let prev_rep = 0.5;
    let delta = apply_all(&state, prev_rep);
    let expected = p.k_se * (0.5 - 0.2);
    assert!(
        (delta.d_arousal - expected).abs() < 1e-4,
        "B_se: d_arousal={} expected={}",
        delta.d_arousal,
        expected
    );
}

#[test]
fn b_se_zero_on_rep_gain() {
    let mut state = default_state();
    state.social.reputation = 0.7; // gained from 0.3
    let delta = apply_all(&state, 0.3);
    assert!(
        delta.d_arousal >= 0.0,
        "B_se should not fire on rep gain (only B_pe can add arousal)"
    );
    // B_pe is 0 since kinetic=0, B_se is 0 since no drop.
    assert!(delta.d_arousal.abs() < 1e-6, "d_arousal should be zero");
}

// ── B_sc ─────────────────────────────────────────────────────────────────────

#[test]
fn b_sc_positive_above_threshold() {
    let p = Params::default();
    let mut state = default_state();
    state.social.trust = 0.8; // above trust_threshold=0.4
    let delta = apply_all(&state, 0.0);
    let b_sc = p.k_sc * (0.8 - p.trust_threshold);
    // Also B_ec=0 (arousal=0), B_pc=0 (kinetic=0)
    assert!(
        delta.d_flow_rate > 0.0,
        "B_sc should increase flow when trust > threshold"
    );
    assert!(
        (delta.d_flow_rate - b_sc).abs() < 1e-4,
        "B_sc: d_flow={} expected={}",
        delta.d_flow_rate,
        b_sc
    );
}

// ── B_cs ─────────────────────────────────────────────────────────────────────

#[test]
fn b_cs_trust_follows_flow() {
    let p = Params::default();
    let mut state = default_state();
    state.economic.flow_rate = 0.6;
    let delta = apply_all(&state, 0.0);
    let expected = p.k_cs * 0.6;
    assert!(
        (delta.d_trust - expected).abs() < 1e-4,
        "B_cs: d_trust={} expected={}",
        delta.d_trust,
        expected
    );
}

#[test]
fn b_cs_negative_on_negative_flow() {
    let p = Params::default();
    let mut state = default_state();
    state.economic.flow_rate = -0.4;
    let delta = apply_all(&state, 0.0);
    let expected = p.k_cs * (-0.4);
    assert!(
        (delta.d_trust - expected).abs() < 1e-4,
        "B_cs: d_trust={} expected={}",
        delta.d_trust,
        expected
    );
}

// ── Custom bridge registration ───────────────────────────────────────────────

#[test]
fn custom_bridge_registered_at_runtime_fires() {
    use mkm_core::layer::Layer;
    use mkm_sim::bridge_registry::{BridgeDelta, BridgeFn, BridgeRegistry, VertexView};

    struct CustomBridge;
    impl BridgeFn for CustomBridge {
        fn source_layer(&self) -> Layer {
            Layer::Economic
        }
        fn target_layer(&self) -> Layer {
            Layer::Physical
        }
        fn apply(&self, _view: &VertexView, _params: &Params) -> BridgeDelta {
            BridgeDelta {
                d_kinetic: 0.42,
                ..Default::default()
            }
        }
    }

    let p = Params::default();
    let mut reg = BridgeRegistry::with_defaults();
    reg.register(Box::new(CustomBridge));

    let state = default_state();
    let view = VertexView {
        state: &state,
        mass: 1.0,
        prev_reputation: 0.0,
    };
    // apply_all sums all bridges; CustomBridge always contributes 0.42 to d_kinetic.
    // Default state has arousal=0 so B_ep contributes 0; only CustomBridge adds 0.42.
    let delta = reg.apply_all(&view, &p, 1.0);
    assert!(
        (delta.d_kinetic - 0.42).abs() < 1e-5,
        "custom bridge should fire: d_kinetic={} expected=0.42",
        delta.d_kinetic
    );
}

// ── Coupling amplification ────────────────────────────────────────────────────

#[test]
fn coupling_amplification_doubles_at_level_one() {
    use mkm_sim::bridge_registry::coupling_amplification;
    let p = Params::default();
    let a = coupling_amplification(1.0, p.coupling_amplification);
    assert!(
        (a - p.coupling_amplification).abs() < 1e-6,
        "A at level=1 should equal COUPLING_AMPLIFICATION"
    );
}

#[test]
fn bridge_output_amplified_by_coupling() {
    let p = Params::default();
    let reg = BridgeRegistry::with_defaults();
    let mut state = default_state();
    state.emotional.arousal = 0.5;
    let view = VertexView {
        state: &state,
        mass: 1.0,
        prev_reputation: 0.0,
    };
    let base = reg.apply_all(&view, &p, 1.0);
    let amped = reg.apply_all(
        &view,
        &p,
        mkm_sim::bridge_registry::coupling_amplification(1.0, p.coupling_amplification),
    );
    let ratio = amped.d_kinetic / base.d_kinetic;
    assert!(
        (ratio - p.coupling_amplification).abs() < 1e-4,
        "coupling_level=1 should amplify by COUPLING_AMPLIFICATION={}, got ratio={}",
        p.coupling_amplification,
        ratio
    );
}
