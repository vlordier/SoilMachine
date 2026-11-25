# SoilMachine Rust bindings Makefile
# Build, test, and formal verification targets

CARGO = cargo
CARGO_TARGET_DIR = ./target

# ── Build ─────────────────────────────────────────────────────────────

.PHONY: all build clean

all: build

build:
	$(CARGO) build --target-dir $(CARGO_TARGET_DIR)

clean:
	$(CARGO) clean --target-dir $(CARGO_TARGET_DIR)

# ── Tests ─────────────────────────────────────────────────────────────

.PHONY: test proptest

test:
	$(CARGO) test --target-dir $(CARGO_TARGET_DIR)

# Property-based tests (Proptest)
proptest:
	$(CARGO) test --test prop_sim --target-dir $(CARGO_TARGET_DIR)

# ── Formal Verification ───────────────────────────────────────────────

.PHONY: kani verus tlc verify

# Kani model checking
# Run: kani src/kani.rs
# Verifies: memory safety, bounds, null-deref, state machine
kani:
	kani src/kani.rs

# Verus formal verification
# Run: verus verification/verus/sim_contracts.rs
# Verifies: pre/post conditions, state machine contracts
verus:
	/opt/verus/current/verify_rust/src/verus verification/verus/sim_contracts.rs

# TLC model checking (requires Java)
# Models: erosion state machine invariants
# Run: java tlc2.TLC -workers 4 verification/tlc/SoilMachine.tla
tlc:
	cd verification/tlc && java tlc2.TLC -workers 4 SoilMachine

# Run all verification
verify: kani tlc

# ── Integration ───────────────────────────────────────────────────────

.PHONY: full-check

# Full check: build + test + verify
full-check: build test verify
	@echo "All checks passed."
