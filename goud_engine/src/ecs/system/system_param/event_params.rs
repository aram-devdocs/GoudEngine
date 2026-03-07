//! [`SystemParam`] implementations for event access types.
//!
//! Provides [`EcsEventReader`] and [`EcsEventWriter`] as ECS system parameters
//! for reading and writing events through the [`Events`] resource.

use std::marker::PhantomData;

use crate::core::event::{Event, Events};
use crate::ecs::query::Access;
use crate::ecs::resource::{Res, ResMut, ResourceId};
use crate::ecs::World;

use super::traits::{ReadOnlySystemParam, SystemParam, SystemParamState};

// =============================================================================
// EcsEventReader<E> - Immutable Event SystemParam
// =============================================================================

/// Cached state for [`EcsEventReader`].
///
/// Stores a per-system cursor that tracks which events have been read across
/// system runs. Each system gets its own independent cursor.
pub struct EcsEventReaderState<E: Event> {
    /// Position in the read buffer up to which events have been consumed.
    /// Reset to 0 on each `update()` cycle (buffer swap) since the read
    /// buffer changes entirely.
    cursor: usize,
    _marker: PhantomData<fn() -> E>,
}

impl<E: Event> SystemParamState for EcsEventReaderState<E> {
    fn init(_world: &mut World) -> Self {
        Self {
            cursor: 0,
            _marker: PhantomData,
        }
    }
}

/// ECS system parameter for reading events of type `E`.
///
/// Wraps a reference to the [`Events<E>`] resource and a cursor stored in
/// the system's cached state. Each system using `EcsEventReader<E>` gets its
/// own cursor, so multiple systems can independently consume the same events.
///
/// # Example
///
/// ```ignore
/// fn damage_system(mut reader: EcsEventReader<DamageEvent>) {
///     for event in reader.read() {
///         println!("Damage: {}", event.amount);
///     }
/// }
/// ```
///
/// # Frame Lifecycle
///
/// Events written during frame N become readable after `Events::update()` is
/// called. The cursor automatically resets when the read buffer changes.
pub struct EcsEventReader<'w, 's, E: Event> {
    events: &'w Events<E>,
    cursor: &'s mut usize,
}

impl<'w, 's, E: Event> EcsEventReader<'w, 's, E> {
    /// Returns an iterator over unread events since this system last ran.
    ///
    /// Advances the cursor so that the next call to `read()` (in a
    /// subsequent system run) returns only newly available events.
    pub fn read(&mut self) -> impl Iterator<Item = &'w E> {
        let buffer = self.events.read_buffer();
        // Clamp cursor to buffer length in case the buffer shrank (new frame)
        let start = (*self.cursor).min(buffer.len());
        *self.cursor = buffer.len();
        buffer[start..].iter()
    }

    /// Returns `true` if there are no unread events for this system.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let buffer = self.events.read_buffer();
        (*self.cursor).min(buffer.len()) >= buffer.len()
    }

    /// Returns the number of unread events for this system.
    #[must_use]
    pub fn len(&self) -> usize {
        let buffer = self.events.read_buffer();
        buffer.len().saturating_sub(*self.cursor)
    }
}

impl<E: Event> SystemParam for EcsEventReader<'_, '_, E> {
    type State = EcsEventReaderState<E>;
    type Item<'w, 's> = EcsEventReader<'w, 's, E>;

    fn update_access(_state: &Self::State, access: &mut Access) {
        access.add_resource_read(ResourceId::of::<Events<E>>());
    }

    fn get_param<'w, 's>(state: &'s mut Self::State, world: &'w World) -> Self::Item<'w, 's> {
        let events: Res<'w, Events<E>> = world
            .resource::<Events<E>>()
            .expect("Events<E> resource not found. Insert it with world.insert_resource(Events::<E>::new()).");
        let events_ref: &'w Events<E> = events.into_inner();
        EcsEventReader {
            events: events_ref,
            cursor: &mut state.cursor,
        }
    }
}

/// `EcsEventReader<E>` is read-only.
impl<E: Event> ReadOnlySystemParam for EcsEventReader<'_, '_, E> {}

// =============================================================================
// EcsEventWriter<E> - Mutable Event SystemParam
// =============================================================================

/// Cached state for [`EcsEventWriter`].
///
/// Marker state with no data -- the writer does not need a cursor.
pub struct EcsEventWriterState<E: Event> {
    _marker: PhantomData<fn() -> E>,
}

impl<E: Event> SystemParamState for EcsEventWriterState<E> {
    fn init(_world: &mut World) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// ECS system parameter for writing events of type `E`.
///
/// Wraps a mutable reference to the [`Events<E>`] resource, providing
/// `send()` and `send_batch()` for emitting events.
///
/// # Example
///
/// ```ignore
/// fn spawn_system(mut writer: EcsEventWriter<SpawnEvent>) {
///     writer.send(SpawnEvent { entity_type: "enemy".into() });
/// }
/// ```
pub struct EcsEventWriter<'w, E: Event> {
    events: ResMut<'w, Events<E>>,
}

impl<E: Event> EcsEventWriter<'_, E> {
    /// Sends a single event.
    pub fn send(&mut self, event: E) {
        self.events.send(event);
    }

    /// Sends multiple events in batch.
    pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) {
        self.events.send_batch(events);
    }
}

impl<E: Event> SystemParam for EcsEventWriter<'_, E> {
    type State = EcsEventWriterState<E>;
    type Item<'w, 's> = EcsEventWriter<'w, E>;

    fn update_access(_state: &Self::State, access: &mut Access) {
        access.add_resource_write(ResourceId::of::<Events<E>>());
    }

    fn get_param<'w, 's>(_state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
        panic!("EcsEventWriter<E> requires mutable world access. Use get_param_mut instead.");
    }

    fn get_param_mut<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        let events = world
            .resource_mut::<Events<E>>()
            .expect("Events<E> resource not found. Insert it with world.insert_resource(Events::<E>::new()).");
        EcsEventWriter { events }
    }
}

// EcsEventWriter is NOT ReadOnlySystemParam -- intentionally omitted
