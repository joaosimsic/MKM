pub fn sigmoid(x: f32, gain: f32, midpoint: f32) -> f32 {
    1.0 / (1.0 + (-gain * (x - midpoint)).exp())
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

/// Damping factor for a vertex with influence mass `m`.
/// Returns `m / (m + k)` — approaches 1 as mass grows.
pub fn mass_damp(m: f32, k: f32) -> f32 {
    m / (m + k)
}
