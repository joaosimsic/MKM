use std::collections::HashMap;

use bevy::prelude::*;
use mkm_core::params::{Params, SimConfig};
use mkm_sim::{
    components::{VId, VState},
    plugin::MkmSimPlugin,
    resources::SimClock,
};

fn run_sim(seed: u64, ticks: u64) -> HashMap<u64, (f32, f32, f32, f32)> {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        n_vertices: 50,
        edge_density: 0.01,
        ..Default::default()
    };
    let params = Params::default();

    let mut app = App::new();
    app.add_plugins(MkmSimPlugin { config, params });

    for _ in 0..ticks {
        app.update();
    }

    let mut result = HashMap::new();
    let world = app.world_mut();
    for (id, state) in world.query::<(&VId, &VState)>().iter(world) {
        let s = &state.0;
        result.insert(
            id.0 .0,
            (
                s.physical.kinetic_energy,
                s.emotional.valence,
                s.economic.resources,
                s.social.trust,
            ),
        );
    }
    result
}

#[test]
fn same_seed_identical_output() {
    let r1 = run_sim(42, 10);
    let r2 = run_sim(42, 10);
    assert_eq!(r1.len(), r2.len());
    for (id, v1) in &r1 {
        let v2 = r2[id];
        assert_eq!(*v1, v2, "vertex {id} differs between runs");
    }
}

#[test]
fn different_seeds_differ() {
    let r1 = run_sim(1, 5);
    let r2 = run_sim(2, 5);
    // At minimum the initial state should differ (different seeds → different positions/states)
    let differs = r1
        .iter()
        .any(|(id, v1)| r2.get(id) != Some(v1));
    assert!(differs, "different seeds should produce different states");
}

#[test]
fn sim_time_advances() {
    let config = SimConfig {
        seed: 7,
        n_vertices: 10,
        edge_density: 0.01,
        ..Default::default()
    };
    let ticks = 5u64;
    let params = Params::default();
    let dt = params.dt;

    let mut app = App::new();
    app.add_plugins(MkmSimPlugin { config, params });

    for _ in 0..ticks {
        app.update();
    }

    let clock = app.world().resource::<SimClock>();
    assert_eq!(clock.tick, ticks);
    assert!((clock.sim_time - dt * ticks as f32).abs() < 1e-5);
}
