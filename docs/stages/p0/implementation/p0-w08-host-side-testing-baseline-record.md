# P0-W08 Host-Side Testing Baseline — Implementation Record

**Status:** Implemented on branch `p0/w08-host-testing-baseline`; verification
evidence in [the verification
record](../verification/p0-w08-host-side-testing-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W08 detailed implementation
design](p0-w08-host-side-testing-baseline/README.md)

## Prerequisite state observed

- Baseline: merge of PR #17 (W06). W02 toolchain contract delivered
  (`rust-toolchain.toml`, pin `1.98.1`); W03 workspace delivered (virtual
  workspace, single bare-metal member `hypervisor`); W05 metadata rules
  delivered and applied to the new contract.

## Canonical entry spelling (recorded per contract §5)

```sh
cargo test --workspace --exclude hypervisor
```

Selected 2026-09-18 under pinned toolchain `1.98.1`. The
`--exclude hypervisor` term is semantically required: the hypervisor member
is a bare-metal-class freestanding binary that cannot compile for a host
target (verified: whole-workspace `cargo test` fails on it with duplicate
`panic_impl`), so the exclusion names exactly the non-host class; future
host-class members are included automatically.

## Placement decision

The placeholder test is the first host-class workspace member,
`crates/host-test-baseline` (edition 2024, zero dependencies/features),
whose single test asserts only entry health. Authority: the delivered
build-target policy assigns the host-native class's build path to "P0-W08
for test members" and records "no member exists until W08" — W08 adding the
first host-class member is the designed delivery path, not a new target or
workspace mechanism (no triple is pinned; the host target is machine-derived
per policy). The `crates/.gitkeep` marker is replaced by the member,
mirroring W03's marker-to-source pattern. The alternative reading (treat the
member as a W03-boundary conflict and record a blocker) was considered and
rejected: it would leave P0-V03/V04 permanently vacuous, contradicting the
delivered W03 policy's own assignment.

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/testing/host-test-baseline.md` (new) | Normative host-test contract v0.1: logic boundary, organization rules, entry semantics + recorded spelling, coverage-category matrix + mapping rule, proof boundary, thresholds, placeholder status |
| `crates/host-test-baseline/` (new; replaces `crates/.gitkeep`) | Placeholder host-test member (entry-health test only) |
| `Cargo.toml` | Workspace membership: `crates/host-test-baseline` added |
| `Cargo.lock` | Updated by the membership change (no dependencies) |
| `docs/README.md` | One routing row for host-side test work |
| `docs/testing/README.md` | One pointer line to the contract (no policy restated) |
| `docs/stages/p0/implementation/README.md` | W08 status row updated truthfully |
| This record; the verification record | Decisions and evidence |

## Deviations from the design

- The design's §7 exclusion ("if the baseline appears to require creating a
  workspace member … that is the W03 boundary") was resolved in favor of
  creating the member, on the authority of the delivered W03 policy quoted
  above; the reasoning is recorded in the placement decision. No target
  triple was chosen and no W03 mechanism was redesigned, which is what the
  exclusion guards.
- Everything else follows the design, including the failure-visibility dry
  run (performed locally, reverted, never committed — the untracked-file
  revert needed a manual edit, which is recorded in the verification log).

## Handoff notes for downstream packages

- **W07:** `cargo test --workspace --exclude hypervisor` is the sole binding
  target for the host-test gate; classification and blocking semantics are
  W07's; semantic entry changes route through W07's register.
- **W19:** the entry is the test step of the clone-to-build path; the proof
  boundary sentence (contract §7) is quotable in onboarding.
- **W20:** same spelling in CI under the pinned toolchain; no emulator
  dependency; CI execution completes P0-V03/V04.
- **Later module designs (P1+):** map host-testable units to the §5 matrix
  with per-exclusion reasons; retire the placeholder when the first real
  host-tested logic lands.
- **W09/P1:** host passing never substitutes for QEMU or hardware evidence.
