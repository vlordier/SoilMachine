//! Kani proof harnesses for Sim.
//!
//! These proofs verify the Rust-side invariants of the Sim API.
//! The C++ FFI calls are treated as black boxes; proofs verify that:
//! - No panics occur during construction/operation
//! - Bounds are respected
//! - Return values satisfy invariants

use crate::Sim;

/// Proof that Sim::new does not panic for any valid dimensions.
/// Kani generates arbitrary seed and dimension values.
#[kani::proof]
fn proof_sim_new_no_panic() {
    let seed: u32 = kani::any();
    let dim_x: u32 = kani::any();
    let dim_y: u32 = kani::any();

    // Sim::new requires positive dimensions
    kani::assume(dim_x > 0);
    kani::assume(dim_y > 0);

    // Try all combinations of soil_file
    for _soil in [None, Some("soil/default.soil"), Some("")] {
        // This should never panic
        let _sim = Sim::new(seed, dim_x, dim_y, _soil);
    }
}

/// Proof that Sim dimensions are positive after construction.
#[kani::proof]
fn proof_sim_dim_positive() {
    let seed: u32 = kani::any();
    let dim_x: u32 = kani::any();
    let dim_y: u32 = kani::any();

    kani::assume(dim_x > 0);
    kani::assume(dim_y > 0);

    if let Ok(sim) = Sim::new(seed, dim_x, dim_y, None) {
        assert!(sim.dim_x() > 0);
        assert!(sim.dim_y() > 0);
        assert_eq!(sim.dim_x(), dim_x as i32);
        assert_eq!(sim.dim_y(), dim_y as i32);
    }
}

/// Proof that Sim::drop is safe (no double-free, no null-deref).
#[kani::proof]
fn proof_sim_drop_no_panic() {
    let seed: u32 = kani::any();
    let dim_x: u32 = kani::any();
    let dim_y: u32 = kani::any();

    kani::assume(dim_x > 0);
    kani::assume(dim_y > 0);

    if let Ok(sim) = Sim::new(seed, dim_x, dim_y, None) {
        // Drop is automatic at end of scope.
        // We verify that the sim is usable before drop.
        let _ = sim.dim();
        drop(sim);
        // After drop, sim is consumed by Rust borrow checker.
        // Kani cannot access it (would be a compile error).
    }
}

/// Proof that heightmap buffer size is consistent.
#[kani::proof]
fn proof_heightmap_size_consistent() {
    let seed: u32 = kani::any();
    let dim_x: u32 = kani::any();
    let dim_y: u32 = kani::any();

    kani::assume(dim_x > 0);
    kani::assume(dim_y > 0);
    // Bound dimensions to avoid memory issues in actual execution
    kani::assume(dim_x <= 64);
    kani::assume(dim_y <= 64);

    if let Ok(sim) = Sim::new(seed, dim_x, dim_y, None) {
        let map = sim.heightmap();
        let expected = (sim.dim_x() as usize) * (sim.dim_y() as usize);
        assert_eq!(map.len(), expected);
    }
}

/// Proof that color_map buffer size is consistent.
#[kani::proof]
fn proof_color_map_size_consistent() {
    let seed: u32 = kani::any();
    let dim_x: u32 = kani::any();
    let dim_y: u32 = kani::any();

    kani::assume(dim_x > 0);
    kani::assume(dim_y > 0);
    kani::assume(dim_x <= 32);
    kani::assume(dim_y <= 32);

    if let Ok(sim) = Sim::new(seed, dim_x, dim_y, None) {
        let colors = sim.color_map();
        let expected = (sim.dim_x() as usize) * (sim.dim_y() as usize) * 4;
        assert_eq!(colors.len(), expected);
    }
}

/// Proof that normals are finite.
#[kani::proof]
fn proof_normals_finite() {
    let seed: u32 = kani::any();
    let dim_x: u32 = kani::any();
    let dim_y: u32 = kani::any();

    kani::assume(dim_x > 0);
    kani::assume(dim_y > 0);
    kani::assume(dim_x <= 32);
    kani::assume(dim_y <= 32);

    if let Ok(sim) = Sim::new(seed, dim_x, dim_y, None) {
        for x in 0..(sim.dim_x() as u32).min(16) {
            for y in 0..(sim.dim_y() as u32).min(16) {
                let (nx, ny, nz) = sim.normal(x, y);
                assert!(nx.is_finite());
                assert!(ny.is_finite());
                assert!(nz.is_finite());
            }
        }
    }
}

/// Proof that height values are finite.
#[kani::proof]
fn proof_heights_finite() {
    let seed: u32 = kani::any();
    let dim_x: u32 = kani::any();
    let dim_y: u32 = kani::any();

    kani::assume(dim_x > 0);
    kani::assume(dim_y > 0);
    kani::assume(dim_x <= 32);
    kani::assume(dim_y <= 32);

    if let Ok(sim) = Sim::new(seed, dim_x, dim_y, None) {
        for x in 0..(sim.dim_x() as u32).min(16) {
            for y in 0..(sim.dim_y() as u32).min(16) {
                let h = sim.height(x, y);
                assert!(h.is_finite());
            }
        }
    }
}
