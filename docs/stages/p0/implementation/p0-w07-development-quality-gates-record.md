# P0-W07 Development Quality Gates — Implementation Record

**Status:** Implemented on branch `p0/w07-quality-gates`; verification
evidence in [the verification
record](../verification/p0-w07-development-quality-gates-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W07 detailed implementation
design](p0-w07-development-quality-gates/README.md)

## Prerequisite surfaces observed

- W02 delivered: pinned toolchain `1.98.1` with `rustfmt` + `clippy`
  components.
- W03 delivered: workspace + bare-metal member; target-build entry;
  zero-warning build posture under toolchain defaults (no lint table).
- W08 delivered: host-test execution entry
  (`cargo test --workspace --exclude hypervisor`).

## Recorded spellings (contract §1, selected 2026-09-18 under `1.98.1`)

- `QG-FMT`: `cargo fmt --all -- --check`
- `QG-LINT`: `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings`
  plus `cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor -- -D warnings`
  (host-class members on the host target; the bare-metal member under its own
  target — two invocations because the class separation makes one
  whole-workspace spelling impossible)
- `QG-TEST-HOST`: bound to the W08 entry, quoted verbatim
- `QG-BUILD-TARGET`: bound to the W03 entry, quoted verbatim
- `QG-WARN`: enforcement point = toolchain defaults (delivered posture);
  `-D warnings` in QG-LINT enforces at lint level
- `QG-DOCS`: semantics per contract §7 (link integrity, ≤4-hop reachability,
  status-header presence); CI realization is W20's

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/quality-gates.md` (new) | Normative quality-gates contract v0.1: six-gate register, per-gate standards, verification sets, failure-handling principles, mutation thresholds, future-class members |
| `hypervisor/src/main.rs` | `cargo fmt` applied (gate finding fixed by reformatting, per QG-FMT failure semantics) |
| `crates/host-test-baseline/src/lib.rs` | Placeholder assertion rewritten from `assert!(true)` (a clippy `assertions_on_constants` finding under `-D warnings`) to a deterministic package-context assertion; behavior and scope unchanged |
| `docs/README.md` | One routing row for quality-gate questions |
| `docs/stages/p0/implementation/README.md` | W07 status row updated truthfully |
| This record; the verification record | Decisions and dry-run evidence |

## Recorded minor changes and fixes during implementation

- Two gate dry-run findings were fixed in source, never by weakening a gate:
  the formatting finding in the W03 probe (reformatted), and the
  constant-assertion lint finding in the W08 placeholder (rewritten). Both
  subjects re-verified passing after the fix.

## Deviations from the design

None. No script, CI workflow, lint table, or executable artifact was added;
QG-DOCS's mechanical realization remains W20's (the dry run used ad-hoc local
checks recorded in the verification file).

## Handoff notes for downstream packages

- **W20:** each Required row maps to one required GitHub check named by its
  evidence label; workflow realization is bounded by contract §8's rules.
- **W09/P1:** QEMU-class checks are future-class; promotion requires an
  owning design and a full register row.
- **W10:** the unsafe-inventory consistency gate activates with the first
  `unsafe` in the tree.
- **W18:** dependency-audit reporting is an informational candidate once
  dependencies exist.
