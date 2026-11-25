# Formal Verification for SoilMachine

This directory contains formal verification artifacts for the SoilMachine Rust bindings crate.

## Tools

| Tool | Purpose | Run |
|------|---------|-----|
| **Kani** | Model checking (memory safety, bounds, invariants) | `make kani` |
| **Proptest** | Property-based testing | `make proptest` |
| **TLC** | Model checking of erosion state machine | `make tlc` |
| **Verus** | Formal contract proofs | `make verus` |

## Kani (Model Checking)

Kani verifies memory safety and pre/post-conditions at the Rust FFI boundary.

**Files:**
- `src/kani_verification.rs` — Proof harnesses for Sim API
- `verification/kani/verify_sim.rs` — Standalone Kani proofs

**Properties verified:**
- `sim_new_no_panic`: Construction never panics for valid inputs
- `sim_dimensions_positive`: Dimensions are always > 0
- `sim_height_finite`: Heights are always finite
- `sim_normal_normalized`: Normals have unit length
- `sim_buffer_sizes_valid`: Buffer allocations fit in memory
- `sim_state_machine_invariant`: Created → operating → drop

```bash
kani src/kani_verification.rs
```

## Proptest (Property-Based Testing)

Tests simulation invariants across random seeds and dimensions.

**Files:**
- `tests/prop_sim.rs` — Property tests

**Properties tested:**
- Heights are non-negative and bounded
- Heightmap and colormap dimensions match grid size
- Normals are normalized (length ≈ 1.0)
- Dimensions are consistent across all accessors
- Erosion steps produce finite values

```bash
cargo test --test prop_sim
```

**Note:** The C++ simulation uses global mutable state that can cause a
process-exit crash after tests complete. This is a pre-existing issue in
the original C++ codebase, not in the Rust bindings.

## TLC (TLA+ Model Checking)

Models the abstract erosion state machine to verify invariants about
mass conservation, height bounds, and particle lifecycle.

**Files:**
- `verification/tlc/SoilMachine.tla` — TLA+ specification
- `verification/tlc/SoilMachine.cfg` — Model checking configuration

**Invariants checked:**
- All heights are non-negative
- Heights bounded by MaxHeight
- Water volume is non-negative
- Particle count bounded
- State machine progresses fairly

```bash
java tlc2.TLC -workers 4 verification/tlc/SoilMachine
```

## Verus (Formal Verification)

Specifies pre/post conditions and state machine contracts for Sim methods.

**Files:**
- `verification/verus/sim_contracts.rs` — Formal contracts

**Contracts specified:**
- `SimSpec::pre_new`: dim_x > 0 && dim_y > 0
- `SimSpec::post_new`: creates valid initialized state
- `SimSpec::post_step_water`: step count increases
- `SimSpec::post_height`: returns finite value
- `heightmap_invariant`: all values non-negative and bounded
- `colormaps_invariant`: RGBA format correct
- `normal_invariant`: unit length

```bash
/opt/verus/current/verify_rust/src/verus verification/verus/sim_contracts.rs
```

## Makefile Targets

```bash
make build          # Build the crate
make test           # Run unit tests
make proptest       # Run property tests
make kani           # Run Kani model checking
make tlc            # Run TLC model checking
make verus          # Run Verus verification
make verify         # Run all verification
make full-check     # Build + test + verify
```
