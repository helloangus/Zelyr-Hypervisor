# P3-W13 Result Classification and Evidence

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W13 detailed design](README.md).

## 1. Principles

- **A result is what was captured, not what was expected.** Exit codes
  and green markers are inputs to classification, never the
  classification itself.
- **Environment is part of the result.** Two runs in different
  environment classes are not comparable; the environment block is
  mandatory in every entry.
- **The claim is the matrix's, exactly.** P3-V13 wording maps to
  "declared criteria met over repeated cold boots at 1/2/4/8 in the
  declared environment" — and to nothing stronger.

## 2. Execution path and the blocked state

Matrix rows are expressed as consumption rules against the P0-W09
automation entry (assumed contract): the entry's declared inputs must
express (a) the smp count, (b) the image/build under test, (c) output and
diagnostics capture, (d) a determinate result. Until that contract exists
and is implemented, every QEMU row is recorded **blocked** with the
blocking contract cited — W13 implements no runner, script, or CI file
(P0-W20 owns CI wiring; its decision to gate on matrix rows is the
Reserved trigger recorded in the entry README).

If the delivered automation contract cannot express (a)–(d) for some row,
that row is blocked on P0-W09 with the gap named; no local workaround
script is authored.

## 3. Result classification

Per row, per count, per campaign — mutually exclusive:

- **run-passed** — all cited assertions held on every repetition in the
  campaign; capture complete; within declared bounds and repetition
  depth.
- **run-failed** — at least one assertion broke or a bound was hit; the
  entry carries the divergent capture and divergence analysis; later
  dependent rows record their dependency status explicitly.
- **blocked** — no run possible: automation entry missing, mechanism
  missing, scenario blocked per W12, or environment unavailable; the
  blocker is cited.
- **not-run** — deliberately deferred or vacuous (e.g. R6 at c = 1);
  reason recorded.

A campaign row's status is the worst of its boots/sessions per the
repetition policy: any divergent boot makes the row `run-failed`;
"passed on the last boot" is not a status.

## 4. Diagnostic capture set (mandatory per session)

1. Full boot log through `SmpReady` (and to the hang point, for a
   bounded hang).
2. W05 phase/rendezvous events and `SmpReadyState`.
3. W03 registry state dump (post-rendezvous and final).
4. W11 SMP-ready counter dump (start/end of session).
5. Per-scenario captures per W12's evidence template.
6. Environment block: date/time, host OS and load note, QEMU version,
   accelerator mode (TCG/KVM), smp count, image/build identity (P0-W16
   version metadata once it exists), and the exact automation-entry
   invocation.

Captures land under `docs/stages/p3/verification/` referenced from
W13's verification record, named by the row/count/boot convention fixed
in the implementation record. A `run-passed` without its capture set is a
review failure.

## 5. The hardware gap (standing statement)

Every row's does-not-prove clause is grounded in this stage-level
statement, handed to W14/P4 and inherited by P15:

- The matrix proves repeatability of the declared criteria on QEMU
  `virt` at the declared counts, in the declared environment class.
- It does not prove: real-hardware bring-up (RK3566/Orange Pi 3B boot
  firmware, PSCI conduit, cache/TLB/IRQ timing), real IPI and memory
  ordering behavior under hardware reordering, thermal/power or
  device-level failure behavior, performance of any kind, or guest/
  Stage-2 semantics (P4+).
- QEMU-observed facts (e.g. deterministic absent-CPU PSCI rejection) are
  recorded as reference-platform observations; no hypervisor Core
  semantic may be derived from them (ADR-003/ADR-044; task book exit
  criterion 4).

P15 owns converting this gap into real-hardware evidence; nothing in any
P3 verification record may close it.

## 6. Comparability and drift rules

- Runs compare within one environment class (same accelerator mode and
  QEMU version family); a class change starts a new campaign and both
  campaigns are kept in the record.
- A mechanism change invalidates prior campaigns for the affected rows —
  the record marks superseded campaigns rather than deleting them.
- The repetition depth and scenario depths are constants; changing them
  is a recorded decision with rationale (never stretched mid-campaign to
  make a failing row pass).
