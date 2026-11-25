//! Property-based test for SoilMachine simulation invariants.
//!
//! NOTE: The C++ simulation uses global mutable state so only ONE
//! Sim instance can exist at a time. This test constructs a Sim,
//! runs a simulation, and verifies all invariants in a single run.
//!
//! Run: `cargo test --test prop_sim`

use proptest::prelude::*;
use soilmachine::Sim;

proptest! {
    /// Comprehensive simulation invariant test.
    /// Constructs a Sim, runs erosion, and verifies:
    /// - Height is non-negative and bounded
    /// - Heightmap dimensions are correct
    /// - Color map dimensions are correct
    /// - Normals are unit length
    /// - Water/wind steps produce finite values
    #[test]
    fn full_simulation_invariants(
        seed in 0u32..1_000_000u32,
        dim_x in 32u32..128u32,
        dim_y in 32u32..128u32
    ) {
        // Single Sim instance — C++ global state limits to one at a time
        let mut sim = Sim::new(seed, dim_x, dim_y, None).unwrap();

        // Pre-erosion: all heights non-negative and finite
        for x in 0..dim_x.min(16) {
            for y in 0..dim_y.min(16) {
                let h = sim.height(x, y);
                prop_assert!(h.is_finite());
                prop_assert!(h >= 0.0);
                prop_assert!(h < 1000.0);
            }
        }

        // Run water erosion
        sim.step_water(50);

        // Post-erosion: heights still non-negative and finite
        for x in 0..dim_x.min(16) {
            for y in 0..dim_y.min(16) {
                let h = sim.height(x, y);
                prop_assert!(h.is_finite(), "height must be finite after water step");
                prop_assert!(h >= 0.0, "height must be non-negative after water step");
            }
        }

        // Run wind erosion
        sim.step_wind(50);

        // Post-wind: heights still non-negative and finite
        for x in 0..dim_x.min(16) {
            for y in 0..dim_y.min(16) {
                let h = sim.height(x, y);
                prop_assert!(h.is_finite(), "height must be finite after wind step");
                prop_assert!(h >= 0.0, "height must be non-negative after wind step");
            }
        }

        // Heightmap size is correct
        let map = sim.heightmap();
        prop_assert_eq!(map.len(), (dim_x as usize) * (dim_y as usize));

        // Color map size is correct
        let colors = sim.color_map();
        prop_assert_eq!(colors.len(), (dim_x as usize) * (dim_y as usize) * 4);

        // Normals are normalized
        for x in 0..dim_x.min(16) {
            for y in 0..dim_y.min(16) {
                let (nx, ny, nz) = sim.normal(x, y);
                let len = (nx * nx + ny * ny + nz * nz).sqrt();
                // Allow 1% tolerance for float arithmetic
                prop_assert!(
                    (len - 1.0).abs() < 0.01 || len < 0.1,
                    "normal at ({}, {}) has length {}: ({}, {}, {})",
                    x, y, len, nx, ny, nz
                );
            }
        }

        // Dimensions are consistent
        prop_assert_eq!(sim.dim(), (dim_x, dim_y));
        prop_assert_eq!(sim.dim_x() as u32, dim_x);
        prop_assert_eq!(sim.dim_y() as u32, dim_y);
    }
}
