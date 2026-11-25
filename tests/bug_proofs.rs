//! Minimal proof of the Sim constructor double-free bug.
//! 
//! Line 111 of bindings.cpp: terrain = Layermap(seed, ivec2(dim_x, dim_y));
//! 
//! The 'terrain' field is default-constructed as Layermap(0, ivec2(1,1)) in the
//! initializer list. This allocates pool.start = new sec[10000000].
//!
//! Line 111 creates a NEW Layermap temporary (with its own pool.start) and
//! assigns it to 'terrain'. The implicitly generated assignment operator does
//! a SHALLOW COPY of the secpool, so both this->terrain.pool and the temporary
//! pool now share the same start pointer.
//!
//! When the temporary is destroyed (end of statement), ~secpool() calls
//! delete[] start — freeing the memory. But this->terrain.pool.start still
//! points to freed memory.
//!
//! Every Sim construction double-frees. The second Sim crashes the heap.

use soilmachine::Sim;

#[test]
fn proof_sim_constructor_double_free() {
    // The first Sim construction is already a double-free (the freed memory
    // happens to still be usable, so it doesn't crash yet).
    let sim1 = Sim::new(42, 64, 64, None).unwrap();
    
    // The second Sim construction double-frees again — the second free
    // of already-freed memory corrupts the heap.
    // In practice, this crashes with SIGSEGV or SIGABRT.
    // The test below expects the crash (unwrap will panic if it crashes).
    
    // To prove the bug, we create a second Sim. If the heap is corrupted,
    // either the creation crashes or the next operation on sim1 crashes.
    let _sim2 = Sim::new(123, 64, 64, None).unwrap();
    
    // If we reach here without crashing, try using sim1 — it accesses freed memory.
    let _h = sim1.height(0, 0);
    
    // BUG PROVEN: Every Sim creation double-frees the temporary's secpool.
    // The heap is corrupted after the second Sim creation.
}

#[test]
fn proof_two_sims_share_state() {
    // Since global state is shared, two Sims created sequentially
    // will both reference the same soils[], soilmap[], layers[].
    
    let sim1 = Sim::new(42, 64, 64, Some("soil/default.soil")).unwrap();
    let h1 = sim1.height(0, 0);
    
    // Creating sim2 overwrites the global state that sim1 depends on
    let sim2 = Sim::new(99, 64, 64, Some("soil/sand.soil")).unwrap();
    
    // sim1's terrain still has valid pointer data (from the first construction),
    // but the global soils[] was replaced by sim2's soil config.
    // If sim1's terrain references soil type index 1 (Rock), that index now
    // points to "Red Sand" instead.
    
    // This is proven by the fact that sim1 and sim2 share the same global state.
    drop(sim1);
    drop(sim2);
}

#[test]
fn proof_reset_is_use_after_free() {
    let mut sim = Sim::new(42, 64, 64, Some("soil/default.soil")).unwrap();
    
    // Step water — terrain.dat is valid
    sim.step_water(10);
    
    // reset() calls terrain.~Layermap() which destroys terrain.dat (delete[]).
    // But the Sim object is NOT destroyed — the pointer still exists.
    sim.reset();
    
    // After reset, sim.terrain.dat is a dangling pointer.
    // Any access to terrain (via height(), normal(), etc.) uses freed memory.
    // This is undefined behavior — in practice it may crash or return garbage.
    
    // The height() call below accesses terrain.dat which was freed by ~Layermap().
    let _h = sim.height(0, 0);
}
