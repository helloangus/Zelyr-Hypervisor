# P6-W05 EL2 Generic Timer — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Scope:** The per-pCPU EL2 monotonic-time and deadline-event mechanism required
by [P6-W05](../../plans/p6-w05-el2-generic-timer.md).
**Owner/change context:** P6-W05 implementation handoff.
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W05. It converts the bounded
work-package plan into a designed, code-bearing mechanism: typed monotonic-time
values, a per-pCPU EL2 deadline-event timer, its integration with the P6-W03
physical-IRQ lifecycle, and the diagnostics and evidence P6-W06, P6-W10,
P6-W11, P6-W13, and P7 consume. It deliberately does **not** design Guest timer
virtualization (P6-W06), scheduler ticks or preemption (P7), wall-clock or
offset behavior, or any concrete crate/file layout, Cargo target, or CI wiring;
those belong to their owning packages.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), completes the
Coding-Guidelines preflight (repository `AGENTS.md`, documentation index, ADR
baseline, [P6 task book](../../task-book-v0.1.md), and the
[P6-W05 plan](../../plans/p6-w05-el2-generic-timer.md)), and then loads only
the linked supporting file needed for its assigned step:

| Assigned material | Supporting file |
|---|---|
| Baseline findings, scope classification, resolved decisions | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Logical modules, ownership, lifecycle, concurrency model | [02-architecture-and-state.md](02-architecture-and-state.md) |
| Time-value and clock-read contracts (types, arithmetic, barriers) | [03-code-contracts-time-core.md](03-code-contracts-time-core.md) |
| Deadline-timer contracts (arm/cancel/rearm, IRQ receipt, register sequences) | [04-code-contracts-deadline-timer.md](04-code-contracts-deadline-timer.md) |
| Ordered implementation steps | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Validation matrix, error/security/observability model, handoff checklist | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Nothing in this design is an implementation or completion claim. Evidence is
recorded only in
`../../verification/p6-w05-el2-generic-timer-verification.md` (created when
evidence exists), and factual implementation traceability only in
`../p6-w05-el2-generic-timer-record.md` (created when work starts).

## Authority, constraints, and scope classification

The governing order is Architecture ADR → P6 task book → P6-W05 plan → this
design → Coding Guidelines. The ADR constraints that bind this design most
directly are: mechanism/policy separation (ADR-004, ADR-005), typed identity
and checked arithmetic (section 13; P0-W15 conventions), structured telemetry
(ADR-048), SMP-from-day-one with no single-pCPU assumption (ADR-015), and
Core/Arch layering with no board-name branches (ADR-041–ADR-045). The
[task book](../../task-book-v0.1.md) classifies timer programming, trace
encoding, and register sequencing as Implementation Choice resolved in approved
detailed design; the full derivation, ledger, and decision list are in
[01-scope-and-foundations.md](01-scope-and-foundations.md).

Classification summary:

- **Required:** the typed EL2 time-value and monotonic-clock surface; the
  per-pCPU deadline-event timer (arm, cancel, rearm, expiry receipt); one-shot
  and repeated-event behavior with attributable per-pCPU events; entry/exit
  monotonicity checks; per-pCPU diagnostics and telemetry hooks; the W03
  integration boundary for the EL2 timer PPI.
- **Reserved** (must not block a future design, not implemented in W05):
  cross-pCPU timer control and timer-driven wakeups (P7); scheduler tick
  policy and time-slice semantics (P7); the EL2 virtual timer (CNTHV) surface
  pending an E2H-based baseline; pause/resume of Host time; migration time
  conversion.
- **Out of Scope:** Guest timer virtualization and vCPU timer state
  (P6-W06); masking/priority semantics (P6-W10); Validation Guest scenarios
  (P6-W11); storm/robustness evidence (P6-W12); telemetry collection and the
  latency baseline report (P6-W13); wall clock, NTP, and advanced scaling; any
  crate, module path, or public API beyond the contracts stated here.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Timer capability intake and frequency/time conversion boundary | [foundations](01-scope-and-foundations.md) §3 (ledger rows 1–2); [time-core contracts](03-code-contracts-time-core.md) | W05-DV01, DV02 → P6-V07/V08 |
| Per-pCPU monotonic-time ownership | [architecture](02-architecture-and-state.md) §2; [time-core contracts](03-code-contracts-time-core.md) | W05-DV03 → P6-V08 |
| Arm/cancel/rearm deadline-event behavior, per-pCPU ownership | [deadline-timer contracts](04-code-contracts-deadline-timer.md) | W05-DV04, DV05 → P6-V07 |
| IRQ receipt integrated with the physical-IRQ lifecycle and per-pCPU diagnostics | [deadline-timer contracts](04-code-contracts-deadline-timer.md) §4; [architecture](02-architecture-and-state.md) §5 | W05-DV06 → P6-V07/V08 |
| One-shot, repeated, cancellation, rearm, entry/exit monotonicity, multi-pCPU acceptance | [workflow](05-implementation-workflow.md); [validation](06-validation-and-handoff.md) §1 | W05-DV07–DV09 → P6-V07, P6-V08 |
| Mechanism permits later scheduling without deciding scheduler policy | [foundations](01-scope-and-foundations.md) §4 (decision D5); [handoff](06-validation-and-handoff.md) §3 | W05-DV10 closure review |
| Factual timer contract handed to W06, W10–W11, W13, P7 | [handoff](06-validation-and-handoff.md) §3 | W05-DV10 |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is a P0 documentation
scaffold. There is no Cargo workspace, no Rust source, and no build target; all
code directories contain only `.gitkeep` markers. P1–P5 have approved plans and
task books but no implementation, implementation records, or verification
evidence (only `p0-w01` has a verification record; `p0-w02` has a proposed
design). Consequently every P6-W05 prerequisite is an **assumed contract** with
an explicit failure boundary, not an inspectable artifact. The full ledger is
in [01-scope-and-foundations.md](01-scope-and-foundations.md) §3; no ledger row
invents a crate, target, module tree, or platform behavior.

## Resolved design decisions and their authority

The numbered decisions (D1–D8), their rationale, and their authority basis are
recorded in [01-scope-and-foundations.md](01-scope-and-foundations.md) §4. In
brief: (D1) the per-pCPU EL2 physical timer (`CNTHP_*`) over `CNTPCT_EL0` is
the W05 register surface, conditional on the P1 timer-access baseline and W01
platform intake; (D2) deadlines are computed against the 64-bit counter with
checked arithmetic (compare-value programming, no 32-bit TVAL on the arm path);
(D3) repeated events are consumer-driven rearms with a deadline-anchored
bounded catch-up option owned by this design; (D4) the timer IRQ handler does
bounded work only and hands a bounded expiry record to the consumer; (D5) W05
exposes a mechanism, never a tick or preemption policy; (D6) the timer PPI
INTID is an intake-validated platform/arch value, never a Core constant;
(D7) all W05 mutable state is per-pCPU and owned by the owning pCPU's timer
object; (D8) monotonic-time reads use an explicit ordering discipline and are
verified against regression across Guest entry/exit windows.

## Work breakdown and loading order

1. Load this README and the Coding Guidelines; complete the coding preflight.
2. Load [01-scope-and-foundations.md](01-scope-and-foundations.md) to obtain
   the prerequisite contracts, failure boundaries, scope classification, and
   decisions D1–D8 before touching any file.
3. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   logical modules, per-pCPU ownership, state machines, and concurrency rules
   that the contracts assume.
4. Implement in the order given in
   [05-implementation-workflow.md](05-implementation-workflow.md), loading
   [03-code-contracts-time-core.md](03-code-contracts-time-core.md) for
   workflow steps 3–4 and
   [04-code-contracts-deadline-timer.md](04-code-contracts-deadline-timer.md)
   for workflow steps 5–7.
5. Close with [06-validation-and-handoff.md](06-validation-and-handoff.md):
   run the validation matrix, record run/not-run evidence in the verification
   record path above, and complete the handoff checklist.

## Explicitly excluded interfaces

W05 authorizes no public VM/vCPU/scheduler API, no cross-pCPU timer control
interface, no wall-clock or offset conversion, no Guest-visible timer surface,
and no timer-device abstraction. The only callers named by this design are the
EL2 time services themselves, the P6-W03 IRQ dispatch integration point, and
the W05 consumer registration boundary for the expiry record. Crate names,
module paths, and the physical file tree are not designed here; they are chosen
by the workspace-owning packages and the implementation record, subject to the
ADR layering rules (Core must not contain system-register access).

## Downstream handoff

Per the [plan index](../../../p6/plans/README.md) consumer map:

- **P6-W06** (`../p6-w06-guest-generic-timer/README.md`) receives the Host
  event basis: the typed time domains, the per-pCPU deadline-event contract,
  and the expiry-record mechanism it uses for deferred Guest-timer expiry
  evaluation.
- **P6-W10** (`../p6-w10-interrupt-semantics/README.md`) and **P6-W11**
  (`../p6-w11-validation-guest-interrupt-suite/README.md`) receive the
  one-shot/repeated/cancel/rearm behavior and its evidence paths for the
  timer-plus-vIRQ and Validation Guest timer scenarios.
- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives the
  timer telemetry counters, late-fire detection fields, and the declared
  environment limits for the latency baseline.
- **P7** receives only an evidenced per-pCPU timer mechanism and its limits —
  no tick policy, no preemption semantics, no scheduler API. P7 owns all
  runnable-state, wakeup, and scheduling decisions on top of it.

W05 must not freeze a scheduler interface, and no downstream consumer may
treat the W05 register sequences as a hardware contract beyond the declared
QEMU reference environment.
