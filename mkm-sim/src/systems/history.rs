use bevy::prelude::*;
use mkm_core::{id::VertexId, layer::signal_extractor, lifecycle::EdgeLifecycle};
use std::collections::HashMap;

use crate::{
    components::{VId, VMass, VState},
    resources::{EdgeStore, ParamsRes},
};

/// Stage 6: push |s_out| for each active edge into its ring buffer, then
/// recompute resistance via hysteresis rule.
///
/// |s_out| matches what was appended to the target inbox during Stage 2 so
/// resistance tracks actual throughput, not raw source signal.
pub fn history_system(
    params: Res<ParamsRes>,
    sources: Query<(&VId, &VState, &VMass)>,
    mut edges: ResMut<EdgeStore>,
) {
    let alpha = params.0.alpha_resistance;

    let source_map: HashMap<VertexId, (&mkm_core::state::VertexState, f32)> = sources
        .iter()
        .map(|(vid, vs, vm)| (vid.0, (&vs.0, vm.0)))
        .collect();

    for edge in &mut edges.0 {
        if edge.lifecycle != EdgeLifecycle::Active {
            continue;
        }
        let Some(&(src_state, src_mass)) = source_map.get(&edge.source) else {
            continue;
        };
        let s_in = signal_extractor(edge.layer, src_state);
        let g = edge.conductance();
        let mass_weight = src_mass / (1.0 + src_mass);
        let s_out = edge.weight * g * s_in * mass_weight;
        edge.history.push(s_out.abs());
        edge.update_resistance(alpha);
    }
}
