use bevy::prelude::*;
use mkm_core::events::{EventTarget, LayerDelta};

use crate::{
    components::{VId, VState},
    resources::{EventQueueRes, SimClock},
};

pub fn ingest_system(
    clock: Res<SimClock>,
    mut event_queue: ResMut<EventQueueRes>,
    mut query: Query<(&VId, &mut VState)>,
) {
    let due = event_queue.0.drain_tick(clock.tick);
    if due.is_empty() {
        return;
    }
    for event in due {
        match event.target {
            EventTarget::Global => {
                for (_, mut vs) in query.iter_mut() {
                    apply_delta(&mut vs, &event.delta);
                }
            }
            EventTarget::Vertex(vid) => {
                for (id, mut vs) in query.iter_mut() {
                    if id.0 == vid {
                        apply_delta(&mut vs, &event.delta);
                        break;
                    }
                }
            }
        }
    }
}

fn apply_delta(vs: &mut VState, delta: &LayerDelta) {
    vs.0.physical.kinetic_energy =
        (vs.0.physical.kinetic_energy + delta.physical).clamp(0.0, 1.0);
    vs.0.emotional.valence = (vs.0.emotional.valence + delta.emotional).clamp(-1.0, 1.0);
    vs.0.economic.resources = (vs.0.economic.resources + delta.economic).clamp(0.0, 1.0);
    vs.0.social.trust = (vs.0.social.trust + delta.social).clamp(0.0, 1.0);
}
