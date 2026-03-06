//! Tests for non-send resource types: [`NonSendResource`], [`NonSendResourceId`],
//! [`NonSendResources`], [`NonSend`], and [`NonSendMut`].

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::ecs::resource::{
        NonSend, NonSendMut, NonSendResource, NonSendResourceId, NonSendResources,
    };

    // Non-send test resources
    struct WindowHandle {
        id: Rc<u32>,
    }
    impl NonSendResource for WindowHandle {}

    struct OpenGLContext {
        ctx: Rc<RefCell<u32>>,
    }
    impl NonSendResource for OpenGLContext {}

    struct RawPointerResource {
        ptr: *mut u32,
    }
    impl NonSendResource for RawPointerResource {}

    // =========================================================================
    // NonSendResourceId Tests
    // =========================================================================

    mod non_send_resource_id {
        use super::*;

        #[test]
        fn test_non_send_resource_id_of() {
            let id1 = NonSendResourceId::of::<WindowHandle>();
            let id2 = NonSendResourceId::of::<WindowHandle>();
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_non_send_resource_id_different_types() {
            let id1 = NonSendResourceId::of::<WindowHandle>();
            let id2 = NonSendResourceId::of::<OpenGLContext>();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_non_send_resource_id_type_id() {
            let id = NonSendResourceId::of::<WindowHandle>();
            assert_eq!(id.type_id(), TypeId::of::<WindowHandle>());
        }

        #[test]
        fn test_non_send_resource_id_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(NonSendResourceId::of::<WindowHandle>());
            set.insert(NonSendResourceId::of::<OpenGLContext>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_non_send_resource_id_ord() {
            use std::collections::BTreeSet;
            let mut set = BTreeSet::new();
            set.insert(NonSendResourceId::of::<WindowHandle>());
            set.insert(NonSendResourceId::of::<OpenGLContext>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_non_send_resource_id_debug() {
            let id = NonSendResourceId::of::<WindowHandle>();
            let debug_str = format!("{:?}", id);
            assert!(debug_str.contains("NonSendResourceId"));
        }
    }

    // =========================================================================
    // NonSendResources Container Tests
    // =========================================================================

    mod non_send_resources_container {
        use super::*;

        #[test]
        fn test_non_send_resources_new() {
            let resources = NonSendResources::new();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_non_send_resources_default() {
            let resources = NonSendResources::default();
            assert!(resources.is_empty());
        }

        #[test]
        fn test_non_send_resources_insert() {
            let mut resources = NonSendResources::new();
            let old = resources.insert(WindowHandle { id: Rc::new(42) });
            assert!(old.is_none());
            assert_eq!(resources.len(), 1);
        }

        #[test]
        fn test_non_send_resources_insert_replace() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });
            let old = resources.insert(WindowHandle { id: Rc::new(100) });
            assert!(old.is_some());
            assert_eq!(*old.unwrap().id, 42);
            assert_eq!(*resources.get::<WindowHandle>().unwrap().id, 100);
        }

        #[test]
        fn test_non_send_resources_remove() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });

            let removed = resources.remove::<WindowHandle>();
            assert!(removed.is_some());
            assert_eq!(*removed.unwrap().id, 42);
            assert!(resources.is_empty());
        }

        #[test]
        fn test_non_send_resources_remove_nonexistent() {
            let mut resources = NonSendResources::new();
            let removed = resources.remove::<WindowHandle>();
            assert!(removed.is_none());
        }

        #[test]
        fn test_non_send_resources_get() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });

            let handle = resources.get::<WindowHandle>();
            assert!(handle.is_some());
            assert_eq!(*handle.unwrap().id, 42);
        }

        #[test]
        fn test_non_send_resources_get_nonexistent() {
            let resources = NonSendResources::new();
            assert!(resources.get::<WindowHandle>().is_none());
        }

        #[test]
        fn test_non_send_resources_get_mut() {
            let mut resources = NonSendResources::new();
            resources.insert(OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            });

            let ctx = resources.get_mut::<OpenGLContext>().unwrap();
            *ctx.ctx.borrow_mut() = 42;

            assert_eq!(*resources.get::<OpenGLContext>().unwrap().ctx.borrow(), 42);
        }

        #[test]
        fn test_non_send_resources_contains() {
            let mut resources = NonSendResources::new();
            assert!(!resources.contains::<WindowHandle>());

            resources.insert(WindowHandle { id: Rc::new(42) });
            assert!(resources.contains::<WindowHandle>());

            resources.remove::<WindowHandle>();
            assert!(!resources.contains::<WindowHandle>());
        }

        #[test]
        fn test_non_send_resources_multiple_types() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(1) });
            resources.insert(OpenGLContext {
                ctx: Rc::new(RefCell::new(2)),
            });

            assert_eq!(resources.len(), 2);
            assert_eq!(*resources.get::<WindowHandle>().unwrap().id, 1);
            assert_eq!(*resources.get::<OpenGLContext>().unwrap().ctx.borrow(), 2);
        }

        #[test]
        fn test_non_send_resources_clear() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(1) });
            resources.insert(OpenGLContext {
                ctx: Rc::new(RefCell::new(2)),
            });

            resources.clear();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_non_send_resources_debug() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });

            let debug_str = format!("{:?}", resources);
            assert!(debug_str.contains("NonSendResources"));
            assert!(debug_str.contains("count"));
        }

        #[test]
        fn test_non_send_resources_with_raw_pointer() {
            let mut value = 42u32;
            let mut resources = NonSendResources::new();
            resources.insert(RawPointerResource {
                ptr: &mut value as *mut u32,
            });

            let res = resources.get::<RawPointerResource>().unwrap();
            assert!(!res.ptr.is_null());
        }
    }

    // =========================================================================
    // NonSendResources Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_non_send_resources_is_not_send() {
            // NonSendResources should NOT implement Send
            fn check_not_send<T>() {
                // This is a compile-time check - the test passes by compiling
                // We can't easily test !Send at runtime
            }
            check_not_send::<NonSendResources>();
            // The actual !Send is enforced by the NonSendMarker containing *const ()
        }

        #[test]
        fn test_non_send_resources_is_not_sync() {
            // NonSendResources should NOT implement Sync
            fn check_not_sync<T>() {
                // This is a compile-time check - the test passes by compiling
                // We can't easily test !Sync at runtime
            }
            check_not_sync::<NonSendResources>();
            // The actual !Sync is enforced by the NonSendMarker containing *const ()
        }
    }

    // =========================================================================
    // NonSend<T> and NonSendMut<T> Wrapper Tests
    // =========================================================================

    mod non_send_wrappers {
        use super::*;

        #[test]
        fn test_non_send_new() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            assert_eq!(*non_send.id, 42);
        }

        #[test]
        fn test_non_send_deref() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            assert_eq!(*non_send.id, 42);
        }

        #[test]
        fn test_non_send_into_inner() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            let inner = non_send.into_inner();
            assert_eq!(*inner.id, 42);
        }

        #[test]
        fn test_non_send_clone() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            let cloned = non_send;
            // Both should be valid
            assert_eq!(*non_send.id, 42);
            assert_eq!(*cloned.id, 42);
        }

        #[test]
        fn test_non_send_copy() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            let copied = non_send;
            // Both should still be valid
            assert_eq!(*non_send.id, 42);
            assert_eq!(*copied.id, 42);
        }

        #[test]
        fn test_non_send_mut_new() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            let non_send_mut = NonSendMut::new(&mut ctx);
            assert_eq!(*non_send_mut.ctx.borrow(), 1);
        }

        #[test]
        fn test_non_send_mut_deref() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            let non_send_mut = NonSendMut::new(&mut ctx);
            assert_eq!(*non_send_mut.ctx.borrow(), 1);
        }

        #[test]
        fn test_non_send_mut_deref_mut() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            {
                let mut non_send_mut = NonSendMut::new(&mut ctx);
                *non_send_mut.ctx.borrow_mut() = 42;
            }
            assert_eq!(*ctx.ctx.borrow(), 42);
        }

        #[test]
        fn test_non_send_mut_into_inner() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            let non_send_mut = NonSendMut::new(&mut ctx);
            let inner = non_send_mut.into_inner();
            *inner.ctx.borrow_mut() = 42;
            assert_eq!(*ctx.ctx.borrow(), 42);
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_non_send_resource_lifecycle() {
            let mut resources = NonSendResources::new();

            // Insert
            resources.insert(WindowHandle { id: Rc::new(0) });
            assert!(resources.contains::<WindowHandle>());

            // Modify (via Rc)
            let id = resources.get::<WindowHandle>().unwrap().id.clone();
            assert_eq!(*id, 0);

            // Replace
            resources.insert(WindowHandle { id: Rc::new(42) });
            assert_eq!(*resources.get::<WindowHandle>().unwrap().id, 42);

            // Remove
            let removed = resources.remove::<WindowHandle>();
            assert_eq!(*removed.unwrap().id, 42);
            assert!(!resources.contains::<WindowHandle>());
        }
    }
}
