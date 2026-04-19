use bevy::prelude::*;
use mkm_core::lifecycle::VertexLifecycle;

use crate::{
    bridge_registry::{coupling_amplification, BridgeRegistry, VertexView},
    components::{
        VCoupling, VInbox, VLifecycle, VMass, VPendingEnergyCost, VPrevReputation, VState,
    },
    resources::ParamsRes,
};

/// Stage 3: intra-layer updates then cross-layer bridge cascade.
///
/// Order per vertex:
///   1. Aggregate inbox signals per layer.
///   2. Apply intra-layer updates (before cross-layer bridges).
///   3. Apply all bridge functions scaled by A·D(m).
///   4. Clamp state to valid ranges.
///   5. Accumulate bridge-activity cost in VPendingEnergyCost (deducted in Stage 5).
///   6. Update VPrevReputation for next tick's B_se.
///   7. Clear inbox.
pub fn bridges_system(
    params: Res<ParamsRes>,
    registry: Res<BridgeRegistry>,
    mut query: Query<(
        &mut VState,
        &VMass,
        &VCoupling,
        &mut VInbox,
        &mut VPrevReputation,
        &mut VPendingEnergyCost,
        &VLifecycle,
    )>,
) {
    let p = &params.0;

    for (mut vstate, vmass, vcoupling, mut vinbox, mut vprev_rep, mut vpending, vlifecycle) in
        &mut query
    {
        if matches!(vlifecycle.0, VertexLifecycle::Zombie { .. }) {
            vinbox.0.clear();
            continue;
        }

        let agg_p = vinbox.0.sum_physical();
        let agg_e = vinbox.0.sum_emotional();
        let agg_c = vinbox.0.sum_economic();
        let agg_s = vinbox.0.sum_social();

        let state = &mut vstate.0;
        let dt = p.dt;

        // ── Intra-layer updates ─────────────────────────────────────────────
        let d_kinetic = p.k_kin * agg_p - p.friction * state.physical.kinetic_energy;
        // Position update: direction of aggregate physical signal × kinetic × dt.
        // agg_p is scalar; use sign as 1D proxy for direction along x-axis.
        let d_pos_x = agg_p.signum() * state.physical.kinetic_energy * dt;

        let d_valence = p.k_val * agg_e - p.decay_val * state.emotional.valence;
        let d_arousal = p.k_ar * agg_e.abs() - p.decay_ar * state.emotional.arousal;

        let d_flow = p.k_flow * agg_c - p.decay_flow * state.economic.flow_rate;
        let d_resources = state.economic.flow_rate * dt;

        let d_trust = p.k_tr * agg_s - p.decay_tr * (state.social.trust - 0.5);
        let d_reputation = p.k_rep * agg_s;

        state.physical.kinetic_energy += d_kinetic;
        state.physical.position.x += d_pos_x;
        state.emotional.valence += d_valence;
        state.emotional.arousal += d_arousal;
        state.economic.flow_rate += d_flow;
        state.economic.resources += d_resources;
        state.social.trust += d_trust;
        state.social.reputation += d_reputation;

        // ── Cross-layer bridge cascade ──────────────────────────────────────
        let a = coupling_amplification(vcoupling.0.level, p.coupling_amplification);
        let view = VertexView {
            state,
            mass: vmass.0,
            prev_reputation: vprev_rep.0,
        };
        let delta = registry.apply_all(&view, p, a);

        state.physical.kinetic_energy += delta.d_kinetic;
        state.emotional.valence += delta.d_valence;
        state.emotional.arousal += delta.d_arousal;
        state.economic.flow_rate += delta.d_flow_rate;
        state.economic.resources += delta.d_resources;
        state.social.trust += delta.d_trust;
        state.social.reputation += delta.d_reputation;

        // ── Clamp to valid ranges ───────────────────────────────────────────
        state.clamp_all();

        // ── Energy bookkeeping (Invariant 5) ────────────────────────────────
        let bridge_cost = p.bridge_activity_cost * delta.magnitude();
        vpending.0 += bridge_cost;

        // ── Update prev-reputation for next tick's B_se ─────────────────────
        vprev_rep.0 = state.social.reputation;

        vinbox.0.clear();
    }
}
