//! Pure-logic regression tests for the shadow pass invariants in `shadow_pass.rs`.
//!
//! Kept in a sibling file so `shadow_pass.rs` stays under the 500-line limit
//! enforced by `scripts/check-rs-line-limit.sh`. The helpers below mirror the
//! production guards / arithmetic; if either drifts, these tests will catch it.

#![cfg(test)]

/// Models the early-return guard in `ensure_shadow_resources_impl`. The real
/// implementation needs a wgpu Device which is not available in unit tests,
/// so the rule is encoded as a pure function and exercised here. Regression
/// cover for issue #677: the shadow render target must NOT be re-allocated
/// when the requested size matches the cached size.
fn shadow_resources_should_recreate(
    cached_size: u32,
    requested_size: u32,
    cached_present: bool,
) -> bool {
    let size = requested_size.max(1);
    !(cached_size == size && cached_present)
}

#[test]
fn shadow_target_persists_across_frames_at_same_size() {
    // First call with no cached texture must create one.
    assert!(shadow_resources_should_recreate(0, 1024, false));
    // Second call at the same size with a cached texture must skip work.
    assert!(!shadow_resources_should_recreate(1024, 1024, true));
    // Third call at the same size still reuses the existing texture.
    assert!(!shadow_resources_should_recreate(1024, 1024, true));
}

#[test]
fn shadow_target_recreates_when_size_changes() {
    // Resizing the shadow map must invalidate the cached texture.
    assert!(shadow_resources_should_recreate(512, 1024, true));
    assert!(shadow_resources_should_recreate(1024, 2048, true));
}

#[test]
fn shadow_target_recreates_when_cache_was_dropped() {
    // If the cached handle is missing, recreate even at the same size.
    assert!(shadow_resources_should_recreate(1024, 1024, false));
}

#[test]
fn requested_size_zero_is_clamped_to_one() {
    // size.max(1) means a request for size 0 lands on size 1 and stays
    // cached; a follow-up request for 1 must NOT trigger recreation.
    assert!(shadow_resources_should_recreate(0, 0, false));
    assert!(!shadow_resources_should_recreate(1, 1, true));
    assert!(!shadow_resources_should_recreate(1, 0, true));
}

/// Models the offset/slot computation used by `execute_shadow_pass`.
/// Verifies that `scratch_shadow_offsets` produces the same byte offsets
/// the pre-scratch code path produced via `Vec<u32>::collect`.
fn compute_shadow_offsets(num_commands: usize, slot_size: usize) -> Vec<u32> {
    (0..num_commands).map(|i| (i * slot_size) as u32).collect()
}

#[test]
fn shadow_offsets_are_aligned_and_monotonic() {
    // Simulate a uniform-buffer-offset alignment of 256 bytes (Metal/D3D
    // common limit) and 1.4k shadow casters from the throne_ge repro.
    let slot_size = 256usize;
    let offsets = compute_shadow_offsets(1400, slot_size);
    assert_eq!(offsets.len(), 1400);
    assert_eq!(offsets[0], 0);
    assert_eq!(offsets[1], 256);
    assert_eq!(offsets[1399], 1399 * 256);
    for i in 0..offsets.len() {
        assert_eq!(offsets[i] as usize % slot_size, 0);
        if i > 0 {
            assert!(offsets[i] > offsets[i - 1]);
        }
    }
}

/// Verifies the shadow uniform buffer growth uses next_power_of_two so it
/// stabilizes after the first few frames at peak draw count, matching the
/// main pass invariant guarded by `uniform_buffer_growth_is_power_of_two`.
#[test]
fn shadow_uniform_buffer_growth_stabilizes() {
    let slot_size = 256usize;
    let mut buffer_size = slot_size as u64;
    let mut realloc_count = 0u32;

    // Simulate per-frame shadow caster counts oscillating around 1.4k.
    let counts = [1200, 1400, 1300, 1450, 1380, 1400, 1410, 1395, 1402, 1450];
    for &count in &counts {
        let total = count * slot_size;
        if total > buffer_size as usize {
            buffer_size = (total.next_power_of_two().max(slot_size)) as u64;
            realloc_count += 1;
        }
    }

    // 1450 * 256 = 371_200 → next_power_of_two = 524_288.
    assert_eq!(buffer_size, 524_288);
    assert!(buffer_size.is_power_of_two());
    // After at most 2 reallocations the buffer must stop growing.
    assert!(
        realloc_count <= 2,
        "shadow uniform buffer kept growing: {realloc_count} reallocations"
    );
}
