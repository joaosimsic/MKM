# Project: The Multidimensional Kinetic Mesh (MKM)

## Overview

**What this is.** MKM simulates a society as a four-layer graph (physical, emotional, economic, social) over a shared vertex set, with cross-layer bridge functions coupling the layers and a structural plasticity loop that rewires topology under stress. Target scale: $10^6$ vertices on a single workstation.

**Why it exists.** To find the *Mathematical Minimum* — the smallest initial allocation of trust and resources from which a population recovers after a given shock. The goal is not pretty collapse dynamics; it is a constructive floor on societal resilience (see Section 18).

**What success looks like.** A reproducible, deterministic pipeline that (a) recovers known resilience boundaries from the literature (percolation, contagion thresholds) as a calibration check, and (b) produces resilience heatmaps over parameter sweeps whose qualitative shape is stable across seed ensembles.

---

## Preamble — Glossary & Terminology

Consistent terminology is critical. Use these terms exactly as defined; do not introduce synonyms without updating this glossary.

* **Agent / Vertex / Node** — informally interchangeable, but in this document:
    * **Agent:** the conceptual entity being modeled (a person, actor, unit).
    * **Vertex:** the canonical simulation object representing an agent. One ECS entity with state in every layer.
    * **Node:** avoid — use "vertex."
* **Layer / Mesh / Plane** — informally interchangeable, but:
    * **Layer:** canonical. One of the four dimensions ($M_p$, $M_e$, $M_c$, $M_s$).
    * **Mesh:** reserved for the entire multi-layer structure as a whole.
* **Edge / Connection:** **Edge** is canonical. An edge belongs to exactly one layer and connects two vertices within that layer.
* **Pillar:** the vertical construct formed by a single vertex's projections across all layers. Shear is measured along the pillar.
* **Bridge Function ($B_{xy}$):** a transfer function between layers *within a single vertex*. Not to be confused with **The Weave** (edge creation event).
* **The Snap:** a single edge-pruning event.
* **The Weave:** a single edge-creation event. (Formerly called "The Bridge" — renamed to avoid collision with Bridge Function.)
* **The Shatter Point:** the global event where any layer's largest connected component falls below the critical fraction.
* **Tick:** one discrete simulation step. Advances `sim_time` by `dt`.
* **Zombie:** a vertex with zero active edges, retained in memory for possible re-entry.

---

## 1. Core Architecture: The Vertex & The Stack
The model treats a society or system as a **Temporal Adaptive Tensor** — a four-layer stack of graphs evolving in discrete time.

### The Vertex (Vertical Axis)
* **Identity:** one agent is one vertex, exposing state in every layer simultaneously. The vertex is the integrative unit.
* **Non-Uniformity:** a vertex's observable state differs per layer. A neighbor in $M_s$ sees reputation and trust; a neighbor in $M_p$ sees position.
* **Variable Mass:** `mass: f32` modulates how strongly the vertex resists bridge-function perturbations and how much its signal dominates its edges. Mass drifts slowly with accumulated influence.

### The Multidimensional Stack ($M_x$)
Four layers, each a distinct graph over the same vertex set:
* **$M_p$ (Physical):** position, kinetic energy, spatial logistics.
* **$M_e$ (Emotional):** valence, arousal, affective state.
* **$M_c$ (Economic):** resources, flow rate, scarcity.
* **$M_s$ (Social):** reputation, hierarchy rank, trust.

Layers are deliberately coarse. Adding a fifth layer is a breaking change — it requires extending every bridge function and the state schema.

**Why these four (and not five).** The layers were chosen so that (a) each has a well-studied empirical literature (percolation for $M_p$, contagion for $M_e$, flow networks for $M_c$, trust networks for $M_s$) usable for calibration, and (b) they are causally non-redundant — no layer can be derived from the others by a local rule. An *epistemic / informational* fifth layer (what agents believe, independent of how they feel) is the strongest candidate for addition but is intentionally omitted: belief formation is folded into `SocialState.trust` and `SocialState.reputation` in v1, on the premise that at the scale of interest the observable dynamics (who cooperates with whom) are what matter, not the internal knowledge state. A v2 with an explicit $M_i$ is a future extension (see Section 18, non-goals).

---


## 3. State Schema & Invariants

### Vertex Components
```
Vertex             { id: u64, mass: f32 ∈ [0, 10] }

PhysicalState      { position: Vec2, kinetic_energy: f32 ∈ [0, 1] }
EmotionalState     { valence: f32 ∈ [-1, 1], arousal: f32 ∈ [0, 1] }
EconomicState      { resources: f32 ∈ [0, 1], flow_rate: f32 ∈ [-1, 1] }
SocialState        { reputation: f32 ∈ [-1, 1], hierarchy_rank: u32, trust: f32 ∈ [0, 1] }

CouplingState      { level: f32 ∈ [0, 1], ema_state: f32 ∈ [0, 1] }
EnergyBudget       { current: f32 ∈ [0, capacity], capacity: f32, regen_rate: f32 }
LifecycleState     = Active | Strained | Zombie { decay_since: Tick }

Inbox {
    physical:  SmallVec<f32; 8>,
    emotional: SmallVec<f32; 8>,
    economic:  SmallVec<f32; 8>,
    social:    SmallVec<f32; 8>,
}
```

`Inbox` is a transient per-vertex scratch buffer populated by Stage 2 (edge signal propagation) and drained by Stage 3 (bridge cascade). Cleared at the end of Stage 3 every tick. `SmallVec` with inline capacity 8 avoids heap allocation in the common case (median degree < 8 per layer).

**EnergyBudget semantics:**
* `current` is non-negative and clamped to `[0, capacity]`.
* `regen_rate` adds to `current` each tick in Stage 5 before structural events fire.
* Structural events (Snap, Weave) deduct from `current`; if `current < event_cost`, the event is **deferred** (re-evaluated next tick).
* Default: `capacity = 1.0`, `regen_rate = 0.01`.

### Edge Components
```
Edge {
    source: Entity,
    target: Entity,
    layer: LayerKind,
    weight: f32 ∈ [-1, 1],
    resistance: f32 ∈ [0, R_max],
    history: RingBuffer<f32; HISTORY_WINDOW>,
    state: EdgeLifecycle = Active | Strained | Snapped,
}
```

### Invariants
All must hold at the end of every tick. Violations are assertion failures in debug builds and structured warnings in release.

1. **State bounds.** Every state value stays within its declared range. Out-of-range values are clamped and logged.
2. **Edge directionality.** `Edge(A → B)` and `Edge(B → A)` are distinct directed edges. Undirected relationships are modeled as a pair.
3. **Layer isolation.** An edge belongs to exactly one layer; cross-layer flow happens only via bridge functions.
4. **No dangling edges.** Every edge's `source` and `target` must reference a live (`Active` or `Zombie`) vertex.
5. **Energy bookkeeping.** Every per-vertex bridge-activity cost ($\sum_l |\delta_l \cdot A \cdot D(m)|$, see Section 5) and every structural plasticity event (Snap, Weave) must be deducted from `EnergyBudget` in the same tick. A tick cannot create free energy: if a vertex's cumulative deductions exceed `current + regen`, the offending event is deferred. Bridge outputs are *not* conservative across layers (an emotional spike can create kinetic energy); the invariant is accounting, not conservation.
6. **Mass monotonicity.** `|Δmass| ≤ MASS_DELTA_MAX` per tick.
7. **Determinism.** Given the same seed and config, a deterministic-mode run produces bit-identical outputs across invocations.

### Initial Conditions & Seeding

All randomness derives from the root `ChaCha20Rng` keyed on `seed`. The RNG is split deterministically for independent streams (positions, states, edges, per-vertex future draws).

**Vertex positions** (by `init_distribution`):
* `uniform` — `position ~ U(spatial_extent)` per axis.
* `clustered` — pick $K = \lceil \sqrt{n} \rceil$ centroids uniformly; each vertex samples from an isotropic Gaussian around its centroid with $\sigma = 0.05 \cdot \text{extent}$.
* `power_law` — positions via a Zipf-weighted mixture; $\sim 5\%$ of vertices carry disproportionate mass (exponent $s = 1.5$).

**Vertex scalar states** (uniform over valid range, unless `init_distribution = clustered`, in which case reputation and trust are correlated within a cluster via $\mathcal{N}(\mu_\text{cluster}, 0.1)$):
* `valence ~ U(-0.2, 0.2)` (low initial emotional charge).
* `arousal ~ U(0, 0.2)`.
* `resources ~ U(0.4, 0.6)`.
* `flow_rate = 0` at $t = 0$.
* `reputation ~ U(-0.1, 0.1)`, `hierarchy_rank = 0`, `trust ~ U(0.3, 0.7)`.
* `mass ~ \mathcal{N}(1.0, 0.2)` clamped to $[0.1, 10]$.
* `kinetic_energy = 0` at $t = 0$.

**Initial edges:** target count per layer is $\lfloor n_{\text{vertices}}^2 \cdot \text{edge\_density} \rfloor$ (default $0.005 \Rightarrow \sim 500\text{K}$ edges per layer at $10^4$ vertices). Per layer:
* $M_p$ — $k$-nearest spatial neighbors where $k$ satisfies target count.
* $M_s$ — random pairs with acceptance probability proportional to $1 - |\text{reputation}_A - \text{reputation}_B|$.
* $M_e$ — random pairs with acceptance proportional to $1 - |\text{valence}_A - \text{valence}_B|$.
* $M_c$ — random pairs with acceptance proportional to complementarity of `resources` (high + low pair preferred).

**Initial edge state:** `weight ~ U(-0.5, 0.5)` (scenarios can override sign bias), `resistance = 0.1`, `history = empty`, `state = Active`.

**Initial energy:** `current = capacity = 1.0` for all vertices.

**Initial coupling:** `level = 0.0`, `ema_state = 0.0`.

---

