use std::{path::PathBuf, process};

use bevy::prelude::*;
use mkm_core::params::{FullConfig, SimConfig};
use mkm_sim::plugin::MkmSimPlugin;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("run") => {
            let path = args.get(2).unwrap_or_else(|| {
                eprintln!("usage: mkm-cli run <config.toml>");
                process::exit(1);
            });
            cmd_run(PathBuf::from(path));
        }
        _ => {
            eprintln!("usage: mkm-cli run <config.toml>");
            process::exit(1);
        }
    }
}

fn cmd_run(config_path: PathBuf) {
    let full_cfg = FullConfig::from_file(&config_path).unwrap_or_else(|e| {
        eprintln!("failed to load config: {e}");
        process::exit(1);
    });

    let sim_config = SimConfig::from(&full_cfg);
    let params = full_cfg.params.clone().unwrap_or_default();

    if let Err(e) = params.validate() {
        eprintln!("invalid params: {e}");
        process::exit(1);
    }

    let max_ticks = sim_config.max_ticks;

    let mut app = App::new();
    app.add_plugins(MkmSimPlugin { config: sim_config, params });

    for _ in 0..max_ticks {
        app.update();
    }
}
