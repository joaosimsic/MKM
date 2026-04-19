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

#[derive(Bundle)]
pub struct VertexBundle {
    pub id: VId,
    pub state: VState,
    pub mass: VMass,
    pub lifecycle: VLifecycle,
    pub coupling: VCoupling,
    pub energy: VEnergy,
    pub inbox: VInbox,
}
