# P6-W06 Validation, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W06 design entry](README.md). This file closes the workflow in
[05-implementation-workflow.md](05-implementation-workflow.md).

## 1. Validation matrix

QEMU (virt, declared configuration) is the reference environment for
runtime rows; **QEMU generic-timer and GIC emulation succeeding does not
prove real-hardware timer behavior** — real-hardware validation belongs to
the Orange Pi stage. Rows that depend on a W07/W08/W09 mechanism are marked
as consuming that dependency; W06 rows must not be reported as proving
those mechanisms.

| ID | Requirement (plan/task book) | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W06-DV01 | Typed domains, control image, intake | host-side unit tests + intake exercise | verdict-matrix tests for `evaluate()`; conversion/overflow tests; intake with deliberately mismatched INTID/layout fixtures | evaluator total and pure; conversions checked; intake mismatch disables delivery with diagnosis | evaluation and intake logic; not hardware behavior |
| W06-DV02 | Guest programming/masking/expiry (P6-V09) | QEMU scenario via W11 asset (consumes W07/W08) | Guest reads/programs/enables CNTV, observes handled virtual-timer event; masked-expiry sub-case | event presented exactly for expired-unmasked conditions; masked expiry never presented, becomes observable after Guest unmask | the defined Guest timer path in the declared environment; not full GIC semantics (W10) or real hardware |
| W06-DV03 | Entry/exit preservation (P6-V10) | QEMU transition scenario | exercise HVC and controlled exits (P4 paths) with a live Guest timer; compare record vs registers at boundaries | timer behavior preserved across declared exits; no loss, no Host-state contamination; record authoritative after exit | preservation across the declared exit set; not all possible exit kinds |
| W06-DV04 | Deferred expiry while absent (P6-V09/V10 basis) | QEMU deferred-delivery scenario | program an expiry whose deadline falls in an exit window (expired-at-exit), and a future-deadline case; re-enter and observe | expired-at-exit event presented at next entry exactly once; future deadline delivered naturally during the next run; no duplicate delivery | the D4 deferred model; not wakeup latency bounds (P7) |
| W06-DV05 | Guest-caused error boundary | QEMU negative scenario | Guest performs EL1 physical-timer access; observe disposition | one controlled Guest fault, VM-local, counted; no Host state change; virtual-timer access untrapped | trap policy conformance; not P5 taxonomy itself |
| W06-DV06 | Per-vCPU independence and physical-PPI rules (P6-V19 basis) | QEMU scenario + invariant review | single-vCPU conversion/hold/release cycle with level re-fire case; multi-vCPU isolation **only if** the evidenced multi-vCPU prerequisite exists (otherwise record the stage block) | one outstanding physical INTID at all times; completion release re-fires only on persisting condition; per-vCPU isolation observed when exercisable | structural isolation and the D6 rules; multi-vCPU isolation evidence only with the prerequisite, else a documented block (allowed by task book) |
| W06-DV07 | Diagnostics and telemetry surface | review + counters observed in DV02–DV06 | counters/trace present and correlated; deferred/converted/coalesced/released/trapped counts match scenarios | all [02] §7 counters populated and consistent with scenario outcomes | observability basis; not latency conclusions (W13) |
| W06-DV08 | Handoff readiness and scope conformance | design-conformance review | review surface for scheduler/machine-ABI leakage; handoff checklist §3 | no P7/P8 semantics in W06's surface; consumers can act without inventing contracts | scope conformance; not downstream correctness |

Planned, run, blocked, and failed are distinct evidence states recorded in
the verification record with command, input, environment, timestamp, and
reason. No W06 validation proves P6-V11–P6-V18 (W07–W10 scope), scheduler
behavior, the Linux timer ABI, or real-hardware correctness, and none may
be reported as doing so.

## 2. Error, security, and observability model

**Error model.** Named cases and guarantees:

- Intake failures (`IntakeMismatch`, posture mismatch): delivery disabled
  with diagnosis; save/restore remains safe; no partial delivery state.
- W07 rejection (`Rejected{reason}`) on deferred injection: nothing queued,
  reason counted (a destroy race is diagnosable, not fatal).
- Invariant violations (flag set at entry, double dispatch while held):
  counted, diagnosed, recovered by releasing the held interrupt — these are
  Host-attributed anomalies, never Guest-caused (the Guest cannot set W06
  flags), so the ADR panic-policy distinction applies: Guest errors are
  VM-local faults; W06 invariant violations are surfaced as Host
  diagnostics.
- Guest-caused errors: trapped timer access -> one VM-local Guest fault;
  hostile timer values (extreme deadlines, rapid reprogramming) -> handled
  by pure evaluation and O(1) paths, never Host faults.
- Failure ordering: exit silences the hardware before releasing any held
  physical interrupt; entry recovery releases before restoring.

**Security model.** Guest-controlled values (control fields, deadlines) are
validated images of hardware registers and are used only through the typed,
total evaluator; no Guest value selects a Host target, index, or length.
The Guest cannot grow Host pending state (W07 dedupes; W06 requests one
event class), cannot reach another vCPU's timer state, and cannot convert a
Guest error into a Host panic. The `unsafe` surface is the Arch-domain
register accessors, inventoried with `SAFETY` justifications; the barrier
table ([03](03-code-contracts-vcpu-timer-state.md) §3.3) is part of the
audited boundary.

**Observability model.** The per-vCPU counters and trace events of
[02](02-architecture-and-state.md) §7 are the designed surface; P6-W13 owns
collection, correlation (P6-V24), and any latency measurement. Deferred-
delivery counts are behavior records, not performance claims.

## 3. Handoff checklist

Before handing W06 to a reviewer, provide:

- the exact changed-file list and the crate/module placement chosen for the
  logical modules;
- the W06 `unsafe` inventory delta with `SAFETY` justifications;
- W06-DV01–DV08 evidence paths and run status, including explicit not-run
  or blocked entries (multi-vCPU rows without the prerequisite; any row
  blocked by a W07/W08/W09 dependency);
- the resolved `TIMER_EVENT_INTID` with its evidence (platform intake and
  Validation Guest layout cross-check);
- the recorded decision on pause/resume (Reserved, per D8) and the deferred-
  delivery limitation statement (delivery latency bounded by next entry);
- confirmation that no scheduler wakeup, runnable-state, LR/maintenance, or
  machine-ABI/DTB semantics were implemented, and no crate dependency was
  added;
- open items for consumers, without resolving their contracts here:
  - **P6-W10** (`../p6-w10-interrupt-semantics/README.md`): timer masking
    honor, condition semantics, and deferred delivery are defined inputs to
    the masking/priority/timer-plus-vIRQ semantics;
  - **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`):
    VG-TIMER scenario expectations are defined by DV02–DV04 conditions;
  - **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`): the
    trapped-access boundary and invariant-recovery rules are the isolation
    surface to stress;
  - **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`):
    counters and trace identities are available; collection is W13's;
  - **P7/P8**: proven vCPU-owned semantics and limits only; wakeup (P7) and
    offset/machine ABI (P8) remain theirs;
  - cross-design: the D6 split-completion expectation on W03 and the
    completion hook expectation on W09 must remain reconciled with those
    packages' delivered designs; divergence is an Architecture Change
    Request, not a W06-local change.

No completion claim may be made anywhere in this design; completion
evidence belongs only in
`../../verification/p6-w06-guest-generic-timer-verification.md`.
