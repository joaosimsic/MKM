# MKM Implementation Checklist

> Hand-off document for step-by-step implementation of the Multidimensional Kinetic Mesh.
> **This file is the single entry point** — read `CLAUDE.md` for project context, then work phase-by-phase from here.

## How to use this file

1. Read `CLAUDE.md` first (project overview, design philosophy, quick facts).
2. Pick the earliest phase that is not fully checked off. **Do not skip phases** — dependencies are strict.
3. For each phase: read the **Pre-reading** docs, create the files under **Files to create**, work the **Tasks** in listed order, and confirm every **Acceptance gate** passes before moving on.
4. Before reporting any *Mathematical Minimum* number, satisfy the **Calibration Gateway** (end of this file).

## Status snapshot (as of 2026-04-19)

- **Code:** Phase 0 complete. Workspace builds and tests pass.
- **Docs:** 10 markdown files under `docs/` + this file + `CLAUDE.md`.
- **Next action:** Phase 1, task 1 (define `VertexId`, `EdgeId`, `Layer` enum).

---

## Phase 0 — Project Scaffolding

**Goal:** Compilable Rust workspace with all dependencies pinned.
**Depends on:** nothing.
**Pre-reading:** `docs/engineering.md` (workspace layout, tech stack), `docs/parameters.md` (Cargo features).

### Files to create

- `/home/joao/proj/MKM/Cargo.toml` (workspace root; members: `mkm-core`, `mkm-sim`, `mkm-viz`, `mkm-cli`)
- `/home/joao/proj/MKM/rust-toolchain.toml` (pin stable toolchain)
- `/home/joao/proj/MKM/.github/workflows/ci.yml`
- `/home/joao/proj/MKM/mkm-core/Cargo.toml` + `src/lib.rs`
- `/home/joao/proj/MKM/mkm-sim/Cargo.toml` + `src/lib.rs`
- `/home/joao/proj/MKM/mkm-viz/Cargo.toml` + `src/lib.rs`
- `/home/joao/proj/MKM/mkm-cli/Cargo.toml` + `src/main.rs`

### Tasks

- [x] Define root workspace `Cargo.toml` with shared profile settings (LTO, `codegen-units = 1` for release).
- [x] Pin toolchain in `rust-toolchain.toml`.
- [x] Add Bevy, Rayon, glam, serde, toml, rand_chacha, thiserror as workspace dependencies.
- [x] Create four crate skeletons with empty `lib.rs` / `main.rs`.
- [x] Write CI workflow: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`.
- [x] Verify clean-checkout build locally.

### Acceptance gates

- [x] `cargo build --workspace` succeeds.
- [x] `cargo test --workspace` passes (empty suite OK).
- [ ] CI green on first push. *(pending remote push)*

---

## Phase 1 — Core Data Model & Tick Loop

**Goal:** Spawn vertices with per-layer state; step time with empty dynamics; deterministic RNG; TOML config load.
**Depends on:** Phase 0.
**Pre-reading:** `docs/model.md` (glossary, state schema, invariants 1–7), `docs/dynamics.md` §1–§3 (tick pipeline stages), `docs/parameters.md` (Params struct).

### Files to create

**`mkm-core/src/`:** `id.rs`, `layer.rs`, `state.rs`, `vertex.rs`, `edge.rs`, `inbox.rs`, `coupling.rs`, `energy.rs`, `lifecycle.rs`, `ringbuffer.rs`, `params.rs`, `invariants.rs`, `math.rs`, `events.rs`, `snapshot.rs`.

**`mkm-sim/src/`:** `tick.rs`, `rng.rs`, `init.rs`, `systems/ingest.rs`, `systems/output.rs`, `scenarios.rs`.

**`mkm-cli/src/`:** `config.rs`, `commands/run.rs`.

**Tests:** `mkm-core/tests/serde_roundtrip.rs`, `mkm-core/tests/invariants_static.rs`, `mkm-sim/tests/determinism.rs`.

### Tasks

- [ ] Define `VertexId`, `EdgeId`, `Layer` enum (Physical, Emotional, Economic, Social).
- [ ] Define per-layer state structs + composite `VertexState` (Section 3.A of `docs/model.md`).
- [ ] Define `Edge` with ring-buffer history (`HISTORY_WINDOW` ticks).
- [ ] Define `Params` with serde + TOML deserialize; validate ranges.
- [ ] Implement `Invariants 1–7` as predicate functions in `invariants.rs`.
- [ ] Implement `ChaCha20Rng` resource in `rng.rs` with `fork(label)` for per-system streams.
- [ ] Implement `init.rs` with three `init_distribution` modes per `docs/model.md` §3.B.
- [ ] Define 7 ordered `SystemSet`s in `tick.rs` (ingest → propagate → bridges → crisis → plasticity → history → output). Stage bodies empty.
- [ ] Per-tick output summary to stdout + JSONL.
- [ ] `mkm-cli run config.toml` loads TOML → builds Bevy app → runs `max_ticks` → exits cleanly.

### Acceptance gates

- [ ] 10K vertices initialized; state distribution matches `docs/model.md` §3.B within tolerance.
- [ ] `sim_time` advances exactly `dt` per tick.
- [ ] Snapshot round-trip (save → load → save) bit-identical.
- [ ] Determinism: 3 runs with same seed → identical SHA-256 of final snapshot.
- [ ] Tick rate ≥ 1000/s at 10K vertices (no dynamics).
- [ ] Invariants 1, 2, 3, 7 hold throughout.

---

## Phase 2 — Edge Mechanics (Pondered Edges)

**Goal:** Edges filter/transform signals; resistance via hysteresis on history.
**Depends on:** Phase 1.
**Pre-reading:** `docs/dynamics.md` §1 (signals per layer), §2 (conductance, categories, hysteresis).

### Files to create

- `mkm-sim/src/systems/propagate.rs` (Stage 2)
- `mkm-sim/src/systems/history.rs` (Stage 6)
- Expand `mkm-core/src/edge.rs` (conductance, category, hysteresis helpers)
- Tests: `mkm-sim/tests/propagate_numeric.rs`, `mkm-sim/tests/hysteresis.rs`
- Bench: `mkm-sim/benches/edges_100k.rs`

### Tasks

- [ ] Per-layer canonical signal extractors (kinetic energy, arousal·sign(valence), flow_rate, trust·reputation).
- [ ] `conductance(history, params)` returning filter gain.
- [ ] Propagation: signal × conductance → target vertex inbox (additive).
- [ ] History stage: push new sample, recompute hysteresis, advance ring buffer tail.
- [ ] Ensure Stage 4 (crisis) reads empty inboxes — ordering invariant.

### Acceptance gates

- [ ] Two-vertex constant-signal chain converges to analytic resistance within 1% in ≤ `HISTORY_WINDOW` ticks.
- [ ] Alternating-signal test shows measurable hysteresis asymmetry at 1e-3 precision.
- [ ] Invariants 1–4 hold at 10K ticks with 1% random `TensorImpact` injections.
- [ ] Bench: 100K edges ≥ 100 ticks/s.

---

## Phase 3 — Kinetic Cascades & Bridge Functions ⚠️ CRITICAL

**Goal:** Cross-layer bleed via eight default bridges. **Model fidelity depends entirely on these.**
**Depends on:** Phase 2.
**Pre-reading:** `docs/dynamics.md` §3 (bridge table, 8 defaults, coupling amplification), `docs/verification.md` §16.1 (sign priors).

### Files to create

- `mkm-sim/src/bridge_registry.rs` (trait `BridgeFn`, registry keyed by `(source, target)` layer pair)
- `mkm-sim/src/systems/bridges.rs` (Stage 3)
- Expand `mkm-core/src/energy.rs` (deduct, regen, can_afford)
- Tests: `mkm-sim/tests/bridges_numeric.rs`, `mkm-sim/tests/energy_bookkeeping.rs`, `mkm-sim/tests/mass_damping.rs`

### Tasks

- [ ] Define `BridgeFn` trait: `apply(&self, source_state, target_state, coupling_level, params) -> Delta`.
- [ ] Register all 8 default bridges per `docs/dynamics.md` §3 table.
- [ ] Apply intra-layer updates, then bridges, with `A·D(m)` modulation (influence mass damping).
- [ ] Multiply bridge outputs by `1 + COUPLING_AMPLIFICATION * coupling_level`.
- [ ] Deduct bridge-activity energy from per-vertex budget.
- [ ] Clear inboxes at stage end.
- [ ] Allow runtime bridge registration (no recompile for custom bridges).

### Acceptance gates

- [ ] Each of the 8 default bridges reproduces a hand-computed delta at a fixed input within 1e-4.
- [ ] Emotional spike produces proportional Mp, Mc response within 3 ticks, ≤ 5% analytic error.
- [ ] `coupling_level = 1.0` amplifies bridge output by exactly `COUPLING_AMPLIFICATION`.
- [ ] Energy bookkeeping (Invariant 5) holds within 1e-5 tolerance.
- [ ] Custom bridge registration at runtime works.
- [ ] Invariants 1–5 hold at 10K ticks with random shocks every 100 ticks.

---

## Phase 4 — Structural Plasticity

**Goal:** Topology evolves: Snap (prune), Weave (create), Zombie (decay + re-entry with memory).
**Depends on:** Phase 3.
**Pre-reading:** `docs/dynamics.md` §4 (plasticity stage ordering), `docs/model.md` (zombie semantics).

### Files to create

- `mkm-sim/src/systems/plasticity.rs` (Stage 5)
- `mkm-sim/src/spatial/quadtree.rs` (rebuild per tick, `query_radius`)
- `mkm-sim/src/spatial/octree.rs` (stub; activated Phase 6)
- Expand `mkm-sim/src/systems/output.rs` (event-log JSONL writer)
- Tests: `mkm-sim/tests/plasticity.rs`, `mkm-sim/tests/zombie_reentry.rs`

### Tasks

- [ ] Stage 5 ordering: regen → Snap → Zombie-check → Weave → cost-deduct.
- [ ] Snap: resistance > `YIELD_POINT` for N ticks → prune edge, emit event.
- [ ] Zombie: vertex with zero edges → mark Zombie, decay state by `ZOMBIE_DECAY` per tick, retain on re-entry.
- [ ] Weave: tight coupling + low degree → spatial/trust lookup → create edge with post-decay state preserved.
- [ ] Emit structured events (JSONL) for every Snap/Weave/Zombie transition.

### Acceptance gates

- [ ] Edge with resistance > `YIELD_POINT` enters Strained then Snaps within 2 ticks.
- [ ] Isolated vertex becomes Zombie within 1 tick; state decays at `ZOMBIE_DECAY` per tick.
- [ ] Weave to a Zombie restores Active with *post-decay* state (no reset).
- [ ] Tight-coupling cluster produces ≥ 3× Weave events vs. baseline.
- [ ] Invariant 4 (no dangling edges) holds at 10K ticks.
- [ ] Event log parses cleanly with a JSONL validator.

---

## Phase 5 — Crisis Metrics

**Goal:** Quantitative stress (shear, collapse, shatter) feeding back into global coupling.
**Depends on:** Phase 4.
**Pre-reading:** `docs/dynamics.md` §5 (shear, collapse, shatter, coupling feedback), `docs/theory.md` (percolation).

### Files to create

- `mkm-sim/src/systems/crisis.rs` (Stage 4)
- `mkm-sim/src/metrics.rs` (`CollapseCounter<N>`, union-find, rolling EMA)
- Expand `mkm-core/src/coupling.rs` (`update(stress, params)`)
- Tests: `mkm-sim/tests/shear.rs`, `mkm-sim/tests/collapse.rs`, `mkm-sim/tests/percolation.rs`

### Tasks

- [ ] Compute `S_μ` (shear) conditionally per vertex.
- [ ] Rolling-window `C_μ` (collapse rate per layer); flag `Collapsed` when exceeded.
- [ ] Largest-connected-component tracking (union-find); `Shatter` flag when LCC < 50%.
- [ ] Coupling transitions Loose → Tight under `shear + collapse` pressure.
- [ ] Feed coupling back into Phase-3 bridge amplification next tick.

### Acceptance gates

- [ ] Hand-computed `S_μ` matches implementation within 1e-4.
- [ ] `C_μ` triggers flag at threshold under synthetic edge-deletion test.
- [ ] Percolation Shatter at φc = 0.5 ± 0.05 averaged over 10 seeds.
- [ ] End-to-end feedback loop demonstrable: shock → shear → coupling tightens → amplified bridge outputs.
- [ ] Invariants 1–7 all hold.

---

## Phase 6 — Performance & Scale

**Goal:** Scale from 10³ → 10⁵–10⁶ nodes; CPU parallelism; baseline benchmarks.
**Depends on:** Phase 5.
**Pre-reading:** `docs/engineering.md` (parallelism strategy, hardware budget), `docs/verification.md` (determinism contract).

### Files to create

- Refactor `mkm-sim/src/systems/propagate.rs` to CSR layout (`row_ptr`, `col_idx`, `data`)
- Rayon parallelism in `mkm-sim/src/systems/bridges.rs` with thread-local shards
- Complete `mkm-sim/src/spatial/octree.rs` (3D fallback when `S_μ > 3× TENSION_THRESHOLD`)
- `mkm-sim/benches/full_tick_100k.rs`, `mkm-sim/benches/full_tick_1m.rs`
- `profiling/flamegraph.sh`, `profiling/notes.md`

### Tasks

- [ ] Convert adjacency to CSR; keep deterministic ordering.
- [ ] Add `parallelism = "deterministic" | "parallel"` switch in `Params`.
- [ ] Rayon-parallel bridges with per-thread delta shards, deterministic reduce.
- [ ] Profile (flamegraph); document top 5 hotspots.
- [ ] Commit baseline numbers at 10³, 10⁴, 10⁵ nodes.

### Acceptance gates

- [ ] Baselines committed for 10³, 10⁴, 10⁵.
- [ ] Tick rate ≥ 30/s at 10⁵ nodes (reference CPU, parallel mode).
- [ ] Deterministic mode bit-identical to single-threaded.
- [ ] Parallel mode matches deterministic within 1e-4 at tick 1000.

---

## Phase 7 — GPU Acceleration

**Goal:** Offload bulk propagate + intra-layer bridges to WGSL compute shaders.
**Depends on:** Phase 6.
**Pre-reading:** `docs/engineering.md` (GPU backend, wgpu, VRAM budget).

### Files to create

- `mkm-sim/src/gpu/mod.rs`, `gpu/shaders.rs`, `gpu/buffers.rs`, `gpu/dispatch.rs`
- `mkm-sim/shaders/propagate.wgsl`, `shaders/bridges.wgsl`
- Tests: `mkm-sim/tests/gpu_parity.rs`

### Tasks

- [ ] Upload CSR + state buffers to GPU.
- [ ] Implement propagate + intra-layer bridges in WGSL.
- [ ] Dispatch per tick; copy deltas back.
- [ ] Parity test: 1000 CPU vs. GPU ticks, diff < 1e-4.

### Acceptance gates

- [ ] CPU vs. GPU output matches within 1e-4 per state variable.
- [ ] ≥ 5× throughput on edge-heavy workloads.
- [ ] 10⁶ nodes feasible at ≥ 10 ticks/s (reference CPU + GPU).
- [ ] VRAM ≤ 8 GB at max scale.

---

## Phase 8 — 3D Visualization

**Goal:** Real-time rendering: vertices, edges, pillars, HUD, LOD throttling.
**Depends on:** Phase 7 (or Phase 6 in parallel at small scale).
**Pre-reading:** `docs/engineering.md` (viz strategy, render primitives).

### Files to create

- `mkm-viz/src/render/vertices.rs`, `render/edges.rs`, `render/pillars.rs`
- `mkm-viz/src/views/top_down.rs`, `views/profile.rs`
- `mkm-viz/src/throttle.rs` (LOD: mesh → point-cloud in collapsed zones)
- `mkm-viz/src/overlay.rs` (HUD: shear, coupling, LCC, shatter flag)
- `mkm-viz/tests/visual_regression.rs` (SHA-256 hash of golden frames)

### Tasks

- [ ] Instanced primitive rendering for vertices + edges.
- [ ] Two camera modes (top-down, profile).
- [ ] LOD throttling driven by crisis state (collapsed zones degrade to point cloud).
- [ ] HUD overlay reads live metrics from sim.
- [ ] Visual-regression test using golden frame hashes.

### Acceptance gates

- [ ] 60 FPS at 10⁴ nodes, full mesh.
- [ ] 30 FPS at 10⁵ nodes with throttling.
- [ ] Regression hash match or SSIM ≥ 0.99.

---

## Phase 9 — Scenarios & Predictive Engineering

**Goal:** Resilience analysis tool; four preset scenarios; parameter sweep + Mathematical Minimum computation.
**Depends on:** all prior phases + Calibration Gateway passed.
**Pre-reading:** `docs/scenarios.md` (all four scenarios + MM definition), `docs/verification.md` §16 (calibration pipeline), `docs/parameters.md` (CLI reference).

### Files to create

- Expand `mkm-sim/src/scenarios.rs` (Economic Shock, Social Fragmentation, Crisis Escalation, Mutual Aid Formation)
- `mkm-cli/src/commands/scenario.rs`, `sweep.rs`, `inspect.rs`, `replay.rs`
- `mkm-cli/src/export/parquet.rs`, `export/jsonl.rs`, `export/msgpack.rs`
- `analysis/` (Python + polars notebooks for heatmap generation)

### Tasks

- [ ] Implement the four preset scenario factories.
- [ ] `mkm-cli scenario <name>` runs a scenario.
- [ ] `mkm-cli sweep config.toml --param X --range A..B:step --seeds N` runs ensemble sweeps.
- [ ] `find_minimum()` binary-searches Mathematical Minimum per `docs/scenarios.md` §16–17.
- [ ] `mkm-cli inspect snapshots/tick_*.msgpack` reports invariant violations.
- [ ] `mkm-cli replay` produces bit-identical continuation from snapshot.
- [ ] Analysis notebooks produce resilience heatmaps.

### Acceptance gates

- [ ] Four scenarios produce expected qualitative outcomes (documented in `docs/scenarios.md`).
- [ ] Parameter sweep reproduces Shatter @ φc with N ≥ 32 seeds; IQR reported.
- [ ] `find_minimum` produces MM with ensemble IQR ≤ 20% of median.
- [ ] `mkm-cli inspect` flags invariant violations correctly.
- [ ] `mkm-cli replay` output bit-identical to original continuation.
- [ ] End-to-end documented workflow: config → run → analysis → report.

---

## Calibration Gateway

**No Mathematical-Minimum number may be reported until all four gates pass.** See `docs/verification.md` §16 for full protocol.

- [ ] **Priors (§16.1):** every bridge coefficient has a cited sign and magnitude prior from outside literature.
- [ ] **Stylized Fact SF-1:** reproduces financial contagion cascade pattern at N ≥ 32 seeds.
- [ ] **Stylized Fact SF-2:** reproduces Granovetter threshold dynamics at N ≥ 32 seeds.
- [ ] **Sensitivity analysis:** per-bridge elasticity ηk computed; `|ηk| > 1` coefficients flagged as load-bearing and cited.
- [ ] **Ensemble stability:** reported MM has IQR ≤ 20 % of median.

---

## Cross-reference: docs → phases

| Doc                        | Phases it covers                                              |
| -------------------------- | ------------------------------------------------------------- |
| `docs/model.md`            | Phase 1 (state, invariants, init), Phases 2–5 (state transitions) |
| `docs/dynamics.md`         | Phase 2 (edges), Phase 3 (bridges), Phase 4 (plasticity), Phase 5 (crisis) |
| `docs/scenarios.md`        | Phase 9 (scenarios, Mathematical Minimum)                     |
| `docs/engineering.md`      | Phase 0 (workspace), Phase 6 (parallelism), Phase 7 (GPU), Phase 8 (viz) |
| `docs/parameters.md`       | Phase 0 (features), Phase 1 (Params + TOML), Phase 9 (CLI)    |
| `docs/verification.md`     | All phases (test tiers), Phase 5+ (calibration)               |
| `docs/design.md`           | All phases (resolved decisions + non-goals)                   |
| `docs/theory.md`           | Background for Phase 3 (bridges), Phase 5 (percolation)       |
| `docs/roadmap.md`          | Primary source for this checklist — read if anything here is unclear |

---

## Ordering dependencies at a glance

```
Phase 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7
                                    ↘
                                     8 (can overlap 6 at small scale)
Phase 5 → 9 (requires Calibration Gateway passed)
```

**Within-phase ordering:**

- Phase 1: `params.rs` + `state.rs` before `tick.rs`.
- Phase 3: `bridge_registry.rs` before `systems/bridges.rs`.
- Phase 4: `spatial/quadtree.rs` before `systems/plasticity.rs` (Weave needs spatial lookup).
- Phase 5: `metrics.rs` before `systems/crisis.rs`.
- Phase 6: CSR layout before Rayon parallelism.
- Phase 7: WGSL shaders parallel with `buffers.rs` + `dispatch.rs`.
- Phase 9: scenario presets register before `sweep.rs` calls `find_minimum`.

---

## Glossary

For all terminology (Shear, Collapse, Snap, Weave, Zombie, Shatter Point, Mathematical Minimum, Coupling states), see `docs/model.md` §1 and `CLAUDE.md` "Key Concepts & Terminology".
