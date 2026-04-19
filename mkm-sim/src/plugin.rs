use bevy::prelude::*;
use mkm_core::{
    events::EventQueue,
    params::{Params, SimConfig},
};

use crate::{
    init::init_world,
    resources::{
        EdgeStore, EventQueueRes, ParamsRes, SimClock, SimConfigRes, SimRngRes, TickMetrics,
    },
    rng::SimRng,
    systems::{
        history::history_system, ingest::ingest_system, output::output_system,
        propagate::propagate_system,
    },
    tick::TickStage,
};

pub struct MkmSimPlugin {
    pub config: SimConfig,
    pub params: Params,
}

impl Plugin for MkmSimPlugin {
    fn build(&self, app: &mut App) {
        let mut rng = SimRng::from_seed(self.config.seed);
        let init = init_world(&self.config, &self.params, &mut rng);

        // Insert resources
        app.insert_resource(SimConfigRes(self.config.clone()));
        app.insert_resource(ParamsRes(self.params.clone()));
        app.insert_resource(SimRngRes(rng));
        app.insert_resource(SimClock::new());
        app.insert_resource(EventQueueRes(EventQueue::new()));
        app.insert_resource(EdgeStore(init.edges));
        app.init_resource::<TickMetrics>();

        // Spawn vertex entities
        for bundle in init.vertex_bundles {
            app.world_mut().spawn(bundle);
        }

        // Order the 7 tick stages
        app.configure_sets(
            Update,
            (
                TickStage::Ingest,
                TickStage::Propagate,
                TickStage::Bridges,
                TickStage::Crisis,
                TickStage::Plasticity,
                TickStage::History,
                TickStage::Output,
            )
                .chain(),
        );

        app.add_systems(Update, ingest_system.in_set(TickStage::Ingest));
        app.add_systems(Update, propagate_system.in_set(TickStage::Propagate));
        app.add_systems(Update, history_system.in_set(TickStage::History));
        app.add_systems(Update, output_system.in_set(TickStage::Output));
    }
}
