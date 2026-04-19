use bevy::prelude::Resource;
use mkm_core::{layer::Layer, params::Params, state::VertexState};

/// Raw per-tick delta produced by a bridge function (before A·D(m) scaling).
#[derive(Default, Clone, Debug)]
pub struct BridgeDelta {
    pub d_kinetic: f32,
    pub d_valence: f32,
    pub d_arousal: f32,
    pub d_flow_rate: f32,
    pub d_resources: f32,
    pub d_trust: f32,
    pub d_reputation: f32,
}

impl BridgeDelta {
    /// Total absolute magnitude — used for energy bookkeeping.
    pub fn magnitude(&self) -> f32 {
        self.d_kinetic.abs()
            + self.d_valence.abs()
            + self.d_arousal.abs()
            + self.d_flow_rate.abs()
            + self.d_resources.abs()
            + self.d_trust.abs()
            + self.d_reputation.abs()
    }
}

/// Per-vertex context passed into each bridge function.
pub struct VertexView<'a> {
    pub state: &'a VertexState,
    pub mass: f32,
    /// Reputation at end of previous tick — used by B_se to detect drops.
    pub prev_reputation: f32,
}

pub trait BridgeFn: Send + Sync {
    fn source_layer(&self) -> Layer;
    fn target_layer(&self) -> Layer;
    /// Returns raw delta (before A·D(m) scaling).
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta;
}

/// Registry of bridge functions. Can be extended at runtime via `register`.
#[derive(Resource)]
pub struct BridgeRegistry {
    bridges: Vec<Box<dyn BridgeFn>>,
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl BridgeRegistry {
    pub fn with_defaults() -> Self {
        let mut r = Self {
            bridges: Vec::new(),
        };
        r.register(Box::new(BridgeEP));
        r.register(Box::new(BridgePE));
        r.register(Box::new(BridgeEC));
        r.register(Box::new(BridgeCE));
        r.register(Box::new(BridgePC));
        r.register(Box::new(BridgeSE));
        r.register(Box::new(BridgeSC));
        r.register(Box::new(BridgeCS));
        r
    }

    /// Register a custom bridge function at runtime.
    pub fn register(&mut self, bridge: Box<dyn BridgeFn>) {
        self.bridges.push(bridge);
    }

    /// Apply all registered bridges; returns the sum of scaled deltas.
    pub fn apply_all(&self, view: &VertexView, params: &Params, amplification: f32) -> BridgeDelta {
        let d_m = mass_damping(view.mass, params.k_mass_damp);
        let scale = amplification * d_m;
        let mut total = BridgeDelta::default();
        for bridge in &self.bridges {
            let raw = bridge.apply(view, params);
            total.d_kinetic += raw.d_kinetic * scale;
            total.d_valence += raw.d_valence * scale;
            total.d_arousal += raw.d_arousal * scale;
            total.d_flow_rate += raw.d_flow_rate * scale;
            total.d_resources += raw.d_resources * scale;
            total.d_trust += raw.d_trust * scale;
            total.d_reputation += raw.d_reputation * scale;
        }
        total
    }

    pub fn bridges(&self) -> &[Box<dyn BridgeFn>] {
        &self.bridges
    }
}

/// D(m) = 1 / (1 + K_mass_damp * (m - 1))
/// High-mass vertices resist bridge perturbations. D(1) = 1, D(5) ≈ 0.33.
pub fn mass_damping(mass: f32, k_mass_damp: f32) -> f32 {
    let denom = 1.0 + k_mass_damp * (mass - 1.0);
    if denom <= 0.0 {
        1.0
    } else {
        1.0 / denom
    }
}

/// A = 1 + (COUPLING_AMPLIFICATION - 1) * level
/// At level=0: A=1. At level=1: A=COUPLING_AMPLIFICATION.
pub fn coupling_amplification(level: f32, coupling_amplification: f32) -> f32 {
    1.0 + (coupling_amplification - 1.0) * level
}

// ── Default bridge implementations ──────────────────────────────────────────

/// B_ep (E→P): arousal spike → kinetic energy. δ_kinetic = K_EP · arousal
/// Prior: affect-driven mobility; arousal increases motion (small, ~1e-2 to 1e-1).
struct BridgeEP;
impl BridgeFn for BridgeEP {
    fn source_layer(&self) -> Layer {
        Layer::Emotional
    }
    fn target_layer(&self) -> Layer {
        Layer::Physical
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        BridgeDelta {
            d_kinetic: params.k_ep * view.state.emotional.arousal,
            ..Default::default()
        }
    }
}

/// B_pe (P→E): physical shock → arousal. δ_arousal = K_PE · kinetic
/// Prior: crowding/stress research; physical disturbance raises emotional arousal (small).
struct BridgePE;
impl BridgeFn for BridgePE {
    fn source_layer(&self) -> Layer {
        Layer::Physical
    }
    fn target_layer(&self) -> Layer {
        Layer::Emotional
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        BridgeDelta {
            d_arousal: params.k_pe * view.state.physical.kinetic_energy,
            ..Default::default()
        }
    }
}

/// B_ec (E→C): fear (high arousal + negative valence) → flow contraction.
/// δ_flow = -K_EC · arousal · max(0, -valence)
/// Prior: behavioural economics; negative affect contracts consumption (small, signed by valence).
struct BridgeEC;
impl BridgeFn for BridgeEC {
    fn source_layer(&self) -> Layer {
        Layer::Emotional
    }
    fn target_layer(&self) -> Layer {
        Layer::Economic
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        let fear = (-view.state.emotional.valence).max(0.0);
        BridgeDelta {
            d_flow_rate: -params.k_ec * view.state.emotional.arousal * fear,
            ..Default::default()
        }
    }
}

/// B_ce (C→E): scarcity/abundance → valence. δ_valence = K_CE · (resources - 0.5)
/// Prior: consumption smoothing; loss-aversion asymmetry (~2× negative branch) (moderate).
struct BridgeCE;
impl BridgeFn for BridgeCE {
    fn source_layer(&self) -> Layer {
        Layer::Economic
    }
    fn target_layer(&self) -> Layer {
        Layer::Emotional
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        BridgeDelta {
            d_valence: params.k_ce * (view.state.economic.resources - 0.5),
            ..Default::default()
        }
    }
}

/// B_pc (P→C): displacement → flow contraction. δ_flow = -K_PC · kinetic
/// Prior: proximity drives transaction volume (gravity models in trade) (moderate ~1e-1).
struct BridgePC;
impl BridgeFn for BridgePC {
    fn source_layer(&self) -> Layer {
        Layer::Physical
    }
    fn target_layer(&self) -> Layer {
        Layer::Economic
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        BridgeDelta {
            d_flow_rate: -params.k_pc * view.state.physical.kinetic_energy,
            ..Default::default()
        }
    }
}

/// B_se (S→E): reputation drop → anxiety. δ_arousal = K_SE · max(0, rep_{t-1} - rep_t)
/// Prior: emotional contagion (Hatfield, Christakis & Fowler); trust inflow damps anxiety (moderate).
struct BridgeSE;
impl BridgeFn for BridgeSE {
    fn source_layer(&self) -> Layer {
        Layer::Social
    }
    fn target_layer(&self) -> Layer {
        Layer::Emotional
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        let rep_drop = (view.prev_reputation - view.state.social.reputation).max(0.0);
        BridgeDelta {
            d_arousal: params.k_se * rep_drop,
            ..Default::default()
        }
    }
}

/// B_sc (S→C): trust gate → flow. δ_flow = K_SC · (trust - TRUST_THRESHOLD)
/// Prior: social-capital literature; trust lowers transaction cost (moderate).
struct BridgeSC;
impl BridgeFn for BridgeSC {
    fn source_layer(&self) -> Layer {
        Layer::Social
    }
    fn target_layer(&self) -> Layer {
        Layer::Economic
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        BridgeDelta {
            d_flow_rate: params.k_sc * (view.state.social.trust - params.trust_threshold),
            ..Default::default()
        }
    }
}

/// B_cs (C→S): flow-rate accumulation → trust. δ_trust = K_CS · flow_rate
/// Prior: reciprocity studies; successful exchange builds trust, failed erodes it faster (moderate).
struct BridgeCS;
impl BridgeFn for BridgeCS {
    fn source_layer(&self) -> Layer {
        Layer::Economic
    }
    fn target_layer(&self) -> Layer {
        Layer::Social
    }
    fn apply(&self, view: &VertexView, params: &Params) -> BridgeDelta {
        BridgeDelta {
            d_trust: params.k_cs * view.state.economic.flow_rate,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use mkm_core::{
        params::Params,
        state::{EconomicState, EmotionalState, PhysicalState, SocialState, VertexState},
    };

    fn make_state(
        kinetic: f32,
        valence: f32,
        arousal: f32,
        resources: f32,
        flow: f32,
        trust: f32,
        rep: f32,
    ) -> VertexState {
        VertexState {
            physical: PhysicalState {
                position: Vec2::ZERO,
                kinetic_energy: kinetic,
            },
            emotional: EmotionalState { valence, arousal },
            economic: EconomicState {
                resources,
                flow_rate: flow,
            },
            social: SocialState {
                reputation: rep,
                hierarchy_rank: 0,
                trust,
            },
        }
    }

    #[test]
    fn mass_damping_at_unit_mass() {
        let d = mass_damping(1.0, 0.5);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn mass_damping_at_high_mass() {
        let d = mass_damping(5.0, 0.5);
        assert!((d - 1.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn coupling_amplification_extremes() {
        let p = Params::default();
        assert!((coupling_amplification(0.0, p.coupling_amplification) - 1.0).abs() < 1e-6);
        assert!(
            (coupling_amplification(1.0, p.coupling_amplification) - p.coupling_amplification)
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn bridge_ep_hand_computed() {
        let p = Params::default();
        let state = make_state(0.0, 0.0, 0.5, 0.5, 0.0, 0.5, 0.0);
        let view = VertexView {
            state: &state,
            mass: 1.0,
            prev_reputation: 0.0,
        };
        let delta = BridgeEP.apply(&view, &p);
        let expected = p.k_ep * 0.5;
        assert!(
            (delta.d_kinetic - expected).abs() < 1e-6,
            "B_ep: got {} expected {}",
            delta.d_kinetic,
            expected
        );
    }

    #[test]
    fn bridge_ec_only_on_negative_valence() {
        let p = Params::default();
        let pos_state = make_state(0.0, 0.5, 0.5, 0.5, 0.0, 0.5, 0.0);
        let pos_view = VertexView {
            state: &pos_state,
            mass: 1.0,
            prev_reputation: 0.0,
        };
        assert_eq!(BridgeEC.apply(&pos_view, &p).d_flow_rate, 0.0);

        let neg_state = make_state(0.0, -0.5, 0.5, 0.5, 0.0, 0.5, 0.0);
        let neg_view = VertexView {
            state: &neg_state,
            mass: 1.0,
            prev_reputation: 0.0,
        };
        let delta = BridgeEC.apply(&neg_view, &p);
        assert!(delta.d_flow_rate < 0.0, "B_ec should decrease flow on fear");
    }

    #[test]
    fn bridge_se_rep_drop() {
        let p = Params::default();
        let state = make_state(0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.3);
        let view = VertexView {
            state: &state,
            mass: 1.0,
            prev_reputation: 0.5,
        };
        let delta = BridgeSE.apply(&view, &p);
        let expected = p.k_se * (0.5 - 0.3);
        assert!((delta.d_arousal - expected).abs() < 1e-6);
    }

    #[test]
    fn bridge_se_no_fire_on_rep_gain() {
        let p = Params::default();
        let state = make_state(0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.7);
        let view = VertexView {
            state: &state,
            mass: 1.0,
            prev_reputation: 0.3,
        };
        let delta = BridgeSE.apply(&view, &p);
        assert_eq!(delta.d_arousal, 0.0);
    }
}
