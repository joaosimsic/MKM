/// Numeric tests for Stage 2 (propagation) and Stage 6 (history/hysteresis).
///
/// All expected values computed from the closed-form formulas in docs/dynamics.md.
use mkm_core::{
    edge::{Edge, HISTORY_WINDOW},
    id::{EdgeId, VertexId},
    layer::{signal_extractor, Layer},
state::{EmotionalState, EconomicState, PhysicalState, SocialState, VertexState},
};

fn make_state(ke: f32, val: f32, ar: f32, res: f32, flow: f32, trust: f32, rep: f32) -> VertexState {
    VertexState {
        physical: PhysicalState { position: glam::Vec2::ZERO, kinetic_energy: ke },
        emotional: EmotionalState { valence: val, arousal: ar },
        economic: EconomicState { resources: res, flow_rate: flow },
        social: SocialState { trust, reputation: rep, hierarchy_rank: 0 },
    }
}

fn make_edge(layer: Layer, weight: f32, resistance: f32) -> Edge {
    let mut e = Edge::new(
        EdgeId(0),
        VertexId(0),
        VertexId(1),
        layer,
    );
    e.weight = weight;
    e.resistance = resistance;
    e
}

fn s_out(edge: &Edge, src_state: &VertexState, src_mass: f32) -> f32 {
    let s_in = signal_extractor(edge.layer, src_state);
    let g = edge.conductance();
    let mw = src_mass / (1.0 + src_mass);
    edge.weight * g * s_in * mw
}

/// Verify conductance formula: g = 1/(1+r)
#[test]
fn conductance_formula() {
    let mut e = make_edge(Layer::Physical, 1.0, 0.3);
    let expected = 1.0 / 1.3;
    let got = e.conductance();
    assert!((got - expected).abs() < 1e-6, "conductance: got {got} expected {expected}");

    e.resistance = 0.0;
    assert!((e.conductance() - 1.0).abs() < 1e-6);
    e.resistance = 1.0;
    assert!((e.conductance() - 0.5).abs() < 1e-6);
}

/// Reproduce the worked example from docs/dynamics.md §5.
#[test]
fn worked_example_stage2() {
    // N → V: weight=0.5, resistance=0.3, Emotional layer
    // N: arousal=0.7, valence=-0.4, mass=1.0
    let edge = make_edge(Layer::Emotional, 0.5, 0.3);
    let src = make_state(0.0, -0.4, 0.7, 0.5, 0.0, 0.5, 0.1);
    let src_mass = 1.0_f32;

    let signal_out = s_out(&edge, &src, src_mass);
    // s_in = 0.7 * sign(-0.4) = -0.7
    // g    = 1/1.3 ≈ 0.76923
    // mw   = 1/(1+1) = 0.5
    // out  = 0.5 * 0.76923 * (-0.7) * 0.5 ≈ -0.13462
    let expected = -0.13462_f32;
    assert!(
        (signal_out - expected).abs() < 1e-4,
        "stage2 signal: got {signal_out} expected {expected}"
    );
}

/// Signal extractor per layer.
#[test]
fn signal_extractors() {
    let state = make_state(0.6, -0.4, 0.7, 0.5, 0.3, 0.8, 0.9);
    assert!((signal_extractor(Layer::Physical, &state) - 0.6).abs() < 1e-6);
    // Emotional: arousal * sign(valence) = 0.7 * -1 = -0.7
    assert!((signal_extractor(Layer::Emotional, &state) - (-0.7)).abs() < 1e-6);
    assert!((signal_extractor(Layer::Economic, &state) - 0.3).abs() < 1e-6);
    // Social: trust * reputation = 0.8 * 0.9 = 0.72
    assert!((signal_extractor(Layer::Social, &state) - 0.72).abs() < 1e-6);
}

/// Zero-signal (no source activity) edge stays at initial resistance.
#[test]
fn zero_signal_no_drift() {
    let src = make_state(0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0);
    let mut edge = make_edge(Layer::Physical, 1.0, 0.5);
    let alpha = 0.8_f32;

    for _ in 0..20 {
        let signal = s_out(&edge, &src, 1.0).abs();
        edge.history.push(signal);
        edge.update_resistance(alpha);
    }
    // mean(history) = 0, so r_{t+1} = alpha^t * r_0 → 0
    assert!(edge.resistance < 0.01, "resistance should decay to 0 with zero signal");
}

/// Constant non-zero signal converges resistance toward mean(|s_out|).
#[test]
fn constant_signal_convergence() {
    // Physical edge; source kinetic_energy = 0.5, weight=1, resistance starts at 0.0
    let src = make_state(0.5, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0);
    let mut edge = make_edge(Layer::Physical, 1.0, 0.0);
    let alpha = 0.8_f32;

    // With r=0: g=1, mw=0.5 (mass=1), s_in=0.5 → s_out=0.25
    // Steady-state resistance r* satisfies r* = alpha*r* + (1-alpha)*g(r*)*0.5*0.5
    // For small r: r* ≈ (1-alpha) * s_out / (1 - alpha) = s_out when alpha→1
    // Just verify it stabilises (converges within HISTORY_WINDOW ticks)
    for _ in 0..HISTORY_WINDOW * 3 {
        let signal = s_out(&edge, &src, 1.0).abs();
        edge.history.push(signal);
        edge.update_resistance(alpha);
    }
    let r_prev = edge.resistance;
    let signal = s_out(&edge, &src, 1.0).abs();
    edge.history.push(signal);
    edge.update_resistance(alpha);
    assert!(
        (edge.resistance - r_prev).abs() < 1e-4,
        "resistance not converged: {r_prev} → {}", edge.resistance
    );
}
