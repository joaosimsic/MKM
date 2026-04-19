use smallvec::SmallVec;

#[derive(Debug, Clone, Default)]
pub struct Inbox {
    pub physical: SmallVec<[f32; 8]>,
    pub emotional: SmallVec<[f32; 8]>,
    pub economic: SmallVec<[f32; 8]>,
    pub social: SmallVec<[f32; 8]>,
}

impl Inbox {
    pub fn clear(&mut self) {
        self.physical.clear();
        self.emotional.clear();
        self.economic.clear();
        self.social.clear();
    }

    pub fn sum_physical(&self) -> f32 {
        self.physical.iter().sum()
    }
    pub fn sum_emotional(&self) -> f32 {
        self.emotional.iter().sum()
    }
    pub fn sum_economic(&self) -> f32 {
        self.economic.iter().sum()
    }
    pub fn sum_social(&self) -> f32 {
        self.social.iter().sum()
    }
}
