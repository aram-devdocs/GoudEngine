use std::time::Instant;

use crate::core::debugger;
use crate::ecs::World;

#[cfg(feature = "native")]
use super::super::parallel_types::UnsafePtr;
use super::ParallelExecutionStats;
use super::ParallelSystemStage;

impl ParallelSystemStage {
    /// Runs all systems using parallel execution.
    pub fn run_parallel(&mut self, world: &mut World) {
        let profiler_route = debugger::current_route().filter(debugger::profiler_enabled_for_route);

        if (self.dirty || self.batches.is_empty()) && self.config.auto_rebuild {
            if let Err(err) = self.rebuild_batches() {
                log::warn!(
                    "ParallelSystemStage '{}': ordering cycle detected, batches may be stale: {err}",
                    self.name
                );
            }
        }
        if !self.initialized {
            for system in &mut self.systems {
                system.initialize(world);
            }
            self.initialized = true;
        }
        let mut stats = ParallelExecutionStats {
            batch_count: self.batches.len(),
            system_count: self.systems.len(),
            ..Default::default()
        };
        for batch in &self.batches {
            if batch.is_empty() {
                continue;
            }
            let runnable: Vec<usize> = batch
                .system_ids
                .iter()
                .filter_map(|&id| {
                    let idx = self.system_indices[&id];
                    if self.systems[idx].should_run(world) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if runnable.is_empty() {
                continue;
            }
            if runnable.len() == 1 {
                if let Some(route_id) = profiler_route.as_ref() {
                    let system_name = self.systems[runnable[0]].name();
                    let started_at = Instant::now();
                    self.systems[runnable[0]].run(world);
                    debugger::set_system_sample(
                        route_id,
                        &self.name,
                        system_name,
                        started_at.elapsed().as_micros() as u64,
                    );
                } else {
                    self.systems[runnable[0]].run(world);
                }
                stats.sequential_systems += 1;
            } else {
                stats.parallel_systems += runnable.len();
                if runnable.len() > stats.max_parallelism {
                    stats.max_parallelism = runnable.len();
                }
                #[cfg(feature = "native")]
                {
                    // SAFETY: Batch computation ensures non-conflicting access.
                    let systems_ptr = UnsafePtr(self.systems.as_mut_ptr());
                    let world_ptr = UnsafePtr(world as *mut World);
                    let profiler_route = profiler_route.clone();
                    let stage_name = self.name.clone();
                    let runnable_systems: Vec<(usize, &'static str)> = runnable
                        .iter()
                        .map(|&idx| (idx, self.systems[idx].name()))
                        .collect();
                    rayon::scope(|s| {
                        for (idx, system_name) in runnable_systems.iter().copied() {
                            let profiler_route = profiler_route.clone();
                            let stage_name = stage_name.clone();
                            s.spawn(move |_| {
                                let started_at = Instant::now();
                                // SAFETY: Each system accesses disjoint data.
                                unsafe {
                                    let sys = &mut *systems_ptr.get().add(idx);
                                    let w = &mut *world_ptr.get();
                                    sys.run(w);
                                }
                                if let Some(route_id) = profiler_route.as_ref() {
                                    debugger::set_system_sample(
                                        route_id,
                                        &stage_name,
                                        system_name,
                                        started_at.elapsed().as_micros() as u64,
                                    );
                                }
                            });
                        }
                    });
                }
                #[cfg(not(feature = "native"))]
                {
                    for &idx in &runnable {
                        if let Some(route_id) = profiler_route.as_ref() {
                            let system_name = self.systems[idx].name();
                            let started_at = Instant::now();
                            self.systems[idx].run(world);
                            debugger::set_system_sample(
                                route_id,
                                &self.name,
                                system_name,
                                started_at.elapsed().as_micros() as u64,
                            );
                        } else {
                            self.systems[idx].run(world);
                        }
                    }
                }
            }
        }
        self.last_stats = stats;
    }
}
