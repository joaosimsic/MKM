use bevy::prelude::*;
use mkm_core::{
    coupling::CouplingState, energy::EnergyBudget, id::VertexId, inbox::Inbox,
    lifecycle::VertexLifecycle, state::VertexState,
};

#[derive(Component)]
pub struct VId(pub VertexId);

#[derive(Component)]
pub struct VState(pub VertexState);

#[derive(Component)]
pub struct VMass(pub f32);

#[derive(Component)]
pub struct VLifecycle(pub VertexLifecycle);

#[derive(Component)]
pub struct VCoupling(pub CouplingState);

#[derive(Component)]
pub struct VEnergy(pub EnergyBudget);

#[derive(Component, Default)]
pub struct VInbox(pub Inbox);

/// Reputation at the end of the previous tick — needed by B_se to detect drops.
#[derive(Component, Default)]
pub struct VPrevReputation(pub f32);

/// Accumulated bridge-activity cost from Stage 3, deducted in Stage 5.
#[derive(Component, Default)]
pub struct VPendingEnergyCost(pub f32);

#[derive(Bundle)]
pub struct VertexBundle {
    pub id: VId,
    pub state: VState,
    pub mass: VMass,
    pub lifecycle: VLifecycle,
    pub coupling: VCoupling,
    pub energy: VEnergy,
    pub inbox: VInbox,
    pub prev_reputation: VPrevReputation,
    pub pending_energy: VPendingEnergyCost,
}
