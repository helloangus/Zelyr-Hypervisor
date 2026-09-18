# P0-W04 Build Profile / Feature Governance — Implementation Record

**Status:** Implemented on branch `p0/w04-build-profile-governance`;
verification evidence in [the verification
record](../verification/p0-w04-build-profile-feature-governance-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W04 detailed implementation
design](p0-w04-build-profile-feature-governance/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/build-profile-governance.md` (new) | Normative build-choice governance v0.1: three switch classes with precedence rule, classification procedure, Reserved profile registry (six names, non-fork rule), review questions, five prohibited cases, three change thresholds, boundary pointers; ADR-047/037/046 cited as semantic authority |
| `docs/README.md` | One routing-table row for switch-classification work |
| `docs/stages/p0/implementation/README.md` | W04 status row updated truthfully |
| This record; the verification record | Decisions and drill/review evidence |

## Deviations from the design

None. No Cargo manifest, feature, `[profile.*]` section, dependency, Rust
source, `unsafe`, or CI workflow was added or modified; no existing switch is
named by the document (none exists); all examples are marked informative.

## Handoff notes for downstream packages

- **W03:** its feature-free, custom-profile-free baseline is compliant by
  construction; any future feature or profile on any member classifies here
  first.
- **W07:** gate matrices use the class vocabulary (§2); a gate that builds a
  specific capability/profile selection names the class it exercises.
- **W16:** the profile identifier is a metadata dimension with the vocabulary
  owned in §4.1; W16 owns how it is recorded and versioned.
- **W12:** release visibility and trimming decisions reference §4 profile
  semantics, not ad-hoc cfg names.
- **P1+:** every feature-bearing design carries its §3 classification; a
  runtime quantity that cannot pass Q1 is a design defect to fix in the
  design, not a feature to add.
