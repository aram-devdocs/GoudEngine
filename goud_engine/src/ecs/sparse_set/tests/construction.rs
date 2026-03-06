//! Construction tests for [`SparseSet`].

#[cfg(test)]
mod tests {
    use crate::ecs::SparseSet;

    #[test]
    fn test_new() {
        let set: SparseSet<i32> = SparseSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let set: SparseSet<i32> = SparseSet::with_capacity(100);
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_default() {
        let set: SparseSet<i32> = SparseSet::default();
        assert!(set.is_empty());
    }
}
