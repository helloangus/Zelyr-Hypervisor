# P0-W16 Version/Build Metadata Baseline — Implementation Record

**Status:** Implemented on branch `p0/w16-version-metadata`; verification
evidence in [the verification
record](../verification/p0-w16-version-build-metadata-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W16 detailed implementation
design](p0-w16-version-build-metadata-baseline/README.md)

## Declared values and prerequisite status

- **`project_version` = `0.1.0`**, declared in contract §3.3 as the single
  authoritative project declaration.
- Prerequisite surfaces: W03 delivered (target boundary names
  `target_architecture`; the platform vocabulary is **not yet named**, so
  `platform` remains required-but-unfilled by the schema's own rule); W04
  delivered as governance (profiles are Reserved, none in-tree —
  `build_profile` is required-once-in-tree per the schema rule and
  `capability_summary` stays reserved).

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/version-build-metadata.md` (new) | Normative identity baseline v0.1: seven identity questions with sources, eleven-field schema with required/reserved status, determinism-first timestamp policy, dirty-tree policy, single project-version declaration, ADR-040 compatibility reservations, W12/W17 linkage rules, mutation rules |
| `docs/README.md` | One routing row for artifact-identity questions |
| `docs/stages/p0/implementation/README.md` | W16 status row updated truthfully |
| This record; the verification record | Decisions, cross-review evidence |

## Cross-review obligations

- **W12 (§5.1, mutual):** W12's association property matches this baseline's
  minimum diagnostic identity set (project_version, source_revision, dirty,
  target_architecture, build_profile); no divergence. Cross-review recorded
  in this package's verification record; W12's record carries the symmetric
  entry ("pending the W16 delivery", now satisfied).
- **W17 (§5.2):** obligation armed — W17's mapping table must trace every
  name field to a §2 schema field; W16's cross-review runs at W17 closure.

## Deviations from the design

- The design's §4.3 anticipated the declaration migrating into the workspace
  manifest once W03 delivered one. W03 delivered a *virtual* workspace whose
  member manifests carry placeholder package versions (its handoff note
  explicitly defers identity semantics to W16/W17) and its root-manifest
  comment reserves `[workspace.package]` for when members need shared keys.
  Introducing that shared key now would be a manifest redesign outside
  W16's §8 exclusions. Resolution: the single declaration lives in this
  contract document; member-manifest versions are explicitly non-authoritative
  package-local values; the migration rule is preserved for the future
  shared-key introduction. No manifest was edited.

## Handoff notes for downstream packages

- **W17:** encodes only a declared subset of §2 fields; the mapping table
  must trace to schema fields (§5.2 cross-review).
- **W19:** the dirty-tree policy's clean-checkout rule binds the documented
  workflow's evidence instructions.
- **W20:** CI builds are from clean checkouts (§3.2); identity fields must
  be associable for CI-produced artifacts.
- **P1 crash/logging designs:** the fatal channel carries the minimum
  diagnostic identity set inline (per W12 §5 + this baseline §5.1).
