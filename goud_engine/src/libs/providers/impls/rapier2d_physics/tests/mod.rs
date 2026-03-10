//! Tests for the Rapier2D physics provider.

mod basic;
mod dynamics;

#[path = "../tests_gravity.rs"]
mod gravity_tests;

#[path = "../tests_collision_response.rs"]
mod collision_response;

#[path = "../tests_events_queries.rs"]
mod events_queries;

#[path = "../tests_collision_events_layers_raycast.rs"]
mod collision_events_layers_raycast;
