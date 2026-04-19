## 8. Simulation Scenarios & Resilience Logic

### Universal Event Injector
Events are specified as:
```
TensorImpact { physical: f32, emotional: f32, economic: f32, social: f32 }
```
Targets: single vertex, spatial region, layer-global, or mesh-global. Sequences supported for escalating crises. Multiple events in the same tick are applied in stable sort order (tick, insertion order).

### Scenario Presets
* **Economic Shock:** `Impact { economic: -0.8 }` mesh-global. Observe cascade into $M_s$ and $M_e$.
* **Social Fragmentation:** targeted edge pruning in $M_s$. Observe shear and potential shatter.
* **Crisis Escalation:** gradual tightening of coupling + sequential shocks. Locate the Shatter Point.
* **Mutual Aid Formation:** sustained $M_c$ stress without shatter; observe Weave events forming new $M_s$ edges.

### Resilience & Self-Correction
* **Edge Elasticity:** resilient meshes enter `Strained` (high resistance, still active) before snapping.
* **Vertex Decoupling:** a vertex can attenuate its $B_{ep}$ gain when $M_e$ volatility exceeds a threshold — a "lock" on physical response to emotional spikes.
* **Predictive Engineering (The Dark Side):** search over initial conditions to find the **Mathematical Minimum** of trust and resources needed for recovery from a given shock. Output: resilience heatmaps.

### Formal Definition: The Mathematical Minimum

Given a scenario $S$ (initial distribution + shock event sequence), a search axis $x$ (e.g. `initial_trust_mean`, `initial_resources_mean`, `edge_density`), and recovery parameters $(\theta_{\text{recovery}}, T_{\text{recovery}}, p)$:

$$
\text{MM}(S, x) \;=\; \arg\min_{x} \Big\{\; P\big(\,L_T / L_0 \;\geq\; \theta_{\text{recovery}} \,\big|\, S, x\big) \;\geq\; p \;\Big\}
$$

where:
* $L_0$ is the pre-shock largest-connected-component fraction in the target layer (default: $M_s$).
* $L_T$ is the same measurement $T_{\text{recovery}}$ ticks after the last shock in $S$.
* $\theta_{\text{recovery}} \in (0, 1]$ is the recovery threshold (default: 0.8 — "80% of pre-shock connectivity restored").
* $T_{\text{recovery}}$ is the recovery horizon in ticks (default: $5 \times$ shock duration, floor 500).
* $p$ is the required success probability across a seed ensemble (default: 0.9, measured over $N \geq 32$ seeds per point — see ensemble support in Section 13).

The `mkm-cli sweep` and `scenarios::find_minimum` helpers implement this as a monotone binary search on $x$, provided $x$ is monotone in $P(\text{recovery})$ (trust and resources are; `edge_density` is bimodal — use a grid sweep for non-monotone axes).

**Failure modes to flag in output:** (i) the entire search range fails to meet $p$ (no MM exists at these recovery params — widen the axis or loosen $\theta_{\text{recovery}}$); (ii) $P(\text{recovery})$ is non-monotone in $x$ (binary search invalid — fall back to grid); (iii) the ensemble's IQR at the boundary exceeds 20% of the median (MM is unstable — report the interval, not a point).

---

