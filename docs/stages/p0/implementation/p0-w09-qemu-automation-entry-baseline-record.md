# P0-W09 QEMU Automation Entry Baseline — Implementation Record

**Status:** Implemented on branch `p0/w09-qemu-runner-entry`; verification
evidence in [the verification
record](../verification/p0-w09-qemu-automation-entry-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W09 detailed implementation
design](p0-w09-qemu-automation-entry-baseline/README.md)

## Prerequisite-surface findings (workflow step 1)

| Surface | State at implementation |
|---|---|
| W02 toolchain pin | Delivered: root `rust-toolchain.toml` (pin `1.98.1`) + normative contract; recorded as the pinned-toolchain constraint for future implementers. |
| W03 build surface | Delivered: workspace with the bare-metal AArch64 member and its build entry; referenced as the future boot subject. |
| W05 conventions | Delivered: documentation baseline (class inventory, status headers) applied to the new contract. |
| W17 naming baseline | Not yet delivered; contract §6 carries the placeholder naming rule with the recorded supersession path. |

Existing tracked QEMU mentions: all are references in ADR/task-book/guide/
design/record documents (informational or requirement statements). None is
an automation entry, script, or CI embedding; no competing entry exists.
`scripts/` and `tests/` contain only tracked `.gitkeep` markers.

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/testing/qemu-runner-entry.md` (new) | Runner entry contract v0.1: single-entry rule, responsibility boundary, grammar sketch, seven reserved parameter classes, runtime behavior requirements, six-class exit taxonomy, evidence content set, consumability constraints, P0 placeholder marking |
| `docs/README.md` | One routing row for QEMU automation/runner work (marks the placeholder status) |
| `docs/testing/README.md` | One pointer line (no policy restated) |
| `docs/stages/p0/implementation/README.md` | W09 status row updated truthfully |
| This record; the verification record | Findings and documentary evidence |

## Deviations from the design

None. No script, program, CI workflow, crate, target, QEMU flag recipe,
image, or timeout default was committed. No QEMU execution occurred; none
was authorized.

## Handoff notes for downstream packages

- **W19:** the placeholder boundary and prerequisites are quotable from
  contract §7/§8.
- **W20:** the future/non-P0 classification is stated in §7/§8; absence of a
  QEMU check in CI is the designed state.
- **P1-W10:** first implementation, subject to this contract; semantic
  renegotiation is a version bump.
- **W17:** naming baseline supersedes §6's placeholder rule (compatible:
  without a bump; incompatible: with one).
- **W07:** promotion into the gate register is a future-class promotion per
  the quality-gates thresholds.
