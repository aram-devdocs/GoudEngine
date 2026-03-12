use std::collections::HashMap;

use crate::ecs::schedule::topological_sort::OrderingCycleError;
use crate::ecs::system::SystemId;

use super::ParallelSystemStage;

pub(super) fn rebuild_batches(stage: &mut ParallelSystemStage) -> Result<(), OrderingCycleError> {
    stage.batches.clear();
    if stage.systems.is_empty() {
        stage.dirty = false;
        return Ok(());
    }

    let execution_order = if stage.config.respect_ordering && !stage.orderings.is_empty() {
        let mut sorter = crate::ecs::schedule::topological_sort::TopologicalSorter::with_capacity(
            stage.systems.len(),
            stage.orderings.len(),
        );
        for system in &stage.systems {
            sorter.add_system(system.id(), system.name());
        }
        for ordering in &stage.orderings {
            sorter.add_system_ordering(*ordering);
        }
        sorter.sort()?
    } else {
        stage.systems.iter().map(|system| system.id()).collect()
    };

    let mut direct_predecessors: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
    if stage.config.respect_ordering {
        for ordering in &stage.orderings {
            direct_predecessors
                .entry(ordering.second())
                .or_default()
                .push(ordering.first());
        }
    }

    let mut system_batch_index: HashMap<SystemId, usize> = HashMap::new();
    for system_id in execution_order {
        let system_idx = stage.system_indices[&system_id];
        let system = &stage.systems[system_idx];
        let system_access = system.component_access();
        let system_read_only = system.is_read_only();
        let min_batch_idx = if stage.config.respect_ordering {
            direct_predecessors
                .get(&system_id)
                .map(|preds| {
                    preds
                        .iter()
                        .filter_map(|pred| system_batch_index.get(pred))
                        .max()
                        .map(|&max| max + 1)
                        .unwrap_or(0)
                })
                .unwrap_or(0)
        } else {
            0
        };

        let mut assigned = false;
        for batch_idx in min_batch_idx..stage.batches.len() {
            let batch = &stage.batches[batch_idx];
            let has_conflict = batch.system_ids.iter().any(|&batch_system_id| {
                let batch_system_idx = stage.system_indices[&batch_system_id];
                system_access.conflicts_with(&stage.systems[batch_system_idx].component_access())
            });
            if !has_conflict {
                stage.batches[batch_idx].add(system_id, system_read_only);
                system_batch_index.insert(system_id, batch_idx);
                assigned = true;
                break;
            }
        }

        if !assigned {
            let new_batch_idx = stage.batches.len();
            let mut batch = crate::ecs::schedule::parallel_types::ParallelBatch::with_capacity(4);
            batch.add(system_id, system_read_only);
            stage.batches.push(batch);
            system_batch_index.insert(system_id, new_batch_idx);
        }
    }

    stage.dirty = false;
    Ok(())
}
