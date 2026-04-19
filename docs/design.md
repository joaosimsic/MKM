## 17. Resolved Design Decisions

These were resolved during doc consolidation. Any change to these requires a version bump and migration notes.

1. **Embedding space for shear.** Linear per-layer normalization to $[0, 1]^n$, then L2 distance averaged over $\binom{4}{2} = 6$ layer pairs. Learned embedding is a v2 consideration.
2. **Bridge function composition.** **Sum**, not chain. Multiple registered bridges with the same target layer have their deltas summed before amplification and mass damping. Chaining requires an explicit scenario override.
3. **Zombie re-entry decay model.** Exponential: `state *= ZOMBIE_DECAY^n` and `mass *= ZOMBIE_DECAY^n` where `n = tick - decay_since`. Position is frozen; scalar state and mass both decay. Influence atrophies — a long-dormant Zombie re-enters with diminished weight on its surviving neighbours.
4. **Edge directionality.** All edges directed. Undirected relationships modeled as a pair `(A→B, B→A)`. Memory doubling is accepted for model clarity.
5. **Parallel determinism cost.** Runtime flag `parallelism = "deterministic" | "parallel"`. Deterministic mode uses sequential reductions; parallel mode uses Rayon with non-associative float ops.
6. **Mass dynamics.** Specified in Section 2 (Mass Dynamics): $\Delta m = K_{\text{mass}} \cdot (\text{social\_influence} + \text{economic\_throughput})$, clamped to `MASS_DELTA_MAX` per tick. **No longer blocks Phase 1.**
7. **Spatial semantics for $M_p$.** Euclidean in a bounded 2D region defined by `spatial_extent`. Toroidal semantics available via a scenario-level `spatial_topology = "torus"` override (not default).
8. **Event atomicity.** Same-tick events applied in stable sort order by `(tick, insertion_order)`. `EventQueue` preserves push order within a tick.
9. **Snapshot cadence at $10^6$ nodes.** Snapshot interval tuned in Phase 6 profiling; initial heuristic: `snapshot_interval ≥ n_vertices / 10^4` ticks (i.e., at $10^6$ nodes, no more than once per 100 ticks). Disk bandwidth budget revisited when Parquet + MessagePack writers are benchmarked.
10. **Tie-breaking in Weave candidate selection.** When multiple candidates have equal score, pick the lowest `VertexId`. Deterministic.
11. **Initial mass distribution.** Gaussian $\mathcal{N}(1.0, 0.2)$ clamped to $[0.1, 10]$.
12. **Signal extraction per layer.** Defined in Section 5: $M_p \to$ kinetic_energy, $M_e \to$ arousal·sign(valence), $M_c \to$ flow_rate, $M_s \to$ trust·reputation.

---

## 18. Design Notes & Principles

> [!IMPORTANT]
> **Bridge Functions are the model.** The simulation succeeds or fails based on the quality of the **Bridge Functions** that determine exactly how Emotional weight transforms into Physical movement. Every other piece is scaffolding. Make them first-class: swappable, testable, parameterized.

> [!IMPORTANT]
> **Edge creation costs more in $M_p$ than in $M_s$.** Physical logistics are harder than social introductions — the model must reflect this via `PHYSICAL_MULTIPLIER`.

> [!IMPORTANT]
> **The Dark Side: Predictive Engineering.** The point of this simulation is not to watch collapse — it is to find the **Mathematical Minimum** of trust and resources needed for resilience. Model the floor, not the ceiling.

> [!IMPORTANT]
> **Shear is a stressed state, not a default.** Perfectly vertical alignment allows 2D spatial partitioning (Quadtrees). Shear forces 3D search (Octrees), which is computationally expensive. Keep shear conditional.

> [!NOTE]
> **Non-Goals (explicitly out of scope):**
> - Sub-tick temporal resolution.
> - More than four layers in the base model.
> - Individual-agent psychological realism (agents are simple state vectors).
> - **Heterogeneous vertex roles.** All vertices share one schema and one set of parameters. Real societies have asymmetric roles (producers, consumers, brokers, sinks); these are left to emerge from topology and mass dynamics rather than be stamped in as types. A `VertexRole` enum modulating bridge gains per-vertex is a v2 consideration — it would at minimum require per-role calibration, which multiplies the Section 16 validation burden.
> - Real-world geographic data ingestion.
> - Multi-machine distributed simulation.
> - Online learning / neural bridge functions (v2 consideration).
> - Epistemic / informational fifth layer $M_i$ (v2; see Section 1).

