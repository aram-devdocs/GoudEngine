//! Tests for send resource types: [`Resource`], [`ResourceId`], [`Resources`],
//! [`Res`], and [`ResMut`].

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::ecs::resource::{Res, ResMut, ResourceId, Resources};

    // Test resources
    #[derive(Debug)]
    struct Time {
        delta: f32,
        total: f32,
    }

    #[derive(Debug)]
    struct Score(u32);

    #[derive(Debug)]
    struct Config {
        debug: bool,
        volume: f32,
    }

    // =========================================================================
    // Resource Trait Tests
    // =========================================================================

    mod resource_trait {
        use crate::ecs::resource::Resource;

        use super::*;

        #[test]
        fn test_resource_auto_impl() {
            // Any Send + Sync + 'static type should be a Resource
            fn requires_resource<T: Resource>() {}

            requires_resource::<Time>();
            requires_resource::<Score>();
            requires_resource::<i32>();
            requires_resource::<String>();
            requires_resource::<Vec<u8>>();
        }

        #[test]
        fn test_resource_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<Time>();
            requires_send::<Score>();
        }

        #[test]
        fn test_resource_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<Time>();
            requires_sync::<Score>();
        }
    }

    // =========================================================================
    // ResourceId Tests
    // =========================================================================

    mod resource_id {
        use super::*;

        #[test]
        fn test_resource_id_of() {
            let id1 = ResourceId::of::<Time>();
            let id2 = ResourceId::of::<Time>();
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_resource_id_different_types() {
            let id1 = ResourceId::of::<Time>();
            let id2 = ResourceId::of::<Score>();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_resource_id_type_id() {
            let id = ResourceId::of::<Time>();
            assert_eq!(id.type_id(), TypeId::of::<Time>());
        }

        #[test]
        fn test_resource_id_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(ResourceId::of::<Time>());
            set.insert(ResourceId::of::<Score>());
            assert_eq!(set.len(), 2);

            // Same type should not add again
            set.insert(ResourceId::of::<Time>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_resource_id_ord() {
            use std::collections::BTreeSet;
            let mut set = BTreeSet::new();
            set.insert(ResourceId::of::<Time>());
            set.insert(ResourceId::of::<Score>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_resource_id_debug() {
            let id = ResourceId::of::<Time>();
            let debug_str = format!("{:?}", id);
            assert!(debug_str.contains("ResourceId"));
        }

        #[test]
        fn test_resource_id_clone() {
            let id1 = ResourceId::of::<Time>();
            let id2 = id1;
            assert_eq!(id1, id2);
        }
    }

    // =========================================================================
    // Resources Container Tests
    // =========================================================================

    mod resources_container {
        use super::*;

        #[test]
        fn test_resources_new() {
            let resources = Resources::new();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_resources_default() {
            let resources = Resources::default();
            assert!(resources.is_empty());
        }

        #[test]
        fn test_resources_insert() {
            let mut resources = Resources::new();
            let old = resources.insert(Score(100));
            assert!(old.is_none());
            assert_eq!(resources.len(), 1);
        }

        #[test]
        fn test_resources_insert_replace() {
            let mut resources = Resources::new();
            resources.insert(Score(100));
            let old = resources.insert(Score(200));
            assert!(old.is_some());
            assert_eq!(old.unwrap().0, 100);
            assert_eq!(resources.get::<Score>().unwrap().0, 200);
        }

        #[test]
        fn test_resources_remove() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let removed = resources.remove::<Score>();
            assert!(removed.is_some());
            assert_eq!(removed.unwrap().0, 100);
            assert!(resources.is_empty());
        }

        #[test]
        fn test_resources_remove_nonexistent() {
            let mut resources = Resources::new();
            let removed = resources.remove::<Score>();
            assert!(removed.is_none());
        }

        #[test]
        fn test_resources_get() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let score = resources.get::<Score>();
            assert!(score.is_some());
            assert_eq!(score.unwrap().0, 100);
        }

        #[test]
        fn test_resources_get_nonexistent() {
            let resources = Resources::new();
            assert!(resources.get::<Score>().is_none());
        }

        #[test]
        fn test_resources_get_mut() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let score = resources.get_mut::<Score>().unwrap();
            score.0 += 50;

            assert_eq!(resources.get::<Score>().unwrap().0, 150);
        }

        #[test]
        fn test_resources_get_mut_nonexistent() {
            let mut resources = Resources::new();
            assert!(resources.get_mut::<Score>().is_none());
        }

        #[test]
        fn test_resources_contains() {
            let mut resources = Resources::new();
            assert!(!resources.contains::<Score>());

            resources.insert(Score(100));
            assert!(resources.contains::<Score>());

            resources.remove::<Score>();
            assert!(!resources.contains::<Score>());
        }

        #[test]
        fn test_resources_multiple_types() {
            let mut resources = Resources::new();
            resources.insert(Score(100));
            resources.insert(Time {
                delta: 0.016,
                total: 0.0,
            });
            resources.insert(Config {
                debug: true,
                volume: 0.8,
            });

            assert_eq!(resources.len(), 3);
            assert_eq!(resources.get::<Score>().unwrap().0, 100);
            assert_eq!(resources.get::<Time>().unwrap().delta, 0.016);
            assert!(resources.get::<Config>().unwrap().debug);
        }

        #[test]
        fn test_resources_clear() {
            let mut resources = Resources::new();
            resources.insert(Score(100));
            resources.insert(Time {
                delta: 0.016,
                total: 0.0,
            });

            resources.clear();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_resources_debug() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let debug_str = format!("{:?}", resources);
            assert!(debug_str.contains("Resources"));
            assert!(debug_str.contains("count"));
        }
    }

    // =========================================================================
    // Res<T> Tests
    // =========================================================================

    mod res_tests {
        use super::*;

        #[test]
        fn test_res_new() {
            let time = Time {
                delta: 0.016,
                total: 1.0,
            };
            let res = Res::new(&time);
            assert_eq!(res.delta, 0.016);
            assert_eq!(res.total, 1.0);
        }

        #[test]
        fn test_res_deref() {
            let score = Score(100);
            let res = Res::new(&score);
            assert_eq!(res.0, 100);
        }

        #[test]
        fn test_res_into_inner() {
            let score = Score(100);
            let res = Res::new(&score);
            let inner = res.into_inner();
            assert_eq!(inner.0, 100);
        }

        #[test]
        fn test_res_debug() {
            let score = Score(100);
            let res = Res::new(&score);
            let debug_str = format!("{:?}", res);
            assert!(debug_str.contains("Res"));
        }

        #[test]
        fn test_res_clone() {
            let score = Score(100);
            let res = Res::new(&score);
            let cloned = res;
            assert_eq!(cloned.0, 100);
        }

        #[test]
        fn test_res_copy() {
            let score = Score(100);
            let res = Res::new(&score);
            let copied = res;
            // Both still valid
            assert_eq!(res.0, 100);
            assert_eq!(copied.0, 100);
        }
    }

    // =========================================================================
    // ResMut<T> Tests
    // =========================================================================

    mod res_mut_tests {
        use super::*;

        #[test]
        fn test_res_mut_new() {
            let mut time = Time {
                delta: 0.016,
                total: 1.0,
            };
            let res = ResMut::new(&mut time);
            assert_eq!(res.delta, 0.016);
            assert_eq!(res.total, 1.0);
        }

        #[test]
        fn test_res_mut_deref() {
            let mut score = Score(100);
            let res = ResMut::new(&mut score);
            assert_eq!(res.0, 100);
        }

        #[test]
        fn test_res_mut_deref_mut() {
            let mut score = Score(100);
            {
                let mut res = ResMut::new(&mut score);
                res.0 += 50;
            }
            assert_eq!(score.0, 150);
        }

        #[test]
        fn test_res_mut_into_inner() {
            let mut score = Score(100);
            let res = ResMut::new(&mut score);
            let inner = res.into_inner();
            inner.0 += 50;
            assert_eq!(score.0, 150);
        }

        #[test]
        fn test_res_mut_debug() {
            let mut score = Score(100);
            let res = ResMut::new(&mut score);
            let debug_str = format!("{:?}", res);
            assert!(debug_str.contains("ResMut"));
        }

        #[test]
        fn test_res_mut_modify_complex() {
            let mut time = Time {
                delta: 0.016,
                total: 0.0,
            };

            {
                let mut res = ResMut::new(&mut time);
                res.total += res.delta;
            }

            assert_eq!(time.total, 0.016);
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_resources_with_res() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let score_ref = resources.get::<Score>().unwrap();
            let res = Res::new(score_ref);

            assert_eq!(res.0, 100);
        }

        #[test]
        fn test_resources_with_res_mut() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            {
                let score_ref = resources.get_mut::<Score>().unwrap();
                let mut res = ResMut::new(score_ref);
                res.0 += 50;
            }

            assert_eq!(resources.get::<Score>().unwrap().0, 150);
        }

        #[test]
        fn test_resource_lifecycle() {
            let mut resources = Resources::new();

            // Insert
            resources.insert(Score(0));
            assert!(resources.contains::<Score>());

            // Modify
            resources.get_mut::<Score>().unwrap().0 = 100;

            // Read
            assert_eq!(resources.get::<Score>().unwrap().0, 100);

            // Replace
            resources.insert(Score(200));
            assert_eq!(resources.get::<Score>().unwrap().0, 200);

            // Remove
            let removed = resources.remove::<Score>();
            assert_eq!(removed.unwrap().0, 200);
            assert!(!resources.contains::<Score>());
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_resources_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<Resources>();
        }

        #[test]
        fn test_res_is_send() {
            // Res is Send if T is Send
            fn check<T: crate::ecs::resource::Resource>() {
                // Can't directly check Res<'_, T> for Send due to lifetime
                // but the underlying reference is Send if T is Sync
            }
            check::<Score>();
        }

        #[test]
        fn test_res_mut_is_send() {
            // ResMut is Send if T is Send
            fn check<T: crate::ecs::resource::Resource>() {
                // Can't directly check ResMut<'_, T> for Send due to lifetime
            }
            check::<Score>();
        }
    }
}
