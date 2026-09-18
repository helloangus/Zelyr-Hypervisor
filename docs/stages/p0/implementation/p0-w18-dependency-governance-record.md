# P0-W18 Dependency Governance — Implementation Record

**Status:** Implemented on branch `p0/w18-dependency-governance`; verification
evidence in [the verification
record](../verification/p0-w18-dependency-governance-verification.md).
**Date:** 2026-09-19 (Asia/Shanghai)
**Design:** [W18 detailed implementation
design](p0-w18-dependency-governance/README.md)

## Prerequisite status observed

- W10 delivered: the unsafe boundary categories
  (`arch-register`, `mmio-volatile`, `memory-mgmt`, `low-level-struct`,
  `asm-glue`, `boot-state`) exist; the policy's checklist dimension 5
  references them by name — the cross-review of §5.2 against the delivered
  unsafe policy resolved with no divergence (third-party unsafe is weighted,
  never adopted into the first-party inventory).
- W03 delivered: a virtual workspace with **zero dependencies named in any
  manifest**; `Cargo.lock` exists. The register↔manifest consistency rule is
  therefore checkable from adoption and holds (0 = 0).

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/dependency-governance.md` (new) | Normative dependency policy v0.1: fail-closed purpose statement, three tiers (D1/D2/D3) with linkage-based classification rules, eleven-dimension evaluation checklist, introduction/upgrade/security/deprecation/exception lifecycle with ADR thresholds, register↔manifest + unsafe-interface + lockfile consistency rules, mutation rules |
| `docs/development/dependency-register.md` (new) | Empty decision ledger v0.1: zero entries, no illustrative content, maintenance rules, adoption-state consistency check recorded |
| `docs/README.md` | One routing row for dependency additions/changes |
| `docs/stages/p0/implementation/README.md` | W18 status row updated truthfully |
| This record; the verification record | Decisions and rehearsal evidence |

## Deviations from the design

None. No crate is approved, rejected, or recommended anywhere; the register
carries zero entries; no manifest, wrapper API, vendoring mechanism, or CI
workflow was added.

## Handoff notes for downstream packages

- **P1+ designers:** the introduction path (§4.1) is the only route to a
  dependency; D3 requires maintainer approval recorded in the register.
- **Build-baseline consumers:** manifest edits that add a dependency require
  the register entry in the same change (register → manifest authority).
- **W07/W20:** the register↔manifest consistency check is a named future
  gate candidate; the predicate is implementable from §5.1.
- **W10:** interlock confirmed — first-party unsafe mechanics remain the
  unsafe policy's; third-party unsafe is checklist-weighted only.
