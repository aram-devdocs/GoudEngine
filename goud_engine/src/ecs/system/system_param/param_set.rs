//! [`ParamSet`] — disjoint access to multiple conflicting queries.

// =============================================================================
// ParamSet
// =============================================================================

/// A parameter that allows disjoint access to multiple queries with conflicting access.
///
/// Normally, two queries that access the same component cannot be used together
/// because of Rust's aliasing rules. `ParamSet` solves this by ensuring only one
/// query can be accessed at a time.
///
/// # Example
///
/// ```ignore
/// // Future: When Query is implemented
/// fn conflicting_access(
///     // These would conflict: both access Position
///     // query1: Query<&mut Position>,
///     // query2: Query<&Position, With<Player>>,
///
///     // Solution: use ParamSet
///     mut set: ParamSet<(Query<&mut Position>, Query<&Position, With<Player>>)>,
/// ) {
///     // Access one at a time
///     for pos in set.p0().iter_mut() {
///         pos.x += 1.0;
///     }
///     // p0 is dropped before p1 is accessed
///     for pos in set.p1().iter() {
///         println!("Player position: {:?}", pos);
///     }
/// }
/// ```
///
/// # Note
///
/// This is a placeholder for the full ParamSet implementation, which requires
/// Query to be implemented first (Step 3.1.3).
#[derive(Debug)]
pub struct ParamSet<T> {
    _marker: std::marker::PhantomData<T>,
}

// ParamSet implementations will be added when Query is implemented (Step 3.1.3)
