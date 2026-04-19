use bevy::prelude::*;
use mkm_core::{id::VertexId, layer::Layer, layer::signal_extractor, lifecycle::EdgeLifecycle};
use std::collections::HashMap;

use crate::{
    components::{VId, VInbox, VMass, VState},
    resources::EdgeStore,
};

/// Stage 2: extract signal from each active edge source, weight by conductance
/// and mass, then append to target inbox.
pub fn propagate_system(
    edges: Res<EdgeStore>,
    sources: Query<(&VId, &VState, &VMass)>,
    mut targets: Query<(&VId, &mut VInbox)>,
) {
    // source_id → (state, mass)
    let source_map: HashMap<VertexId, (&mkm_core::state::VertexState, f32)> = sources
        .iter()
        .map(|(vid, vs, vm)| (vid.0, (&vs.0, vm.0)))
        .collect();

    // target_id → Vec<(layer, signal)>
    let mut pending: HashMap<VertexId, Vec<(Layer, f32)>> = HashMap::new();

    for edge in &edges.0 {
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
        pending.entry(edge.target).or_default().push((edge.layer, s_out));
    }

    for (vid, mut vinbox) in &mut targets {
        let Some(writes) = pending.get(&vid.0) else { continue };
        for &(layer, signal) in writes {
            match layer {
                Layer::Physical => vinbox.0.physical.push(signal),
                Layer::Emotional => vinbox.0.emotional.push(signal),
                Layer::Economic => vinbox.0.economic.push(signal),
                Layer::Social => vinbox.0.social.push(signal),
            }
        }
    }
}
