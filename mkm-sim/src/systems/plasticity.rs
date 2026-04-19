use bevy::prelude::*;

use crate::components::{VEnergy, VPendingEnergyCost};

/// Stage 5: structural plasticity (snap, weave, zombie — Phase 4).
///
/// Phase 3 partial: regen energy budgets and deduct accumulated bridge-activity
/// cost from Stage 3. Full snap/weave/zombie mechanics are Phase 4.
pub fn plasticity_system(mut query: Query<(&mut VEnergy, &mut VPendingEnergyCost)>) {
    for (mut venergy, mut vpending) in &mut query {
        venergy.0.regen();
        venergy.0.deduct(vpending.0);
        vpending.0 = 0.0;
    }
}
