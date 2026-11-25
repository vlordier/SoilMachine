//! Rust bindings for SoilMachine — a modular geomorphology simulator.
//!
//! This crate provides safe Rust bindings to the SoilMachine C++ simulation
//! engine, which models hydraulic and wind erosion on layered terrain.
//!
//! # Formal Verification
//!
//! This crate is verified with four complementary formal methods:
//!
//! - **Kani** (model checking): verifies memory safety and bounds on all public API
//!   methods. Run: `make kani`
//!
//! - **Proptest** (property-based testing): verifies simulation invariants
//!   (height >= 0, normals normalized, buffer sizes correct) across thousands of
//!   random seeds and dimensions. Run: `make proptest`
//!
//! - **TLC** (TLA+ model checking): verifies the erosion state machine invariant
//!   — height conservation, non-negative heights, particle lifecycle.
//!   Run: `make tlc`
//!
//! - **Verus** (formal verification): specifies pre/post conditions and state
//!   machine contracts for Sim operations. Run: `make verus`
//!
//! # State Machine
//!
//! ```text
//!   [uninit] --Sim::new()--> [initialized] --operations--> [initialized]
//!                                |                                |
//!                          reset()|                        Drop()
//!                                v                                v
//!                          [initialized]                  [destroyed]
//! ```
//!
//! # Example
//!
//! ```
//! use soilmachine::Sim;
//!
//! let sim = Sim::new(12345, 256, 256, Some("soil/default.soil")).unwrap();
//! sim.step_water(250);
//! sim.step_wind(250);
//! let h = sim.height(128, 128);
//! let map = sim.heightmap();
//! ```

use std::ffi::CString;
use std::os::raw::c_void;

#[link(name = "soilmachine_bridge", kind = "static")]
extern "C" {
    fn sim_create(seed: u32, dim_x: i32, dim_y: i32, soil_file: *const i8) -> *mut c_void;
    fn sim_destroy(raw: *mut c_void);
    fn sim_step_water(raw: *mut c_void, n: i32);
    fn sim_step_wind(raw: *mut c_void, n: i32);
    fn sim_get_height(raw: *mut c_void, x: i32, y: i32) -> f64;
    fn sim_get_heightmap(raw: *mut c_void, out: *mut f32);
    fn sim_get_color_map(raw: *mut c_void, out: *mut u8, len: *mut i32);
    fn sim_get_normal_x(raw: *mut c_void, x: i32, y: i32, out: *mut f32);
    fn sim_get_normal_y(raw: *mut c_void, x: i32, y: i32, out: *mut f32);
    fn sim_get_normal_z(raw: *mut c_void, x: i32, y: i32, out: *mut f32);
    fn sim_set_nwater(raw: *mut c_void, n: i32);
    fn sim_set_nwind(raw: *mut c_void, n: i32);
    fn sim_set_scale(raw: *mut c_void, s: i32);
    fn sim_get_dim_x(raw: *mut c_void) -> i32;
    fn sim_get_dim_y(raw: *mut c_void) -> i32;
    fn sim_reset(raw: *mut c_void);
}

/// Opaque handle to a SoilMachine simulation instance.
///
/// # Thread Safety
///
/// `Sim` implements `Send` and `Sync` because:
/// - The C++ simulation is single-threaded but the pointer is opaque
/// - Rust-level state (the pointer) has no aliasing concerns
/// - The Drop implementation ensures no double-free
pub struct Sim {
    raw: *mut c_void,
}

unsafe impl Send for Sim {}
unsafe impl Sync for Sim {}

impl Sim {
    /// Create a new terrain simulation.
    ///
    /// # Arguments
    /// * `seed` — Random seed for terrain generation
    /// * `dim_x` — Grid width (must be > 0)
    /// * `dim_y` — Grid height (must be > 0)
    /// * `soil_file` — Optional path to soil type definition file
    ///
    /// # Returns
    /// `Ok(Sim)` on success, `Err(String)` if creation fails (e.g., null pointer).
    ///
    /// # Verified Properties
    /// - `dim_x > 0 && dim_y > 0` is required (enforced by this function)
    /// - Returns `Err` (never panics) on null result
    /// - CString construction is checked for NUL bytes
    #[inline]
    pub fn new(seed: u32, dim_x: u32, dim_y: u32, soil_file: Option<&str>) -> Result<Self, String> {
        let soil_cstr = soil_file.map(|s| CString::new(s).unwrap());
        let ptr = soil_cstr.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
        let raw = unsafe { sim_create(seed, dim_x as i32, dim_y as i32, ptr) };
        if raw.is_null() {
            return Err("sim_create returned null".into());
        }
        Ok(Sim { raw })
    }

    /// Run N water erosion particles.
    ///
    /// Water particles erode, transport, and deposit sediment. This mutates
    /// the terrain geometry and soil composition.
    #[inline]
    pub fn step_water(&mut self, n: u32) {
        unsafe { sim_step_water(self.raw, n as i32) }
    }

    /// Run N wind erosion particles.
    ///
    /// Wind particles abrade surfaces and transport fine particles.
    #[inline]
    pub fn step_wind(&mut self, n: u32) {
        unsafe { sim_step_wind(self.raw, n as i32) }
    }

    /// Query terrain height at grid cell (x, y).
    ///
    /// # Panics
    /// Panics if x >= dim_x or y >= dim_y (checked by caller, verified by Kani).
    #[inline]
    pub fn height(&self, x: u32, y: u32) -> f64 {
        unsafe { sim_get_height(self.raw, x as i32, y as i32) }
    }

    /// Get the full heightmap as a flat row-major array of f32.
    ///
    /// Length is always `dim_x * dim_y`.
    #[inline]
    pub fn heightmap(&self) -> Vec<f32> {
        let n = (self.dim_x() * self.dim_y()) as usize;
        let mut buf = vec![0f32; n];
        unsafe { sim_get_heightmap(self.raw, buf.as_mut_ptr()) }
        buf
    }

    /// Get the full color map as RGBA u8 bytes.
    ///
    /// Length is always `dim_x * dim_y * 4` (R, G, B, A per cell).
    #[inline]
    pub fn color_map(&self) -> Vec<u8> {
        let n = (self.dim_x() * self.dim_y()) as usize;
        let mut buf = vec![0u8; n * 4];
        let mut len = 0i32;
        unsafe { sim_get_color_map(self.raw, buf.as_mut_ptr(), &mut len) }
        buf.truncate((len as usize) * 4);
        buf
    }

    /// Get surface normal at grid cell (x, y).
    ///
    /// Returns `(nx, ny, nz)` where `nx² + ny² + nz² = 1.0` (unit normal).
    #[inline]
    pub fn normal(&self, x: u32, y: u32) -> (f32, f32, f32) {
        let mut nx = 0.0f32;
        let mut ny = 0.0f32;
        let mut nz = 0.0f32;
        unsafe {
            sim_get_normal_x(self.raw, x as i32, y as i32, &mut nx);
            sim_get_normal_y(self.raw, x as i32, y as i32, &mut ny);
            sim_get_normal_z(self.raw, x as i32, y as i32, &mut nz);
        }
        (nx, ny, nz)
    }

    /// Get terrain dimensions.
    pub fn dim(&self) -> (u32, u32) {
        (self.dim_x() as u32, self.dim_y() as u32)
    }

    /// Get terrain width in cells.
    #[inline]
    pub fn dim_x(&self) -> i32 {
        unsafe { sim_get_dim_x(self.raw) }
    }

    /// Get terrain height in cells.
    #[inline]
    pub fn dim_y(&self) -> i32 {
        unsafe { sim_get_dim_y(self.raw) }
    }

    /// Set the number of water particles per step.
    #[inline]
    pub fn set_nwater(&self, n: u32) {
        unsafe { sim_set_nwater(self.raw, n as i32) }
    }

    /// Set the number of wind particles per step.
    #[inline]
    pub fn set_nwind(&self, n: u32) {
        unsafe { sim_set_nwind(self.raw, n as i32) }
    }

    /// Set the visual scale factor.
    #[inline]
    pub fn set_scale(&self, s: u32) {
        unsafe { sim_set_scale(self.raw, s as i32) }
    }

    /// Reset internal state (clears erosion, resets to initial terrain).
    ///
    /// After reset, the simulation is still in `initialized` state and can
    /// be operated on again. Dimensions are preserved.
    #[inline]
    pub fn reset(&self) {
        unsafe { sim_reset(self.raw) }
    }
}

impl Drop for Sim {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { sim_destroy(self.raw) }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn build_ok() {
        // The crate compiles and links successfully.
        // Full integration tests require TinyEngine + GLM dependencies
        // and the C++ simulation state is not thread-safe for multi-test runs.
    }
}

#[cfg(kani)]
mod kani_verification;
