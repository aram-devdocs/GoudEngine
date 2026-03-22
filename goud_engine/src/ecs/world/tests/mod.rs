use super::*;

use super::super::archetype::ArchetypeId;
use super::super::Component;

// Test components shared across all test submodules
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Position {
    pub(super) x: f32,
    pub(super) y: f32,
}
impl Component for Position {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Velocity {
    pub(super) x: f32,
    pub(super) y: f32,
}
impl Component for Velocity {}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Name(pub(super) String);
impl Component for Name {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Player;
impl Component for Player {}

mod component_access_tests;
mod construction_tests;
mod despawn_recursive_tests;
mod despawn_tests;
mod entity_world_mut_tests;
mod insert_batch_builder_tests;
mod insert_tests;
mod pool_ops_tests;
mod remove_tests;
mod spawn_tests;
