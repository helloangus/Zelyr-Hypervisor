# P0-W17 Artifact Naming Baseline — Implementation Record

**Status:** Implemented on branch `p0/w17-artifact-naming`; verification
evidence in [the verification
record](../verification/p0-w17-artifact-naming-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W17 detailed implementation
design](p0-w17-artifact-naming-baseline/README.md)

## Prerequisite status assumptions

- W16 delivered: identity schema is the sole field-meaning authority; the
  §5 mapping table traces every name field to a schema field (W16's §5.2
  cross-review obligation satisfied — recorded in both verification records).
- W04/W03: profile vocabulary intentionally empty (fail-closed); platform
  vocabulary carries the two ADR-named designators; `host` deliberately
  not pre-declared.
- QEMU runner entry (W09): its §6 placeholder naming rule is superseded by
  this contract, compatibly (deterministic, unique, machine-processable);
  noted in this document's Supersedes header and in the runner contract's
  own text (which already anticipated the supersession).

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/artifact-naming.md` (new) | Normative naming baseline v0.1: governed scope, seven dimensions + `+dirty`, grammar and character-set parseability rules, five vocabularies with governance (profile empty fail-closed; `host` not pre-declared), stability/determinism rules, W16 identity mapping table, eleven-category inventory (one P0-required, ten Reserved), evolution rules, format non-commitment |
| `docs/README.md` | One routing row for artifact naming |
| `docs/stages/p0/implementation/README.md` | W17 status row updated truthfully |
| This record; the verification record | Decisions and evidence |

## Deviations from the design

None. No format, generation process, output layout, code, or CI was
prescribed; no governed artifact name instance exists yet (the first
`hypervisor`-class instance is the owning build design's deliverable and is
expected not-run at W17 closure).

## Handoff notes for downstream packages

- **Producing build designs (first `hypervisor`-class instance):** declare
  the format/extension; names follow §3 with the §6 applicability row.
- **W19/W20:** evidence artifacts leaving the build tree carry governed
  names; `+dirty` asserts clean per the W16 policy.
- **W09 consumers (P1 runner):** evidence naming now follows this contract
  (supersedes the placeholder rule compatibly).
- **ADR-054 owners:** the name-token migration path is §4.1.
