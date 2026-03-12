use crate::context_registry::GoudContextId;
use crate::core::debugger::{
    begin_frame, register_context, reset_for_tests, scoped_route, set_profiling_enabled_for_context,
    snapshot_for_context, test_lock, DebuggerConfig, RuntimeSurfaceKind,
};
use crate::ecs::schedule::{ParallelSystemStage, Stage, SystemStage};
use crate::ecs::system::System;
use crate::ecs::World;

struct NamedSystem(&'static str);

impl System for NamedSystem {
    fn name(&self) -> &'static str {
        self.0
    }

    fn run(&mut self, _: &mut World) {}
}

#[test]
fn test_system_stage_debugger_samples_only_when_profiling_enabled() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(91, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            ..DebuggerConfig::default()
        },
    );

    let mut stage = SystemStage::new("Update");
    stage.add_system(NamedSystem("ExampleSystem"));
    let mut world = World::new();

    begin_frame(&route, 1, 0.016, 0.016);
    scoped_route(Some(route.clone()), || stage.run(&mut world));
    assert!(
        snapshot_for_context(context_id)
            .expect("snapshot should exist")
            .profiler_samples
            .is_empty()
    );

    assert!(set_profiling_enabled_for_context(context_id, true));
    begin_frame(&route, 2, 0.016, 0.032);
    scoped_route(Some(route), || stage.run(&mut world));
    let snapshot = snapshot_for_context(context_id).expect("snapshot should exist");
    assert_eq!(snapshot.profiler_samples.len(), 1);
    assert_eq!(snapshot.profiler_samples[0].sample_kind, "system");
    assert_eq!(snapshot.profiler_samples[0].stage, "Update");
    assert_eq!(snapshot.profiler_samples[0].name, "ExampleSystem");
}

#[test]
fn test_parallel_stage_debugger_samples_capture_each_system_name() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(92, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            ..DebuggerConfig::default()
        },
    );
    assert!(set_profiling_enabled_for_context(context_id, true));

    let mut stage = ParallelSystemStage::new("Update");
    stage.add_system(NamedSystem("SystemA"));
    stage.add_system(NamedSystem("SystemB"));
    let mut world = World::new();

    begin_frame(&route, 1, 0.016, 0.016);
    scoped_route(Some(route), || stage.run(&mut world));
    let snapshot = snapshot_for_context(context_id).expect("snapshot should exist");
    let sample_names: Vec<&str> = snapshot
        .profiler_samples
        .iter()
        .map(|sample| sample.name.as_str())
        .collect();

    assert_eq!(snapshot.profiler_samples.len(), 2);
    assert!(sample_names.contains(&"SystemA"));
    assert!(sample_names.contains(&"SystemB"));
    assert!(snapshot
        .profiler_samples
        .iter()
        .all(|sample| sample.stage == "Update" && sample.sample_kind == "system"));
}
