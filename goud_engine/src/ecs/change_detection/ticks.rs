//! Tick type and ComponentTicks for change detection.

/// A monotonically increasing tick used for change detection.
///
/// Ticks are compared to determine whether a component was added or
/// changed since a particular point in time. A tick with a higher value
/// is considered "newer."
///
/// # Example
///
/// ```
/// use goud_engine::ecs::change_detection::Tick;
///
/// let t1 = Tick::new(1);
/// let t2 = Tick::new(5);
///
/// assert!(t2.is_newer_than(t1));
/// assert!(!t1.is_newer_than(t2));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tick(u32);

impl Tick {
    /// Creates a new tick with the given value.
    #[inline]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw `u32` value of this tick.
    #[inline]
    pub fn get(self) -> u32 {
        self.0
    }

    /// Returns `true` if this tick is strictly newer than `other`.
    #[inline]
    pub fn is_newer_than(self, other: Tick) -> bool {
        self.0 > other.0
    }
}

impl From<u32> for Tick {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Tick> for u32 {
    #[inline]
    fn from(tick: Tick) -> Self {
        tick.0
    }
}

/// Stores the `added` and `changed` ticks for a single component instance.
///
/// When a component is first inserted, both ticks are set to the current
/// world tick. Subsequent mutations update only `changed`.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::change_detection::{ComponentTicks, Tick};
///
/// let ticks = ComponentTicks::new(Tick::new(3));
/// assert_eq!(ticks.added(), Tick::new(3));
/// assert_eq!(ticks.changed(), Tick::new(3));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentTicks {
    added: Tick,
    changed: Tick,
}

impl ComponentTicks {
    /// Creates new ticks where both `added` and `changed` equal `tick`.
    #[inline]
    pub fn new(tick: Tick) -> Self {
        Self {
            added: tick,
            changed: tick,
        }
    }

    /// Returns the tick at which the component was added.
    #[inline]
    pub fn added(&self) -> Tick {
        self.added
    }

    /// Returns the tick at which the component was last changed.
    #[inline]
    pub fn changed(&self) -> Tick {
        self.changed
    }

    /// Sets the `changed` tick to `tick`.
    #[inline]
    pub fn set_changed(&mut self, tick: Tick) {
        self.changed = tick;
    }

    /// Returns `true` if the component was added after `last_change_tick`.
    #[inline]
    pub fn is_added(&self, last_change_tick: Tick) -> bool {
        self.added.is_newer_than(last_change_tick)
    }

    /// Returns `true` if the component was changed after `last_change_tick`.
    #[inline]
    pub fn is_changed(&self, last_change_tick: Tick) -> bool {
        self.changed.is_newer_than(last_change_tick)
    }
}
