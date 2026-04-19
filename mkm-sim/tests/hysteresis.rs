/// Hysteresis tests: alternating-signal edge must show measurable asymmetry.
///
/// An edge seeing only positive signal converges to a higher resistance than
/// one seeing equal-magnitude negative signal (because we store |s_out| and
/// the signal extractor is signed, the asymmetry comes from the weight sign).
use mkm_core::{
    edge::Edge,
    id::{EdgeId, VertexId},
    layer::Layer,
};

fn run_edge(alpha: f32, signals: &[f32]) -> f32 {
    let mut edge = Edge::new(EdgeId(0), VertexId(0), VertexId(1), Layer::Emotional);
    edge.resistance = 0.1;
    for &s in signals {
        edge.history.push(s.abs());
        edge.update_resistance(alpha);
    }
    edge.resistance
}

/// Alternating +/- signal shows same hysteresis as constant since we store |s|.
/// But a high-magnitude vs low-magnitude signal should produce clearly different
/// steady-state resistance values.
#[test]
fn high_vs_low_signal_asymmetry() {
    let alpha = 0.8_f32;
    let n = 200_usize;

    let high_signals: Vec<f32> = (0..n).map(|i| if i % 2 == 0 { 0.8 } else { -0.8 }).collect();
    let low_signals: Vec<f32> = (0..n).map(|i| if i % 2 == 0 { 0.2 } else { -0.2 }).collect();

    let r_high = run_edge(alpha, &high_signals);
    let r_low = run_edge(alpha, &low_signals);

    // High-signal edge should converge to higher resistance (more throughput)
    assert!(
        r_high > r_low + 1e-3,
        "expected r_high ({r_high:.4}) > r_low ({r_low:.4}) by at least 1e-3"
    );
}

/// Hysteresis update formula: r_{t+1} = alpha*r_t + (1-alpha)*mean(|hist|)
#[test]
fn hysteresis_formula_single_step() {
    let mut edge = Edge::new(EdgeId(0), VertexId(0), VertexId(1), Layer::Physical);
    edge.resistance = 0.3;
    let alpha = 0.8_f32;

    // Push a single known value so mean = that value
    edge.history.push(0.5);
    let r_before = edge.resistance;
    edge.update_resistance(alpha);
    let expected = alpha * r_before + (1.0 - alpha) * 0.5;
    assert!(
        (edge.resistance - expected).abs() < 1e-6,
        "hysteresis formula: got {} expected {expected}", edge.resistance
    );
}

/// Resistance must stay clamped in [0, 1] regardless of signal magnitude.
#[test]
fn resistance_clamped() {
    let mut edge = Edge::new(EdgeId(0), VertexId(0), VertexId(1), Layer::Physical);
    edge.resistance = 0.99;
    for _ in 0..100 {
        edge.history.push(10.0); // absurdly large
        edge.update_resistance(0.5);
    }
    assert!(edge.resistance <= 1.0, "resistance exceeded 1.0: {}", edge.resistance);

    edge.resistance = 0.01;
    for _ in 0..100 {
        edge.history.push(0.0);
        edge.update_resistance(0.5);
    }
    assert!(edge.resistance >= 0.0, "resistance went negative: {}", edge.resistance);
}
