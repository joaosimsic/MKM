## 2. Dynamic Mechanics & Topology

### Pondered Edges
Edges are **active filters with memory**, not passive links.
* **Resistance:** `resistance: f32 ∈ [0, R_max]`. Attenuates signal strength. `conductance = 1 / (1 + resistance)`.
* **Weight:** `weight: f32 ∈ [-1, 1]`. Sign determines cooperative vs. antagonistic.
* **Hysteresis:** resistance is recomputed each tick as `resistance_{t+1} = α · resistance_t + (1 - α) · f(history)`. Path dependency is first-class.

### Kinetic Cascades (Bleed-through)
Horizontal propagation along edges happens per layer. Vertical propagation happens per vertex via **Bridge Functions** (Section 5 defines the full tick pipeline).

### Structural Plasticity (Evolutionary Topology)
* **Edge Pruning (The Snap):** triggered when `resistance > YIELD_POINT` or `resource_flow == 0` in $M_c$.
* **Edge Creation (The Weave):** triggered when a vertex enters **Tight Coupling** with fewer than `MIN_EDGES` active connections; candidates found via spatial (Quadtree) or trust-threshold lookup.
* **Node Latency (Zombies):** a vertex with zero edges does not vanish — it is flagged `Zombie` with a decayed snapshot, available for re-entry. Scalar state *and* mass decay by `ZOMBIE_DECAY` per tick while `Zombie`; position is frozen. On re-entry (a Weave targeting the Zombie), the decayed state is retained — the vertex does not "reset" to any default.

### Coupling States
* **Loose Coupling** (`level ∈ [0.0, 0.5)`): layers operate with high independence (normal/stable).
* **Tight Coupling** (`level ∈ [0.5, 1.0]`): layers lock; bridge-function outputs are amplified by the coupling factor (crisis/shock).

`coupling.level` is itself a feedback function of global stress metrics (see Crisis Metrics).

### Mass Dynamics
Mass drifts slowly with cumulative influence and throughput. Per tick, for vertex $V$:

$$\Delta m_{\text{raw}} = K_{\text{mass}} \cdot \left( \text{social\_influence}(V) + \text{economic\_throughput}(V) \right)$$

$$\Delta m = \text{clamp}(\Delta m_{\text{raw}}, -\text{MASS\_DELTA\_MAX}, +\text{MASS\_DELTA\_MAX})$$

where:
* $\text{social\_influence}(V) = \sum_{e \in \text{out}_s(V)} |e.\text{weight}|$ — sum of absolute weights of active outgoing $M_s$ edges.
* $\text{economic\_throughput}(V) = \overline{|\text{flow\_rate}|}$ over `HISTORY_WINDOW`.
* $K_{\text{mass}} = 0.001$ default — mass takes $\sim 10^3$ ticks of sustained influence to move by 1.0.

**Effect:** high-mass vertices resist bridge-function perturbations and dominate their edges' signal contribution. Mass is never reset; it only clamps and drifts within `MASS_DELTA_MAX` per tick in either direction. The damping factor $D(m) = 1 / (1 + K_{\text{mass\_damp}} \cdot (m - 1))$ applied to every bridge output is specified in Section 6 (**Bridge Functions**); the edge-propagation mass weighting $m / (1 + m)$ is specified in Section 5, Stage 2.

---


## 4. Crisis Metrics: Shear & Collapse
Two primary indicators of systemic failure, plus one global event.

### Vertex Shear ($S_\mu$)
* **Definition:** horizontal displacement between a vertex's representations across layers.
* **Formula:** $S_\mu = \frac{1}{\binom{|L|}{2}} \sum_{i < j} \| \hat{s}_i - \hat{s}_j \|_2$, where $\hat{s}_l$ is the vertex's state in layer $l$ projected into a common normalized embedding (default: linear per-layer normalization to $[0, 1]^n$).
* **Conditional Calculation:** only computed when `max_layer_delta > TENSION_THRESHOLD`. Below threshold, $S_\mu$ is assumed unchanged.
* **Consequence:** increases per-tick energy cost (`energy_cost += SHEAR_COST_FACTOR · S_μ`); feeds back into coupling.
* **Interpretation caveat:** $S_\mu$ mixes incommensurate quantities (position, trust, valence, flow) via ad-hoc $[0, 1]$ normalization. Treat it as a **relative** quantity — meaningful for trajectories within one run and for comparing runs at matched `init_distribution`, **not** as an absolute cross-scenario distance. A learned embedding is a v2 consideration (Section 17.1).

### Layer Collapse ($C_\mu$)
* **Definition:** rate of horizontal edge deletions within a specific layer.
* **Formula:** $C_\mu^{(l)} = \frac{|\text{snapped}(l, T)|}{|\text{active}(l, t_0)|}$ over rolling window $T$ (default `COLLAPSE_WINDOW = 64`).
* **Collapse Flag:** layer $l$ enters `Collapsed` when $C_\mu^{(l)} > \text{COLLAPSE\_THRESHOLD}$.

### The Shatter Point
* **Global event:** triggered when the largest connected component (LCC) in any layer falls below $\phi_c$ of total vertices (default `0.5`, tunable per Percolation Theory).
* **Post-Shatter:** simulation continues (to model aftermath) and the event is flagged in the output stream. Recovery via Weave events is possible but rare.

### Feedback Loop
$S_\mu$ and $C_\mu$ feed back into the global stress metric that drives `CouplingState.level`. This is the positive feedback loop that produces crisis escalation.

**Global stress formula:**

$$\text{stress}_t = w_s \cdot \overline{S_\mu} + w_c \cdot \max_l C_\mu^{(l)} + w_r \cdot (1 - \overline{\text{resources}})$$

with default weights $w_s = 0.4$, $w_c = 0.4$, $w_r = 0.2$. All three terms are in $[0, 1]$, so $\text{stress} \in [0, 1]$.

**Coupling update (logistic + EMA smoothing):**

$$\text{level}_\text{new} = \sigma\left( \text{STRESS\_GAIN} \cdot (\text{stress}_t - \text{STRESS\_MIDPOINT}) \right)$$

$$\text{level}_{t+1} = \beta \cdot \text{level}_t + (1 - \beta) \cdot \text{level}_\text{new}$$

where $\sigma(x) = 1 / (1 + e^{-x})$, $\text{STRESS\_GAIN} = 4.0$, $\text{STRESS\_MIDPOINT} = 0.5$, $\beta = 0.9$ (stored in `CouplingState.ema_state`).

**Amplification factor used by bridge functions:**

$$A = 1 + (\text{COUPLING\_AMPLIFICATION} - 1) \cdot \text{level}$$

At `level = 0.0`: $A = 1$ (no amplification). At `level = 1.0`: $A = \text{COUPLING\_AMPLIFICATION}$ (default 2.0).

---

## 5. Tick Execution Model

Every tick is a **strictly ordered pipeline**. Parallelism within a stage is allowed; cross-stage parallelism is not.

### Stage Order (per tick)
1. **Ingest events.** Drain `EventQueue`; apply `TensorImpact` to affected vertices.
2. **Edge signal propagation.** For each edge $E = (A \to B, l, w, r)$:
    * Extract scalar signal from source in layer $l$ (see **Per-Layer Signal Extractor** below).
    * Compute conductance: $g = 1 / (1 + r)$.
    * Output: $s_\text{out} = w \cdot g \cdot s_\text{in} \cdot (A.\text{mass} / (1 + A.\text{mass}))$ — mass weighting ensures high-mass sources dominate their edges.
    * Append $s_\text{out}$ to $B.\text{inbox}[l]$.
3. **Bridge function cascade.** For each vertex: aggregate inbox per layer ($\text{agg}_l = \sum B.\text{inbox}[l]$), apply intra-layer updates, then apply all bridge functions targeting each layer, sum deltas, clamp to valid ranges. Clear inbox at end of stage.
4. **Crisis metric update.** Recompute $S_\mu$ (conditionally), $C_\mu$ per layer, global LCC. Update `CouplingState`.
5. **Structural plasticity.** Regen energy budgets. Evaluate Snap, Weave, and Zombie transitions. Mutate the graph. Deduct event costs.
6. **History commit.** Append tick's signal magnitudes to edge history buffers. Recompute edge resistance via hysteresis rule.
7. **Output & logging.** Emit per-tick metrics; flush snapshots if scheduled.

### Per-Layer Signal Extractor
Each layer defines a canonical scalar signal for edge propagation:
* $M_p$: $s = \text{kinetic\_energy}$ (non-negative; represents physical disturbance radiating outward).
* $M_e$: $s = \text{arousal} \cdot \text{sign}(\text{valence})$ (signed emotional charge; negative = fear/anger, positive = joy/confidence).
* $M_c$: $s = \text{flow\_rate}$ (signed; represents resource flux direction).
* $M_s$: $s = \text{trust} \cdot \text{reputation}$ (signed; trust gates how much reputation translates to social pressure).

### Determinism
* **Single seed:** the simulation accepts a single `u64` seed; all RNG derives from a seeded `ChaCha20Rng`.
* **Parallel ordering:** within a stage, parallelism must use associative reductions. Non-associative operations (e.g., parallel float summation) introduce bit-level noise and require `parallelism = "parallel"` in config.
* **Reproducibility target:** two runs with the same seed and config must produce identical outputs under `parallelism = "deterministic"`.

### Time Model
* **Fixed `dt`:** each tick advances `sim_time += dt`. Default `dt = 1.0` (abstract units).
* **No variable-step integration.** Sub-tick resolution is out of scope.
* **Tick → wall-time mapping is intentionally unfixed.** One tick represents "one interaction round" of unspecified real-world duration. Decay rates, hysteresis windows, and mass drift are all defined per-tick, so they inherit whatever calendar time a tick ends up mapping to at calibration. Calibration fixes the mapping by matching one stylized fact's observed timescale (e.g. a financial-contagion propagation that completes in N hours) to a simulated tick count (see Section 16).

### Intra-Layer Updates

Applied first inside Stage 3, from the inbox aggregate $\text{agg}_l$. These are *not* bridge functions — they are per-layer dynamics on each vertex, executed before cross-layer bridges fire. Full bridge-function spec is in Section 6.

| Layer | Update |
|---|---|
| $M_p$ | $\Delta \text{kinetic} = K_{\text{KIN}} \cdot \text{agg}_p - \text{FRICTION} \cdot \text{kinetic}$ |
| $M_p$ | $\Delta \text{position} = \text{direction}(\text{agg}_p) \cdot \text{kinetic} \cdot dt$ |
| $M_e$ | $\Delta \text{valence} = K_{\text{VAL}} \cdot \text{agg}_e - \text{DECAY}_{\text{val}} \cdot \text{valence}$ |
| $M_e$ | $\Delta \text{arousal} = K_{\text{AR}} \cdot |\text{agg}_e| - \text{DECAY}_{\text{ar}} \cdot \text{arousal}$ |
| $M_c$ | $\Delta \text{flow\_rate} = K_{\text{FLOW}} \cdot \text{agg}_c - \text{DECAY}_{\text{flow}} \cdot \text{flow\_rate}$ |
| $M_c$ | $\Delta \text{resources} = \text{flow\_rate} \cdot dt$ |
| $M_s$ | $\Delta \text{trust} = K_{\text{TR}} \cdot \text{agg}_s - \text{DECAY}_{\text{tr}} \cdot (\text{trust} - 0.5)$ |
| $M_s$ | $\Delta \text{reputation} = K_{\text{REP}} \cdot \text{agg}_s$ |

Decay terms mean-revert states in the absence of signal. All outputs clamped to the ranges declared in Section 3.

---

## 6. Bridge Functions

> [!IMPORTANT]
> **Bridge functions are the model.** The simulation succeeds or fails based on the quality of the equations below — the ones that say exactly how emotional weight transforms into physical movement, how scarcity erodes trust, and so on. Every other piece is scaffolding. Make them first-class: swappable, testable, parameterized, and *calibrated* (Section 16).

### Mechanics

Each bridge function is a closure `fn(&VertexView, &Params, A) -> LayerDelta` keyed by `(source_layer, target_layer)`, registered in a `BridgeRegistry` at scenario load. Multiple bridges targeting the same layer are **summed** (not chained); chaining requires an explicit scenario override.

Applied in Stage 3 of the tick pipeline (Section 5), *after* intra-layer updates. Each bridge output is modulated by the coupling amplification $A$ (from the global coupling state; see Section 4) and by a per-vertex mass damping factor $D(m)$.

**Mass damping factor** (applied to every bridge output; drift rule for $m$ itself is in Section 2, **Mass Dynamics**):

$$D(m) = \frac{1}{1 + K_{\text{mass\_damp}} \cdot (m - 1)}$$

where $K_{\text{mass\_damp}} = 0.5$ default. At $m = 1.0$: $D = 1$. At $m = 5.0$: $D \approx 0.33$. High-mass vertices are stiffer under cross-layer perturbation.

### Default Set

Let $\delta_l$ denote the raw delta produced; applied as $\text{state}_l \mathrel{{+}{=}} \delta_l \cdot A \cdot D(m)$.

| Bridge | Trigger | Delta |
|---|---|---|
| $B_{ep}$ (E → P) | arousal spike | $\delta_{\text{kinetic}} = K_{EP} \cdot \text{arousal}$ |
| $B_{pe}$ (P → E) | physical shock | $\delta_{\text{arousal}} = K_{PE} \cdot \text{kinetic}$ |
| $B_{ec}$ (E → C) | fear (high arousal + negative valence) | $\delta_{\text{flow}} = -K_{EC} \cdot \text{arousal} \cdot \max(0, -\text{valence})$ |
| $B_{ce}$ (C → E) | scarcity / abundance | $\delta_{\text{valence}} = K_{CE} \cdot (\text{resources} - 0.5)$ |
| $B_{pc}$ (P → C) | displacement | $\delta_{\text{flow}} = -K_{PC} \cdot \text{kinetic}$ |
| $B_{se}$ (S → E) | reputation drop | $\delta_{\text{arousal}} = K_{SE} \cdot \max(0, \text{reputation}_{t-1} - \text{reputation}_t)$ |
| $B_{sc}$ (S → C) | trust gate | $\delta_{\text{flow}} = K_{SC} \cdot (\text{trust} - \text{TRUST\_THRESHOLD})$ |
| $B_{cs}$ (C → S) | flow-rate accumulation | $\delta_{\text{trust}} = K_{CS} \cdot \text{flow\_rate}$ |

Eight default bridges covering both directions of each adjacent layer pair, except the two noted below. Every bridge's $K$ constant is a calibration target (Section 16).

### Omitted Bridges (documented rationale)

* $B_{cp}$ (C → P): wealth does not directly cause kinetic motion — movement requires agent decisions, which this model does not represent. Omitted to avoid a free "money moves people" channel that would bias the Mathematical Minimum calculation. Scenarios modeling migration should register it explicitly.
* $B_{es}$ (E → S): emotional state affecting reputation is mediated by observable behaviour, which is out of scope. Sustained $M_e$ volatility feeds back to $M_s$ indirectly via $B_{ec} \to B_{cs}$ (fear → flow collapse → trust erosion), which is the realistic channel.

### Registering Custom Bridges

Scenarios may register additional bridges (including the two above, or e.g. a v2 $M_i$ epistemic layer's cross-bridges) by providing a closure and a $K$ constant through `BridgeRegistry::register()`. Custom bridges receive the same $A$ and $D(m)$ treatment as defaults.

### Energy Bookkeeping

Per-vertex **bridge activity cost** = $\sum_l |\delta_l \cdot A \cdot D(m)|$ across all bridges that fired this tick. This cost is deducted from `EnergyBudget.current` in Stage 5, implementing Invariant 5. A vertex that cannot afford its own bridge activity has the lowest-magnitude bridges suppressed until it can.

### Worked Example — One Vertex, One Tick

A fully-traced example to anchor the pipeline. Use it to verify you understand the ordering and the formulas before reading code.

**Setup (vertex $V$ at start of tick $t$):**
* `PhysicalState`: position $(50, 50)$, kinetic_energy $0.0$
* `EmotionalState`: valence $0.0$, arousal $0.0$
* `EconomicState`: resources $0.5$, flow_rate $0.0$
* `SocialState`: reputation $0.1$, trust $0.5$
* `mass` $= 1.0$, `CouplingState.level` $= 0.2$, `EnergyBudget.current` $= 1.0$
* One incoming $M_e$ edge from neighbour $N$: `weight = 0.5`, `resistance = 0.3`. $N$'s emotional state: valence $= -0.4$, arousal $= 0.7$, mass $= 1.0$.
* No `TensorImpact` events this tick.

**Stage 1 — Ingest.** Empty event queue. No state change.

**Stage 2 — Edge signal propagation.** For the $N \to V$ edge in $M_e$:
* Signal extractor: $s_\text{in} = \text{arousal}_N \cdot \text{sign}(\text{valence}_N) = 0.7 \cdot (-1) = -0.7$.
* Conductance: $g = 1 / (1 + 0.3) = 0.769$.
* Mass weighting: $m_N / (1 + m_N) = 0.5$.
* Output: $s_\text{out} = 0.5 \cdot 0.769 \cdot (-0.7) \cdot 0.5 = -0.1346$.
* Append to $V.\text{inbox}[M_e]$.

**Stage 3 — Bridge cascade.** Aggregate: $\text{agg}_e = -0.1346$.

*Intra-layer updates (applied first, per the table above):*
* $\Delta\text{valence} = K_{VAL} \cdot \text{agg}_e - \text{DECAY}_{val} \cdot \text{valence} = 0.5 \cdot (-0.1346) - 0 = -0.0673$.
* $\Delta\text{arousal} = K_{AR} \cdot |\text{agg}_e| - \text{DECAY}_{ar} \cdot \text{arousal} = 0.3 \cdot 0.1346 - 0 = +0.0404$.

After intra-updates: valence $= -0.0673$, arousal $= 0.0404$.

*Cross-layer bridges (applied next, with $A = 1 + (2.0 - 1) \cdot 0.2 = 1.2$ and $D(1.0) = 1.0$):*
* $B_{ep}$: raw $\delta_\text{kinetic} = K_{EP} \cdot \text{arousal} = 0.4 \cdot 0.0404 = 0.0162$; applied as $0.0162 \cdot 1.2 \cdot 1.0 = +0.0194$.
* $B_{ec}$: raw $\delta_\text{flow} = -K_{EC} \cdot \text{arousal} \cdot \max(0, -\text{valence}) = -0.3 \cdot 0.0404 \cdot 0.0673 = -0.000816$; applied as $-0.000816 \cdot 1.2 = -0.000979$.
* $B_{ce}$: $\delta_\text{valence} = K_{CE} \cdot (0.5 - 0.5) = 0$ — no effect (resources at midpoint).
* All other bridges have zero trigger.

After bridges, clamped to valid ranges:
* valence $= -0.0673$, arousal $= 0.0404$, kinetic $= 0.0194$, flow $= -0.000979$.

Bridge activity cost: $|0.0194| + |0.000979| = 0.0203$. Clear inbox.

**Stage 4 — Crisis metrics.** `max_layer_delta` ≈ $0.067$ (valence). Below `TENSION_THRESHOLD = 0.15`, so $S_\mu$ is **not recomputed** for $V$. No snaps this tick, so $C_\mu$ unchanged. Global stress stays low; `coupling.level` drifts via EMA toward a low target.

**Stage 5 — Structural plasticity.** Energy regen: `current = min(1.0 + 0.01, 1.0) = 1.0`. No snap (edge `resistance = 0.3 < 0.9`). No zombies. No Weave triggers (`active_edges ≥ MIN_EDGES`). Deduct bridge activity: `current -= BRIDGE_ACTIVITY_COST · 0.0203 = 0.000406`. Final `current = 0.9996`.

**Stage 6 — History commit.** Append $|s_\text{out}| = 0.1346$ to the $N \to V$ edge's ring buffer. New resistance: $0.8 \cdot 0.3 + 0.2 \cdot \overline{\text{history}} \approx 0.267$ (assuming prior steady state at $0.3$).

**Stage 7 — Output.** Emit the tick-level row (n_active_vertices, mean_shear, coupling_mean, etc.).

**What this example demonstrates:** (a) intra-layer updates always precede cross-layer bridges; (b) coupling amplifies bridge outputs but not intra-layer updates; (c) bridge activity is *non-conservative* across layers (emotion created kinetic energy) — Invariant 5 is accounting, not conservation; (d) hysteresis lets resistance move down as well as up when signal is moderate.

---

