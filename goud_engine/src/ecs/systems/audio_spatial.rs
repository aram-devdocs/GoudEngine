//! Spatial audio ECS system.
//!
//! Collects listener and emitter transforms each frame and synchronizes them
//! to the `AudioManager` resource.

#[cfg(feature = "native")]
use crate::assets::AudioManager;
#[cfg(any(feature = "native", test))]
use crate::ecs::components::{AudioEmitter, AudioListener, AudioSource, Transform, Transform2D};
#[cfg(any(feature = "native", test))]
use crate::ecs::Entity;
use crate::ecs::World;

#[cfg(any(feature = "native", test))]
#[derive(Debug, Clone)]
struct SpatialAudioFrame {
    listener_position: [f32; 3],
    sources: Vec<SpatialEmitterUpdate>,
}

#[cfg(any(feature = "native", test))]
#[derive(Debug, Clone)]
struct SpatialEmitterUpdate {
    sink_id: u64,
    source_position: [f32; 3],
    max_distance: f32,
    rolloff: f32,
    base_volume: f32,
}

#[cfg(any(feature = "native", test))]
fn entity_position_3d(world: &World, entity: Entity) -> Option<[f32; 3]> {
    if let Some(t3) = world.get::<Transform>(entity) {
        return Some([t3.position.x, t3.position.y, t3.position.z]);
    }

    world
        .get::<Transform2D>(entity)
        .map(|t2| [t2.position.x, t2.position.y, 0.0])
}

#[cfg(any(feature = "native", test))]
fn collect_spatial_audio_frame(world: &World) -> SpatialAudioFrame {
    let entities: Vec<_> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().copied())
        .collect();

    let listener_position = entities
        .iter()
        .copied()
        .find_map(|entity| {
            let listener = world.get::<AudioListener>(entity)?;
            if !listener.enabled {
                return None;
            }
            entity_position_3d(world, entity)
        })
        .unwrap_or([0.0, 0.0, 0.0]);

    let mut sources = Vec::new();
    for entity in entities {
        let Some(emitter) = world.get::<AudioEmitter>(entity) else {
            continue;
        };
        if !emitter.enabled {
            continue;
        }

        let Some(source) = world.get::<AudioSource>(entity) else {
            continue;
        };

        if !source.spatial {
            continue;
        }

        let Some(sink_id) = source.sink_id else {
            continue;
        };

        let Some(position) = entity_position_3d(world, entity) else {
            continue;
        };

        sources.push(SpatialEmitterUpdate {
            sink_id,
            source_position: position,
            max_distance: emitter.max_distance.max(0.1),
            rolloff: emitter.rolloff.max(0.01),
            base_volume: source.volume.clamp(0.0, 1.0),
        });
    }

    SpatialAudioFrame {
        listener_position,
        sources,
    }
}

/// Synchronizes ECS listener/emitter spatial state into [`AudioManager`].
///
/// This function is intentionally non-panicking and returns early when no
/// `AudioManager` resource is present.
#[cfg(feature = "native")]
pub fn update_spatial_audio(world: &mut World) {
    let frame = collect_spatial_audio_frame(world);

    let Some(audio_manager) = world.get_resource_mut::<AudioManager>() else {
        return;
    };

    audio_manager.set_listener_position(frame.listener_position);

    let mut active_sink_ids = Vec::with_capacity(frame.sources.len());
    for update in &frame.sources {
        if audio_manager.register_spatial_source(
            update.sink_id,
            update.source_position,
            update.max_distance,
            update.rolloff,
            update.base_volume,
        ) {
            active_sink_ids.push(update.sink_id);
        }
    }

    audio_manager.retain_spatial_sources(&active_sink_ids);
}

/// Web builds do not include the native `AudioManager` resource.
#[cfg(not(feature = "native"))]
pub fn update_spatial_audio(_world: &mut World) {}

#[cfg(test)]
mod tests {
    use super::collect_spatial_audio_frame;
    use crate::assets::AssetHandle;
    use crate::core::math::{Vec2, Vec3};
    use crate::ecs::components::{
        AudioEmitter, AudioListener, AudioSource, Transform, Transform2D,
    };
    use crate::ecs::World;

    #[test]
    fn test_collect_frame_uses_listener_and_emitter_positions() {
        let mut world = World::new();

        let listener = world.spawn_empty();
        world.insert(listener, AudioListener::new());
        world.insert(listener, Transform2D::from_position(Vec2::new(5.0, 7.0)));

        let emitter = world.spawn_empty();
        let mut source = AudioSource::new(AssetHandle::default()).with_spatial(true);
        source.sink_id = Some(42);
        world.insert(emitter, source);
        world.insert(
            emitter,
            AudioEmitter::new()
                .with_max_distance(120.0)
                .with_rolloff(2.0),
        );
        world.insert(
            emitter,
            Transform::from_position(Vec3::new(10.0, 11.0, 12.0)),
        );

        let frame = collect_spatial_audio_frame(&world);
        assert_eq!(frame.listener_position, [5.0, 7.0, 0.0]);
        assert_eq!(frame.sources.len(), 1);
        assert_eq!(frame.sources[0].sink_id, 42);
        assert_eq!(frame.sources[0].source_position, [10.0, 11.0, 12.0]);
        assert_eq!(frame.sources[0].max_distance, 120.0);
        assert_eq!(frame.sources[0].rolloff, 2.0);
    }

    #[test]
    fn test_collect_frame_ignores_disabled_listener_and_emitter() {
        let mut world = World::new();

        let disabled_listener = world.spawn_empty();
        world.insert(disabled_listener, AudioListener::new().with_enabled(false));
        world.insert(
            disabled_listener,
            Transform2D::from_position(Vec2::new(100.0, 100.0)),
        );

        let enabled_listener = world.spawn_empty();
        world.insert(enabled_listener, AudioListener::new());
        world.insert(
            enabled_listener,
            Transform2D::from_position(Vec2::new(1.0, 2.0)),
        );

        let disabled_emitter = world.spawn_empty();
        let mut source = AudioSource::new(AssetHandle::default()).with_spatial(true);
        source.sink_id = Some(7);
        world.insert(disabled_emitter, source);
        world.insert(disabled_emitter, AudioEmitter::new().with_enabled(false));
        world.insert(
            disabled_emitter,
            Transform2D::from_position(Vec2::new(9.0, 9.0)),
        );

        let frame = collect_spatial_audio_frame(&world);
        assert_eq!(frame.listener_position, [1.0, 2.0, 0.0]);
        assert!(frame.sources.is_empty());
    }

    #[test]
    fn test_collect_frame_defaults_listener_to_origin() {
        let mut world = World::new();
        let frame = collect_spatial_audio_frame(&world);
        assert_eq!(frame.listener_position, [0.0, 0.0, 0.0]);
        assert!(frame.sources.is_empty());
    }
}
