# The Multidimensional Kinetic Mesh (MKM) — Project Guide

Quick reference for navigating the MKM simulation project.

## What is MKM?

A simulation of societal resilience as a **four-layer graph** (physical, emotional, economic, social) over a shared vertex set. Cross-layer bridges couple the layers; structural plasticity rewires topology under stress. Target: find the *Mathematical Minimum* — the smallest initial allocation of trust and resources needed to recover from a given shock.

**Key goal:** reproducible, deterministic pipeline producing resilience heatmaps whose qualitative shape is stable across seed ensembles.

**Scale target:** $10^6$ vertices on a single workstation.

## Essential Reading (In Order)

1. **Start here:** `docs/model.md` — glossary, vertex, four layers, state schema, invariants.
2. **Dynamics:** `docs/dynamics.md` — edge mechanics, crisis metrics, tick pipeline, bridge functions.
3. **Scenarios:** `docs/scenarios.md` — shocks, resilience logic, Mathematical Minimum definition.
4. **Engineering:** `docs/engineering.md` — crate workspace, tech stack, viz, hardware optimization.
5. **Roadmap:** `docs/roadmap.md` — nine phases from scaffolding to predictive engineering.

## Other References

- **Theoretical foundations:** `docs/theory.md` — multiplex networks, percolation, complex adaptive systems.
- **Parameters:** `docs/parameters.md` — complete lookup table; defaults, ranges, TOML config, CLI reference.
- **Roadmap:** `docs/roadmap.md` — phase deliverables and acceptance criteria.
- **Verification:** `docs/verification.md` — five-tier test strategy, calibration, stylized facts, sensitivity analysis.
- **Design decisions:** `docs/design.md` — resolved decisions and design principles.

## Quick Facts

- **Four layers:** $M_p$ (Physical), $M_e$ (Emotional), $M_c$ (Economic), $M_s$ (Social).
- **Vertex:** one agent exposing state in every layer simultaneously; mass drifts with influence.
- **Edge:** active filter with memory; resistance via hysteresis on signal history; can snap (prune) or weave (create).
- **Tick pipeline:** 7 strictly ordered stages (ingest → propagate → bridges → crisis metrics → plasticity → history → output).
- **Bridge functions:** 8 default cross-layer transfers (emotion→physical, scarcity→emotional, etc.); **these are the model** — their coefficients are the primary calibration targets.
- **Energy budget:** per-vertex resource; structural events (snap/weave) and bridge activity cost energy; simulation respects conservation accounting.
- **Coupling:** global state transitions from Loose to Tight under stress (shear + collapse); amplifies bridge outputs during crisis.
- **Zombie:** vertex with zero edges; decays in place until a new edge restores it. Retains decayed state on re-entry.

## Key Concepts & Terminology

- **Shear ($S_\mu$):** horizontal displacement of a vertex's state across layers; increases energy cost; triggers tight coupling.
- **Collapse ($C_\mu$):** rate of edge pruning in a layer; layer enters `Collapsed` when rate exceeds threshold.
- **Shatter Point:** global event when any layer's largest connected component falls below 50% of vertices.
- **The Snap:** edge-pruning event (resistance exceeds yield point).
- **The Weave:** edge-creation event (triggered by tight coupling + low degree; candidates found via spatial or trust lookup).
- **Mathematical Minimum:** the minimum initial trust/resources required for 90% of seeds to recover 80% of connectivity within a fixed horizon after a shock.

## Workspace Structure

Four Rust crates under a root workspace:

- **`mkm-core`** — types, math, state schema (no behavior).
- **`mkm-sim`** — tick pipeline, systems, Bevy ECS plugin.
- **`mkm-viz`** — (deferred) 3D rendering plugin.
- **`mkm-cli`** — headless runner, scenario loader, data export.

## Design Philosophy

1. **Data-oriented.** State laid out for cache locality and SIMD.
2. **Bridge functions are first-class.** Swappable, testable, parameterized.
3. **Determinism is non-negotiable.** Same seed → bit-identical output (under `parallelism = "deterministic"`).
4. **Energy is accounting, not conservation.** Bridges can create energy across layers; what's tracked is per-vertex cost.
5. **Zombies have memory.** Isolated vertices decay in place but retain state on re-entry — no reset.

## Implementation Roadmap

Nine phases: Phase 0 (scaffolding) → Phase 1–5 (core sim) → Phase 6 (performance) → Phase 7 (GPU) → Phase 8–9 (viz + predictive engineering).

See `docs/roadmap.md` for detailed milestones and acceptance criteria.

## Calibration Gateway

Before any Mathematical-Minimum number is reported:

1. Bridge coefficients must pass sign & magnitude **priors** (Section 10, §16.1).
2. Must reproduce **two stylized facts** from outside the project (financial contagion, Granovetter thresholds).
3. Global **sensitivity analysis** identifies load-bearing coefficients.
4. Ensemble **IQR ≤ 20%** of median.

See `docs/verification.md` for the full pipeline.

## Running the Sim

```bash
# Build all crates
cargo build --release -p mkm-cli

# Run a scenario
mkm-cli scenario economic_shock

# Run a config file
mkm-cli run config.toml

# Parameter sweep (resilience analysis)
mkm-cli sweep config.toml --param initial_trust_mean --range 0.3..0.8:0.05 --seeds 32

# Inspect a snapshot
mkm-cli inspect snapshots/tick_1000.msgpack
```

See `docs/parameters.md` for full CLI reference and config syntax.

## Testing Tiers

1. **Unit tests** — pure functions, signal extractors, bridge mechanics.
2. **Integration tests** — tick ordering, determinism, invariants.
3. **Property tests** — state clamping, edge properties.
4. **Scenario / regression tests** — full simulations; qualitative outcomes vs. golden.
5. **Benchmarks** — throughput at scale (edges_10k, full_tick_100k, etc.).

See `docs/verification.md` for detailed test suite and determinism contract.

## Key Files (Phase 0–5)

- `mkm-core/src/state.rs` — vertex/edge state definitions.
- `mkm-sim/src/tick.rs` — seven-stage pipeline definition.
- `mkm-sim/src/systems/bridges.rs` — bridge function registry and application.
- `mkm-sim/src/systems/crisis.rs` — shear, collapse, coupling, shatter.
- `mkm-sim/src/systems/plasticity.rs` — snap, weave, zombie mechanics.

## Non-Goals (Explicitly Out of Scope)

- Sub-tick temporal resolution.
- More than four layers (v2 only).
- Heterogeneous vertex roles (v2 only).
- Epistemic fifth layer (v2 only).
- Multi-machine distributed sim.
- Online learning / neural bridges (v2 only).

---

**Next step:** Read `docs/model.md` to nail down terminology and architecture, then `docs/dynamics.md` for the tick pipeline and bridges.