## 9. Implementation Strategy

### Design Philosophy
**Data-Oriented Design.** State is laid out for cache locality and SIMD friendliness. Behavior lives in systems that operate on contiguous arrays of state.

### Tech Stack
* **Language:** Rust (safety, zero-cost abstractions, predictable performance).
* **ECS:** Bevy.
    * Built-in scheduler, plugin architecture, and rendering pipeline.
    * `SystemSet` ordering maps naturally to the tick pipeline (Section 5).
    * Runs headless during early phases; rendering plugin activates later.
    * Legion was considered but rejected because it would require gluing in `wgpu` and a custom scheduler separately.
* **Parallelism:** Rayon for CPU-side reductions; Bevy's parallel system scheduling for non-conflicting systems.
* **GPU:** `wgpu` compute shaders (WGSL) for bulk edge and cascade math.
* **Math:** `glam` (chosen for Bevy integration over `nalgebra`).
* **Serialization:** `serde` + MessagePack for snapshots; CSV/Parquet for metric export.
* **RNG:** `rand` + `rand_chacha` (seeded, deterministic).

### Crate Workspace
Four crates under a root `Cargo.toml` workspace. Each listed file is created during its referenced phase (see Section 14).

#### `mkm-core` — types, math primitives, layer definitions, state schema (no behavior)
```
src/
  lib.rs                 # re-exports; no logic
  id.rs                  # VertexId (u64), EdgeId (u64), Tick (u64)
  layer.rs               # LayerKind enum, Projection trait, signal extractors
  state.rs               # PhysicalState, EmotionalState, EconomicState, SocialState
  vertex.rs              # Vertex component; mass clamp helpers
  edge.rs                # Edge struct, EdgeLifecycle, conductance()
  inbox.rs               # Inbox component (per-layer SmallVec<f32;8>)
  coupling.rs            # CouplingState; level/ema_state update helpers
  energy.rs              # EnergyBudget; regen, can_afford(), deduct()
  lifecycle.rs           # LifecycleState (Active | Strained | Zombie)
  ringbuffer.rs          # RingBuffer<T; N> for edge history
  params.rs              # Params struct mirroring Section 12; TOML derive
  invariants.rs          # check_* functions for Invariants 1-7
  math.rs                # sigmoid, clamp, lerp, projection helpers
  events.rs              # TensorImpact, EventTarget, EventQueue type
  snapshot.rs            # Serde wrappers; MessagePack encode/decode stubs
tests/
  serde_roundtrip.rs
  invariants_static.rs
```

#### `mkm-sim` — tick pipeline, systems, event injection
```
src/
  lib.rs                 # Bevy plugin registering all systems
  tick.rs                # SystemSet ordering for the 7 stages
  rng.rs                 # ChaCha20Rng resource; per-system fork helpers
  systems/
    ingest.rs            # Stage 1: drain events, apply TensorImpact
    propagate.rs         # Stage 2: edge signal extraction + inbox writes
    bridges.rs           # Stage 3: intra-layer + cross-layer; BridgeRegistry
    crisis.rs            # Stage 4: S_μ, C_μ, LCC, Shatter, coupling update
    plasticity.rs        # Stage 5: energy regen, Snap, Weave, Zombie
    history.rs           # Stage 6: ring-buffer append + resistance hysteresis
    output.rs            # Stage 7: per-tick metrics, snapshot flush
  bridge_registry.rs     # trait BridgeFn; default_bridges(); register()
  spatial/
    quadtree.rs          # 2D spatial index for M_p neighbor queries
    octree.rs             # 3D fallback for sheared vertices
    index.rs             # SpatialIndex trait; per-tick rebuild
  metrics.rs             # running aggregates, EMA windows
  scenarios.rs           # preset definitions loaded by name
  init.rs                # initial vertex/edge sampling per Section 3.B
tests/
  tick_order.rs
  bridges_numeric.rs
  determinism.rs
  plasticity.rs
benches/
  edges_10k.rs
  full_tick_100k.rs
```

#### `mkm-viz` — Bevy rendering plugin (deferred; stubbed until Phase 8)
```
src/
  lib.rs                 # VizPlugin; no-op until Phase 8
  render/
    vertices.rs          # sphere/point primitives per vertex, per layer
    edges.rs             # line primitives; color/thickness mapping
    pillars.rs           # vertical connectors across layer Z-offsets
  views/
    top_down.rs          # XY camera, layer isolation
    profile.rs           # XZ camera, pillar stress visualization
  throttle.rs            # LOD switch: mesh → point cloud in collapsed zones
  overlay.rs             # HUD with shear/coupling/LCC readouts
```

#### `mkm-cli` — headless runner, scenario loader, data export
```
src/
  main.rs                # clap-based arg parsing; dispatches to commands
  commands/
    run.rs               # mkm-cli run <config.toml>
    replay.rs            # mkm-cli replay <snapshot>
    scenario.rs          # mkm-cli scenario <preset>
    sweep.rs             # mkm-cli sweep <config> --param ... --range ...
    inspect.rs           # mkm-cli inspect <snapshot>
  config.rs              # TOML parse → Params + scenario spec
  export/
    parquet.rs           # per-tick metrics writer
    jsonl.rs             # event-log writer
    msgpack.rs           # snapshot writer (via mkm-core::snapshot)
  report.rs              # summary stats printing for `inspect`
```

#### Workspace root
```
Cargo.toml               # workspace members, shared profile settings
rust-toolchain.toml      # pinned stable toolchain
.github/workflows/ci.yml # fmt, clippy, test, bench-smoke
README.md                # build & run
docs/                    # this file lives here as core.md
```

---

## 10. 3D Visualization Strategy
* **Volumetric Stacking:** each layer rendered at a fixed Z-offset. Vertices appear as points/spheres at their per-layer XY position.
* **Pillars:** vertical lines connecting a vertex's four layer projections. Healthy → perfectly vertical; stressed → diagonal, with color mapped to $S_\mu$ magnitude.
* **Edge Rendering:** thickness ∝ weight; color ∝ resistance (green → yellow → red → snapped). Strained edges pulse.
* **Diagnostic Views:**
    * **Top-down (XY):** clustering and edge-density per layer. Layer selector to isolate one plane.
    * **Profile (XZ):** detection of **Tight Coupling** (aligned oscillations) and **Shear** (bent pillars).
* **Crisis Throttling:** collapsed zones render as point-cloud primitives instead of mesh geometry to preserve frame rate.

---

## 11. Hardware-Specific Optimization
* **CPU (16 threads):** non-linear vertex logic, structural plasticity, coupling transitions.
* **GPU (8GB VRAM):** edge weight updates, cascade propagation, batch shear math.
* **Memory budget:**
    * ~32GB RAM reserved for edge history buffers at $10^6$ nodes with 64-tick windows.
    * VRAM at $10^6$ nodes: ~256MB vertex state + ~1.6GB edges (avg degree 50) + headroom for render.
* **Crisis Throttling:** LOD degradation (point clouds) in collapsed zones preserves frame rate during high-entropy events.
* **2D vs. 3D partitioning:** Quadtrees in normal state; Octree fallback activated per-vertex only when $S_\mu$ exceeds threshold.

---

