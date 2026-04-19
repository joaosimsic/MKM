## 12. Parameter Reference

Every tunable in one place. Defaults shown; scenarios may override via config.

**Naming conventions:**
* `UPPER_CASE` — thresholds, window sizes, and physical-meaning constants (e.g. `YIELD_POINT`, `COLLAPSE_WINDOW`, `TRUST_THRESHOLD`). Interpret these as *structural limits*; they are unlikely to move during calibration.
* `K_xx` — gain constants on specific bridge or intra-layer updates (e.g. `K_EP`, `K_VAL`). These are the **primary calibration targets** (see Section 16).
* Greek letters (`α`, `β`, `φ_c`) — ratios, smoothing factors, or critical fractions from the literature. Bounded in $[0, 1]$ except where noted.

| Parameter | Default | Range | Meaning |
|---|---|---|---|
| `dt` | 1.0 | (0, ∞) | Tick duration (abstract units) |
| `HISTORY_WINDOW` | 64 | [1, 512] | Edge ring-buffer size (ticks) |
| `R_max` | 1.0 | (0, ∞) | Maximum edge resistance |
| `YIELD_POINT` | 0.9 | (0, R_max] | Resistance at which an edge snaps |
| `α_resistance` | 0.8 | [0, 1] | Resistance smoothing factor (hysteresis) |
| `TENSION_THRESHOLD` | 0.15 | (0, 1] | Max-layer-delta above which shear is computed |
| `SHEAR_COST_FACTOR` | 0.1 | [0, ∞) | Per-tick energy cost multiplier for shear |
| `COLLAPSE_THRESHOLD` | 0.4 | (0, 1] | $C_\mu$ fraction flagging layer collapse |
| `COLLAPSE_WINDOW` | 64 | [1, 1024] | Tick window over which $C_\mu$ is measured |
| `φ_c` | 0.5 | (0, 1) | LCC fraction below which Shatter fires |
| `MIN_EDGES` | 3 | [0, ∞) | Min active edges before Weave search runs |
| `BASE_COST` | 0.05 | [0, 1] | Baseline energy cost of edge creation |
| `PHYSICAL_MULTIPLIER` | 3.0 | [1, ∞) | Extra cost for creating $M_p$ edges |
| `COUPLING_AMPLIFICATION` | 2.0 | [1, ∞) | Bridge-function gain at `level = 1.0` |
| `MASS_DELTA_MAX` | 0.01 | (0, 1] | Max per-tick mass change |
| `ZOMBIE_DECAY` | 0.99 | (0, 1] | Per-tick state decay factor for Zombies |
| `ENERGY_CAPACITY` | 1.0 | (0, ∞) | Default per-vertex energy capacity |
| `ENERGY_REGEN_RATE` | 0.01 | [0, 1] | Per-tick energy regeneration |
| `BRIDGE_ACTIVITY_COST` | 0.02 | [0, 1] | Cost multiplier for total bridge delta magnitude |
| `K_mass` | 0.001 | [0, 1] | Mass drift gain from influence + throughput |
| `K_mass_damp` | 0.5 | [0, ∞) | Mass damping coefficient in bridge output |
| `STRESS_GAIN` | 4.0 | (0, ∞) | Logistic steepness for coupling update |
| `STRESS_MIDPOINT` | 0.5 | [0, 1] | Stress value at which coupling = 0.5 |
| `STRESS_EMA_β` | 0.9 | [0, 1] | EMA smoothing factor for coupling.level |
| `w_shear` | 0.4 | [0, 1] | Stress weight on mean shear |
| `w_collapse` | 0.4 | [0, 1] | Stress weight on max layer collapse rate |
| `w_resources` | 0.2 | [0, 1] | Stress weight on resource deficit (1 - mean resources) |
| `FRICTION` | 0.1 | [0, 1] | Per-tick kinetic energy decay |
| `DECAY_val` | 0.05 | [0, 1] | Valence mean-reversion rate |
| `DECAY_ar` | 0.1 | [0, 1] | Arousal decay rate |
| `DECAY_flow` | 0.1 | [0, 1] | Flow-rate decay rate |
| `DECAY_tr` | 0.02 | [0, 1] | Trust mean-reversion rate (toward 0.5) |
| `K_KIN` | 0.5 | [0, ∞) | Intra-layer $M_p$ inbox gain |
| `K_VAL` | 0.5 | [0, ∞) | Intra-layer valence inbox gain |
| `K_AR` | 0.3 | [0, ∞) | Intra-layer arousal inbox gain |
| `K_FLOW` | 0.5 | [0, ∞) | Intra-layer flow-rate inbox gain |
| `K_TR` | 0.2 | [0, ∞) | Intra-layer trust inbox gain |
| `K_REP` | 0.3 | [0, ∞) | Intra-layer reputation inbox gain |
| `K_EP` | 0.4 | [0, ∞) | $B_{ep}$ gain (Emotion → Physical) |
| `K_PE` | 0.2 | [0, ∞) | $B_{pe}$ gain (Physical → Emotional) |
| `K_EC` | 0.3 | [0, ∞) | $B_{ec}$ gain (Emotion → Economic; fear) |
| `K_CE` | 0.3 | [0, ∞) | $B_{ce}$ gain (Economic → Emotional; scarcity/abundance) |
| `K_PC` | 0.2 | [0, ∞) | $B_{pc}$ gain (Physical → Economic) |
| `K_SE` | 0.5 | [0, ∞) | $B_{se}$ gain (Social → Emotional) |
| `K_SC` | 0.3 | [0, ∞) | $B_{sc}$ gain (Social → Economic) |
| `K_CS` | 0.2 | [0, ∞) | $B_{cs}$ gain (Economic → Social; flow → trust) |
| `TRUST_THRESHOLD` | 0.4 | [0, 1] | Trust floor below which $B_{sc}$ becomes inhibitive |

---

## 13. Configuration, CLI & Data Output

### Configuration Format (TOML)
```toml
[simulation]
seed          = 42
dt            = 1.0
max_ticks     = 10_000
parallelism   = "deterministic"   # or "parallel"

[initial_conditions]
n_vertices        = 10_000
edge_density      = 0.005
spatial_extent    = [0.0, 100.0]
init_distribution = "uniform"     # "uniform" | "clustered" | "power_law"

[parameters]
# Any override from Section 12
yield_point    = 0.9
history_window = 64

[[events]]
tick   = 500
kind   = "TensorImpact"
target = { scope = "global" }
impact = { physical = 0.0, emotional = 0.3, economic = -0.8, social = 0.0 }

[output]
metrics_path      = "out/metrics.parquet"
snapshot_path     = "out/snapshots/"
snapshot_interval = 100
log_level         = "info"
```

### CLI Specification (`mkm-cli`)
* `mkm-cli run <config.toml>` — headless simulation run.
* `mkm-cli replay <snapshot>` — resume from snapshot.
* `mkm-cli scenario <preset>` — run a named preset.
* `mkm-cli sweep <config.toml> --param <name> --range <lo>..<hi>:<step> [--seeds <N>]` — parameter sweep for resilience analysis. With `--seeds N` (default 32), each parameter point is run $N$ times with distinct seeds derived from the base seed; output includes per-point median and IQR for every metric. Single-seed results are not acceptable for Predictive Engineering claims.
* `mkm-cli inspect <snapshot>` — print summary stats and invariant checks.

### Output Schema
* **Per-tick metrics (Parquet columns):** `tick`, `sim_time`, `n_active_vertices`, `n_zombies`, `n_edges_per_layer[4]`, `mean_shear`, `max_shear`, `c_mu_per_layer[4]`, `coupling_mean`, `lcc_per_layer[4]`, `shatter_flag`.
* **Event log (JSONL):** one line per Snap / Weave / Zombie / Re-entry / Shatter event with `tick`, vertex/edge IDs, and relevant state.
* **Snapshots (MessagePack):** full mesh state at configurable intervals — used for replay, debugging, and resilience analysis.

---

