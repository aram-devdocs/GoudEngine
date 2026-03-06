//! [`SystemParam`] implementation for [`Query`].

use crate::ecs::system::{ReadOnlySystemParam, SystemParam, SystemParamState};
use crate::ecs::World;

use super::fetch::{Access, ReadOnlyWorldQuery, WorldQuery};
use super::query_type::Query;

// =============================================================================
// Query SystemParam Implementation
// =============================================================================

/// Cached state for Query as a system parameter.
///
/// This stores the initialized query and filter states, allowing efficient
/// reuse across multiple system runs.
#[derive(Clone)]
pub struct QuerySystemParamState<Q: WorldQuery, F: WorldQuery> {
    /// Cached query state.
    pub(crate) query_state: Q::State,
    /// Cached filter state.
    pub(crate) filter_state: F::State,
}

impl<Q: WorldQuery, F: WorldQuery> std::fmt::Debug for QuerySystemParamState<Q, F>
where
    Q::State: std::fmt::Debug,
    F::State: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuerySystemParamState")
            .field("query_state", &self.query_state)
            .field("filter_state", &self.filter_state)
            .finish()
    }
}

// SAFETY: QuerySystemParamState is Send + Sync if the underlying states are
unsafe impl<Q: WorldQuery, F: WorldQuery> Send for QuerySystemParamState<Q, F>
where
    Q::State: Send,
    F::State: Send,
{
}

unsafe impl<Q: WorldQuery, F: WorldQuery> Sync for QuerySystemParamState<Q, F>
where
    Q::State: Sync,
    F::State: Sync,
{
}

impl<Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParamState
    for QuerySystemParamState<Q, F>
where
    Q::State: Clone + Send + Sync,
    F::State: Clone + Send + Sync,
{
    fn init(world: &mut World) -> Self {
        Self {
            query_state: Q::init_state(world),
            filter_state: F::init_state(world),
        }
    }
}

/// `Query<Q, F>` as a system parameter.
///
/// This allows functions to declare queries as parameters:
///
/// ```ignore
/// fn my_system(query: Query<&Position, With<Player>>) {
///     for pos in query.iter() {
///         println!("Player position: {:?}", pos);
///     }
/// }
/// ```
impl<Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParam for Query<Q, F>
where
    Q::State: Clone + Send + Sync + 'static,
    F::State: Clone + Send + Sync + 'static,
{
    type State = QuerySystemParamState<Q, F>;
    type Item<'w, 's> = Query<Q, F>;

    fn update_access(state: &Self::State, access: &mut Access) {
        // Add component access from the query
        for id in Q::component_access(&state.query_state) {
            // For now, we add as read. Mutable queries need special handling.
            // The WorldQuery implementation determines read vs write.
            access.add_read(id);
        }
        // Filters don't count as access (they only check archetype)
    }

    fn get_param<'w, 's>(state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
        Query::from_state(state.query_state.clone(), state.filter_state.clone())
    }

    fn get_param_mut<'w, 's>(
        state: &'s mut Self::State,
        _world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        // For mutable access, we still return a Query that can iterate mutably
        Query::from_state(state.query_state.clone(), state.filter_state.clone())
    }
}

/// Query with read-only data query is a read-only system parameter.
impl<Q: ReadOnlyWorldQuery + 'static, F: ReadOnlyWorldQuery + 'static> ReadOnlySystemParam
    for Query<Q, F>
where
    Q::State: Clone + Send + Sync + 'static,
    F::State: Clone + Send + Sync + 'static,
{
}
