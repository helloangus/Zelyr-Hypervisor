# P0-W05 Documentation Baseline — Implementation Record

**Status:** Implemented on branch `p0/w05-documentation-baseline`;
verification evidence in [the verification
record](../verification/p0-w05-documentation-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W05 detailed implementation
design](p0-w05-documentation-baseline/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/documentation-baseline.md` (new) | Normative documentation baseline v0.1: class inventory, normative/informative discipline, metadata rules, change thresholds, stage-document separation, entry/link/referencability rules |
| `docs/README.md` | Header metadata fields added; one routing-table row for documentation work |
| `docs/development/README.md` (new) | Sweep: missing directory README created (role + status class) |
| `docs/stages/README.md` (new) | Sweep: missing directory README created (role + status class) |
| `docs/development/toolchain-baseline.md` | Sweep: broken manifest link fixed (`../../../` → `../../`) |
| `docs/adr/README.md`, `docs/abi/README.md`, `docs/architecture/README.md`, `docs/machine-types/README.md`, `docs/platform/README.md`, `docs/security/README.md`, `docs/templates/README.md`, `docs/testing/README.md` | Sweep: status-class line added per the inventory |
| `docs/development/coding-guidelines.md`, `docs/development/plan-agent-guidelines.md`, `docs/development/integration-workflow.md` | Sweep: missing metadata fields (Version/Owner-change context/Supersedes) added |
| `docs/stages/p0/implementation/README.md` | W05 status row updated truthfully |
| This record; the verification record | Findings, decisions, and evidence |

## Coherence-sweep findings and edits

| # | Finding | Edit | Baseline rule |
|---|---|---|---|
| 1 | `docs/development/` had no README | Created with role + status class | §6 entry rule |
| 2 | `docs/stages/` had no README | Created with role + status class | §6 entry rule |
| 3 | Directory READMEs lacked an explicit status class (adr, abi, architecture, machine-types, platform, security, templates, testing) | One status-class line each | §1 inventory, §6 |
| 4 | `docs/README.md`, the concise guides, the integration workflow, and `docs/adr/README.md` lacked some required metadata fields | Minimal field additions | §3 metadata rules |
| 5 | Broken relative link to `rust-toolchain.toml` in the toolchain baseline | Path depth corrected | §6 link rule |
| 6 | Five broken links inside P8 **proposed designs**, each pointing at a future P8 verification record or record file that exists only after P8 implements | **No edit.** Not entry documents; forward references within proposed designs. Recorded as an observation for the P8 implementing packages. | §6 scope (entry documents) |
| 7 | All other entry documents conforming | No edit needed | — |

No style rewrite occurred; every edit maps to a specific baseline rule. No
re-homing or re-versioning of an accepted contract was executed; no Reserved
follow-up requiring owner consent arose.

## Deviations from the design

None. No ADR lifecycle content, workflow admission rules, template, code,
build artifact, or CI configuration was added.

## Handoff notes for downstream packages

- **W06:** the ADR class slot and header conventions are in place; the ADR
  lifecycle document lands in `docs/adr/` under the normative class.
- **W21:** stage sublocations and their allowed contents are fixed (§5); the
  responsibility flow between layers is W21's to define.
- **W07/W20:** the coherence and metadata rules are written to be checkable;
  mechanical enforcement wiring is theirs.
- **P1+:** the standing rules for new documents are §1–§6; a P1 planner finds
  every output's home in the §1 inventory before starting.
