use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Params {
    pub dt: f32,
    pub history_window: usize,
    pub r_max: f32,
    pub yield_point: f32,
    pub alpha_resistance: f32,
    pub tension_threshold: f32,
    pub shear_cost_factor: f32,
    pub collapse_threshold: f32,
    pub collapse_window: usize,
    pub phi_c: f32,
    pub min_edges: usize,
    pub base_cost: f32,
    pub physical_multiplier: f32,
    pub coupling_amplification: f32,
    pub mass_delta_max: f32,
    pub zombie_decay: f32,
    pub energy_capacity: f32,
    pub energy_regen_rate: f32,
    pub bridge_activity_cost: f32,
    pub k_mass: f32,
    pub k_mass_damp: f32,
    pub stress_gain: f32,
    pub stress_midpoint: f32,
    pub stress_ema_beta: f32,
    pub w_shear: f32,
    pub w_collapse: f32,
    pub w_resources: f32,
    pub friction: f32,
    pub decay_val: f32,
    pub decay_ar: f32,
    pub decay_flow: f32,
    pub decay_tr: f32,
    pub k_kin: f32,
    pub k_val: f32,
    pub k_ar: f32,
    pub k_flow: f32,
    pub k_tr: f32,
    pub k_rep: f32,
    pub k_ep: f32,
    pub k_pe: f32,
    pub k_ec: f32,
    pub k_ce: f32,
    pub k_pc: f32,
    pub k_se: f32,
    pub k_sc: f32,
    pub k_cs: f32,
    pub trust_threshold: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            dt: 1.0,
            history_window: 64,
            r_max: 1.0,
            yield_point: 0.9,
            alpha_resistance: 0.8,
            tension_threshold: 0.15,
            shear_cost_factor: 0.1,
            collapse_threshold: 0.4,
            collapse_window: 64,
            phi_c: 0.5,
            min_edges: 3,
            base_cost: 0.05,
            physical_multiplier: 3.0,
            coupling_amplification: 2.0,
            mass_delta_max: 0.01,
            zombie_decay: 0.99,
            energy_capacity: 1.0,
            energy_regen_rate: 0.01,
            bridge_activity_cost: 0.02,
            k_mass: 0.001,
            k_mass_damp: 0.5,
            stress_gain: 4.0,
            stress_midpoint: 0.5,
            stress_ema_beta: 0.9,
            w_shear: 0.4,
            w_collapse: 0.4,
            w_resources: 0.2,
            friction: 0.1,
            decay_val: 0.05,
            decay_ar: 0.1,
            decay_flow: 0.1,
            decay_tr: 0.02,
            k_kin: 0.5,
            k_val: 0.5,
            k_ar: 0.3,
            k_flow: 0.5,
            k_tr: 0.2,
            k_rep: 0.3,
            k_ep: 0.4,
            k_pe: 0.2,
            k_ec: 0.3,
            k_ce: 0.3,
            k_pc: 0.2,
            k_se: 0.5,
            k_sc: 0.3,
            k_cs: 0.2,
            trust_threshold: 0.4,
        }
    }
}

impl Params {
    /// Validate that all parameters are within their legal ranges.
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=10.0).contains(&self.dt) {
            return Err(format!("dt={} out of range (0, 10]", self.dt));
        }
        if self.history_window == 0 {
            return Err("history_window must be > 0".into());
        }
        if self.r_max <= 0.0 {
            return Err(format!("r_max={} must be > 0", self.r_max));
        }
        if !(0.0..=self.r_max).contains(&self.yield_point) {
            return Err(format!("yield_point={} out of [0, r_max]", self.yield_point));
        }
        if !(0.0..=1.0).contains(&self.alpha_resistance) {
            return Err(format!("alpha_resistance={} out of [0, 1]", self.alpha_resistance));
        }
        if !(0.0..=1.0).contains(&self.collapse_threshold) {
            return Err(format!("collapse_threshold={} out of [0, 1]", self.collapse_threshold));
        }
        if !(0.0..=1.0).contains(&self.phi_c) {
            return Err(format!("phi_c={} out of [0, 1]", self.phi_c));
        }
        if self.energy_capacity <= 0.0 {
            return Err(format!("energy_capacity={} must be > 0", self.energy_capacity));
        }
        if self.energy_regen_rate < 0.0 {
            return Err(format!("energy_regen_rate={} must be >= 0", self.energy_regen_rate));
        }
        if !(0.0..=1.0).contains(&self.zombie_decay) {
            return Err(format!("zombie_decay={} out of [0, 1]", self.zombie_decay));
        }
        if !(0.0..=1.0).contains(&self.stress_ema_beta) {
            return Err(format!("stress_ema_beta={} out of [0, 1]", self.stress_ema_beta));
        }
        Ok(())
    }
}

// ── InitDistribution ────────────────────────────────────────────────────────

/// How to distribute vertex positions and correlated scalar states on init.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InitDistribution {
    #[default]
    Uniform,
    Clustered,
    PowerLaw,
}

// ── SimConfig ────────────────────────────────────────────────────────────────

/// Flat simulation configuration derived from `FullConfig`. Stored as a Bevy
/// Resource via a newtype wrapper in mkm-sim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimConfig {
    pub seed: u64,
    pub max_ticks: u64,
    pub n_vertices: usize,
    pub edge_density: f32,
    /// Maximum coordinate value; vertices are placed in [0, spatial_extent]².
    pub spatial_extent: f32,
    pub init_distribution: InitDistribution,
    pub snapshot_interval: u64,
    pub output_path: Option<String>,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            max_ticks: 1000,
            n_vertices: 1000,
            edge_density: 0.005,
            spatial_extent: 100.0,
            init_distribution: InitDistribution::Uniform,
            snapshot_interval: 100,
            output_path: None,
        }
    }
}

// ── TOML full-config structs ─────────────────────────────────────────────────

/// Raw TOML `[simulation]` section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SimSettings {
    pub seed: Option<u64>,
    pub max_ticks: Option<u64>,
    pub n_vertices: Option<usize>,
    pub edge_density: Option<f32>,
    pub spatial_extent: Option<f32>,
    pub init_distribution: Option<InitDistribution>,
}

/// Raw TOML `[initial_conditions]` section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InitConditions {
    /// Optional events to inject at tick 0.
    pub events: Option<Vec<serde_json::Value>>,
}

/// Raw TOML `[output]` section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    pub snapshot_interval: Option<u64>,
    pub output_path: Option<String>,
}

/// Top-level TOML config file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FullConfig {
    pub simulation: SimSettings,
    pub initial_conditions: Option<InitConditions>,
    pub output: Option<OutputConfig>,
    pub params: Option<Params>,
}

impl FullConfig {
    /// Load a `FullConfig` from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&text)?;
        Ok(cfg)
    }
}

impl From<&FullConfig> for SimConfig {
    fn from(fc: &FullConfig) -> Self {
        let defaults = SimConfig::default();
        let sim = &fc.simulation;
        let (snap_interval, output_path) = fc
            .output
            .as_ref()
            .map(|o| {
                (
                    o.snapshot_interval.unwrap_or(defaults.snapshot_interval),
                    o.output_path.clone(),
                )
            })
            .unwrap_or((defaults.snapshot_interval, None));

        SimConfig {
            seed: sim.seed.unwrap_or(defaults.seed),
            max_ticks: sim.max_ticks.unwrap_or(defaults.max_ticks),
            n_vertices: sim.n_vertices.unwrap_or(defaults.n_vertices),
            edge_density: sim.edge_density.unwrap_or(defaults.edge_density),
            spatial_extent: sim.spatial_extent.unwrap_or(defaults.spatial_extent),
            init_distribution: sim
                .init_distribution
                .clone()
                .unwrap_or(defaults.init_distribution),
            snapshot_interval: snap_interval,
            output_path,
        }
    }
}
