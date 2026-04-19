use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

pub struct SimRng(pub ChaCha20Rng);

impl SimRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha20Rng::seed_from_u64(seed))
    }

    pub fn fork(&mut self, label: u64) -> ChaCha20Rng {
        use rand::RngCore;
        let seed = self.0.next_u64() ^ label;
        ChaCha20Rng::seed_from_u64(seed)
    }
}
