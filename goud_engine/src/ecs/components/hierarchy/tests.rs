//! Unit tests for hierarchy components.
//!
//! Split into focused submodules per component:
//! - `tests_parent`: tests for the [`Parent`] component
//! - `tests_children`: tests for the [`Children`] component
//! - `tests_name`: tests for the [`Name`] component
//! - `tests_integration`: cross-component interaction tests

#[cfg(test)]
mod tests_parent;

#[cfg(test)]
mod tests_children;

#[cfg(test)]
mod tests_name;

#[cfg(test)]
mod tests_integration;
