//! Tests for AssetHandle<A> and WeakAssetHandle<A>.

use crate::assets::{AssetId, AssetType};

use super::super::typed::{AssetHandle, WeakAssetHandle};
use super::common::{TestAudio, TestTexture};

mod asset_handle {
    use super::*;

    #[test]
    fn test_new() {
        let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
        assert_eq!(handle.index(), 42);
        assert_eq!(handle.generation(), 7);
    }

    #[test]
    fn test_invalid() {
        let handle: AssetHandle<TestTexture> = AssetHandle::INVALID;
        assert_eq!(handle.index(), u32::MAX);
        assert_eq!(handle.generation(), 0);
        assert!(!handle.is_valid());
    }

    #[test]
    fn test_is_valid() {
        let valid: AssetHandle<TestTexture> = AssetHandle::new(0, 1);
        assert!(valid.is_valid());

        let invalid: AssetHandle<TestTexture> = AssetHandle::INVALID;
        assert!(!invalid.is_valid());

        // Edge case: index=MAX but gen!=0 is still valid
        let edge: AssetHandle<TestTexture> = AssetHandle::new(u32::MAX, 1);
        assert!(edge.is_valid());
    }

    #[test]
    fn test_default() {
        let handle: AssetHandle<TestTexture> = Default::default();
        assert!(!handle.is_valid());
        assert_eq!(handle, AssetHandle::INVALID);
    }

    #[test]
    fn test_clone_copy() {
        let h1: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
        let h2 = h1; // Copy
        let h3 = h1.clone();

        assert_eq!(h1, h2);
        assert_eq!(h1, h3);
    }

    #[test]
    fn test_equality() {
        let h1: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
        let h2: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
        let h3: AssetHandle<TestTexture> = AssetHandle::new(1, 2);
        let h4: AssetHandle<TestTexture> = AssetHandle::new(2, 1);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3); // Different generation
        assert_ne!(h1, h4); // Different index
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(AssetHandle::<TestTexture>::new(1, 1));
        set.insert(AssetHandle::<TestTexture>::new(2, 1));

        assert_eq!(set.len(), 2);

        // Same handle shouldn't add again
        set.insert(AssetHandle::<TestTexture>::new(1, 1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_ord() {
        use std::collections::BTreeSet;

        let mut set = BTreeSet::new();
        set.insert(AssetHandle::<TestTexture>::new(3, 1));
        set.insert(AssetHandle::<TestTexture>::new(1, 1));
        set.insert(AssetHandle::<TestTexture>::new(2, 1));

        let vec: Vec<_> = set.iter().collect();
        assert!(vec[0].index() < vec[1].index());
        assert!(vec[1].index() < vec[2].index());
    }

    #[test]
    fn test_debug() {
        let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
        let debug_str = format!("{:?}", handle);
        assert!(debug_str.contains("AssetHandle"));
        assert!(debug_str.contains("TestTexture"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("7"));

        let invalid: AssetHandle<TestTexture> = AssetHandle::INVALID;
        let debug_str = format!("{:?}", invalid);
        assert!(debug_str.contains("INVALID"));
    }

    #[test]
    fn test_display() {
        let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
        assert_eq!(format!("{}", handle), "42:7");

        let invalid: AssetHandle<TestTexture> = AssetHandle::INVALID;
        assert_eq!(format!("{}", invalid), "INVALID");
    }

    #[test]
    fn test_to_u64() {
        let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
        let packed = handle.to_u64();

        // Upper 32 = generation, lower 32 = index
        assert_eq!(packed & 0xFFFFFFFF, 42);
        assert_eq!(packed >> 32, 7);
    }

    #[test]
    fn test_from_u64() {
        let packed: u64 = (7u64 << 32) | 42u64;
        let handle: AssetHandle<TestTexture> = AssetHandle::from_u64(packed);

        assert_eq!(handle.index(), 42);
        assert_eq!(handle.generation(), 7);
    }

    #[test]
    fn test_u64_roundtrip() {
        let original: AssetHandle<TestTexture> = AssetHandle::new(12345, 99);
        let packed = original.to_u64();
        let recovered: AssetHandle<TestTexture> = AssetHandle::from_u64(packed);

        assert_eq!(original, recovered);
    }

    #[test]
    fn test_from_into_u64() {
        let handle: AssetHandle<TestTexture> = AssetHandle::new(10, 20);
        let packed: u64 = handle.into();
        let recovered: AssetHandle<TestTexture> = packed.into();

        assert_eq!(handle, recovered);
    }

    #[test]
    fn test_untyped() {
        let typed: AssetHandle<TestTexture> = AssetHandle::new(5, 3);
        let untyped = typed.untyped();

        assert_eq!(untyped.index(), 5);
        assert_eq!(untyped.generation(), 3);
        assert_eq!(untyped.asset_id(), AssetId::of::<TestTexture>());
    }

    #[test]
    fn test_asset_id() {
        assert_eq!(
            AssetHandle::<TestTexture>::asset_id(),
            AssetId::of::<TestTexture>()
        );
        assert_ne!(
            AssetHandle::<TestTexture>::asset_id(),
            AssetHandle::<TestAudio>::asset_id()
        );
    }

    #[test]
    fn test_asset_type() {
        assert_eq!(AssetHandle::<TestTexture>::asset_type(), AssetType::Texture);
        assert_eq!(AssetHandle::<TestAudio>::asset_type(), AssetType::Audio);
    }

    #[test]
    fn test_size_and_align() {
        // Should be 8 bytes (2 x u32), PhantomData is zero-sized
        assert_eq!(std::mem::size_of::<AssetHandle<TestTexture>>(), 8);
        assert_eq!(std::mem::align_of::<AssetHandle<TestTexture>>(), 4);
    }

    #[test]
    fn test_is_send() {
        fn requires_send<T: Send>() {}
        requires_send::<AssetHandle<TestTexture>>();
    }

    #[test]
    fn test_is_sync() {
        fn requires_sync<T: Sync>() {}
        requires_sync::<AssetHandle<TestTexture>>();
    }
}

mod weak_asset_handle {
    use super::*;

    #[test]
    fn test_from_handle() {
        let strong: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
        let weak = WeakAssetHandle::from_handle(&strong);

        assert_eq!(weak.index(), 10);
        assert_eq!(weak.generation(), 5);
    }

    #[test]
    fn test_new() {
        let weak: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(42, 7);
        assert_eq!(weak.index(), 42);
        assert_eq!(weak.generation(), 7);
    }

    #[test]
    fn test_invalid() {
        let weak: WeakAssetHandle<TestTexture> = WeakAssetHandle::INVALID;
        assert!(!weak.is_valid());
    }

    #[test]
    fn test_upgrade() {
        let strong: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
        let weak = WeakAssetHandle::from_handle(&strong);
        let upgraded = weak.upgrade();

        assert_eq!(upgraded, strong);
    }

    #[test]
    fn test_clone_copy() {
        let w1: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 1);
        let w2 = w1; // Copy
        let w3 = w1.clone();

        assert_eq!(w1, w2);
        assert_eq!(w1, w3);
    }

    #[test]
    fn test_default() {
        let weak: WeakAssetHandle<TestTexture> = Default::default();
        assert!(!weak.is_valid());
    }

    #[test]
    fn test_equality() {
        let w1: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 1);
        let w2: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 1);
        let w3: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 2);

        assert_eq!(w1, w2);
        assert_ne!(w1, w3);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(WeakAssetHandle::<TestTexture>::new(1, 1));
        set.insert(WeakAssetHandle::<TestTexture>::new(2, 1));

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_debug() {
        let weak: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(42, 7);
        let debug_str = format!("{:?}", weak);
        assert!(debug_str.contains("WeakAssetHandle"));
        assert!(debug_str.contains("TestTexture"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_from_ref() {
        let strong: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
        let weak: WeakAssetHandle<TestTexture> = (&strong).into();

        assert_eq!(weak.index(), 10);
        assert_eq!(weak.generation(), 5);
    }

    #[test]
    fn test_size_and_align() {
        // Should be 8 bytes (2 x u32), PhantomData is zero-sized
        assert_eq!(std::mem::size_of::<WeakAssetHandle<TestTexture>>(), 8);
        assert_eq!(std::mem::align_of::<WeakAssetHandle<TestTexture>>(), 4);
    }

    #[test]
    fn test_is_send() {
        fn requires_send<T: Send>() {}
        requires_send::<WeakAssetHandle<TestTexture>>();
    }

    #[test]
    fn test_is_sync() {
        fn requires_sync<T: Sync>() {}
        requires_sync::<WeakAssetHandle<TestTexture>>();
    }
}
