pub fn sigmoid(x: f32, gain: f32, midpoint: f32) -> f32 {
    1.0 / (1.0 + (-gain * (x - midpoint)).exp())
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
