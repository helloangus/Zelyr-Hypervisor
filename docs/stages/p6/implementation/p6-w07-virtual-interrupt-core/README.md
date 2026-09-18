# P6-W07 Virtual Interrupt Core — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Scope:** The controller-independent vIRQ lifecycle required by
[P6-W07](../../plans/p6-w07-virtual-interrupt-core.md).
**Owner/change context:** P6-W07 implementation handoff.
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W07. It converts the bounded
work-package plan into a designed, code-bearing mechanism: the vCPU-scoped
virtual-interrupt state that safely preserves authorized, target-specific
events from injection through presentation hand-off to completion; the
authorization boundary for Guest-requested and Host-internal producers; the
repeated-arrival and unavailable-vCPU policies; and the diagnostics other
packages consume. W07 is the **single owner of pending/active software
semantics** for P6 virtual interrupts; the List-Register presentation of
that state is P6-W08, maintenance processing is P6-W09, and Guest-visible
masking/priority semantics are P6-W10. W07 deliberately does **not** design
LR mechanics, Guest GIC MMIO, scheduler wakeups, device passthrough, or any
crate/file layout.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), completes
the Coding-Guidelines preflight (repository `AGENTS.md`, documentation
index, ADR baseline, [P6 task book](../../task-book-v0.1.md), and the
[P6-W07 plan](../../plans/p6-w07-virtual-interrupt-core.md)), and then loads
only the linked supporting file needed for its assigned step:

| Assigned material | Supporting file |
|---|---|
| Baseline findings, scope classification, resolved decisions | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Logical modules, state ownership, lifecycle, concurrency model | [02-architecture-and-state.md](02-architecture-and-state.md) |
| vIRQ lifecycle contracts (state, inject, selection, completion, drain) | [03-code-contracts-virq-lifecycle.md](03-code-contracts-virq-lifecycle.md) |
| Authorization, validation, and error-taxonomy contracts (Guest HVC path) | [04-code-contracts-authorization-and-errors.md](04-code-contracts-authorization-and-errors.md) |
| Ordered implementation steps | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Validation matrix, error/security/observability model, handoff checklist | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Nothing in this design is an implementation or completion claim. Evidence is
recorded only in
`../../verification/p6-w07-virtual-interrupt-core-verification.md` (created
when evidence exists), and factual implementation traceability only in
`../p6-w07-virtual-interrupt-core-record.md` (created when work starts).

## Authority, constraints, and scope classification

The governing order is Architecture ADR → P6 task book → P6-W07 plan → this
design → Coding Guidelines. Binding ADR constraints: Guest-untrusted inputs
with VM-local fault containment (ADR-007, section 19); capability + rights +
generation as the authorization model, with no VM-ID or role shortcuts
(ADR-013, ADR-051); one authoritative owner per state datum and typed
identities with checked arithmetic (section 13); physical and virtual IRQs
modeled as distinct things (section 7); structured telemetry (ADR-048);
soft queues must be bounded (section 12). The task book §8 classifies Host
IRQ/vIRQ state representation, locking, and coalescing as Implementation
Choice resolved here.

Classification summary:

- **Required:** the per-vCPU vIRQ bank (pending/active software truth for
  the SGI/PPI bank and the intake-declared SPI window); injection with
  validation, dedupe, and bounded repetition accounting; the presentation
  claim/return protocol consumed by W08; completion reporting consumed from
  W09; priority carriage with validated range; the Guest HVC authorization
  path over the P5 boundary; vCPU-lifecycle drain; per-vCPU diagnostics and
  telemetry hooks.
- **Reserved:** LR allocation policy internals (W08's, evolved by W08/W09
  designs); Guest-chosen priorities and full GIC preemption semantics
  (W10 and later); device-originated injection (P9+); cross-VM signaling
  primitives (P12 Notification/IPC); pending+active simultaneous
  presentation nuances beyond the bounded P6 rule (see D6).
- **Out of Scope:** GIC List-Register mechanics (W08); maintenance
  processing (W09); Guest masking/priority semantics as Guest-visible
  contracts (W10); Validation Guest scenarios (W11); robustness/storm
  evidence (W12); telemetry collection and latency baseline (W13); the
  Linux-visible vGIC Distributor/Redistributor MMIO model (P8); scheduler
  wakeups (P7); crate/module paths and public API beyond the stated
  contracts.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| vIRQ request authorization | [foundations](01-scope-and-foundations.md) §4 (D4); [authorization contracts](04-code-contracts-authorization-and-errors.md) | W07-DV05 → P6-V11 basis, P6-V21 input |
| Target isolation | [architecture](02-architecture-and-state.md) §3, §5; [lifecycle contracts](03-code-contracts-virq-lifecycle.md) §2 | W07-DV06 → P6-V18 basis |
| Pending/deferred/presented/active/completed semantic coverage | [architecture](02-architecture-and-state.md) §4; [lifecycle contracts](03-code-contracts-virq-lifecycle.md) §3–§5 | W07-DV02–DV04 → P6-V11, P6-V12 |
| Repeated arrival policy | [foundations](01-scope-and-foundations.md) §4 (D5); [lifecycle contracts](03-code-contracts-virq-lifecycle.md) §4 | W07-DV04 → P6-V14 basis |
| Unavailable-vCPU outcomes | [lifecycle contracts](03-code-contracts-virq-lifecycle.md) §3, §6 | W07-DV03 |
| Diagnostic/telemetry boundaries | [architecture](02-architecture-and-state.md) §7; [validation](06-validation-and-handoff.md) §2 | W07-DV07 |
| Controller independence | [architecture](02-architecture-and-state.md) §1–§2 (Core-domain placement) | W07-DV08 review |
| Contract handoff to W08–W13 and P7/P8 | [handoff](06-validation-and-handoff.md) §3 | W07-DV08 |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is a P0 documentation
scaffold — no Cargo workspace, no Rust sources, no implemented P3/P4/P5
foundations, no W03 implementation, and no P6 implementation designs in this
worktree yet (siblings W01–W04 and W09–W13 are being prepared in parallel on
this branch). Every prerequisite is an **assumed contract** with an entry
check and failure boundary; the ledger is in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §3 and the
contract table in §5. No row invents a crate, target, handle encoding, or
Guest-visible vGIC model.

## Resolved design decisions and their authority

The numbered decisions (D1–D8), their rationale, and authority basis are in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §4. In brief:
(D1) W07 is the sole owner of per-vCPU pending/active software truth;
presentation residency is W08's datum and maintenance is W09's; (D2) state
is keyed by typed `(VcpuId, VirqId)` with a per-vCPU bank covering the
SGI/PPI range and the intake-declared SPI window; (D3) delivery semantics
are queue-and-preserve: an event for an absent vCPU stays pending until
presentation, with no wakeup (P7); (D4) authorization distinguishes
Host-mechanism producers from Guest HVC requests, the latter constrained to
the SGI class through the P5 capability boundary; (D5) repeated arrivals
coalesce with saturating occurrence counts — no unbounded queues;
(D6) selection excludes `active` vINTIDs until completion is reported, a
bounded P6 rule recorded with its Reserved extension; (D7) priority is a
validated, carried field whose Guest-visible semantics belong to W10;
(D8) destruction drains determinately.

## Work breakdown and loading order

1. Load this README and the Coding Guidelines; complete the coding
   preflight.
2. Load [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   prerequisite contracts, failure boundaries, scope classification, and
   decisions D1–D8.
3. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   logical modules, the ownership table, the state model, and concurrency
   rules.
4. Implement in the order given in
   [05-implementation-workflow.md](05-implementation-workflow.md), loading
   [03-code-contracts-virq-lifecycle.md](03-code-contracts-virq-lifecycle.md)
   for workflow steps 3–5 and
   [04-code-contracts-authorization-and-errors.md](04-code-contracts-authorization-and-errors.md)
   for workflow steps 6–7.
5. Close with [06-validation-and-handoff.md](06-validation-and-handoff.md):
   run the validation matrix, record run/not-run evidence in the
   verification record path above, and complete the handoff checklist.

## Explicitly excluded interfaces

W07 authorizes no List-Register, maintenance, or physical-GIC interface; no
Guest-visible MMIO or vGIC model; no scheduler or wakeup API; no IPC/
Notification surface; and no device-driver injection API beyond the
Host-mechanism producer contract stated here. Its only named collaborators
are: W06 (timer expiry producer), W08 (presentation claim/return), W09
(completion reporting path), W03 (physical-IRQ lifecycle as the distinct
Host-side domain), and the P5 hypercall/dispatch/capability boundary for
Guest requests. Crate names, module paths, and file trees are not designed
here; W07 is Core-domain code with no system-register access at all.

## Downstream handoff

Per the [plan index](../../../p6/plans/README.md) consumer map:

- **P6-W08** (`../p6-w08-gic-virtualization-interface/README.md`) receives
  the pending-state semantics and the presentation claim/return protocol:
  W08 consumes selections, claims, and returns presentations; it never
  mutates W07 bits directly.
- **P6-W09** (`../p6-w09-maintenance-interrupt/README.md`) receives the
  completion-reporting contract that lets maintenance release presentations
  and re-pend evicted state without loss or duplicate completion.
- **P6-W10** (`../p6-w10-interrupt-semantics/README.md`) receives the
  lifecycle and priority-carriage field as the substrate for bounded
  masking/priority/repeated-event semantics (including the rule that Guest
  masking does not imply completion).
- **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`)
  receives the injection request behavior its VG-IRQ scenarios exercise
  (single, multiple-pending, repeated, cross-vCPU isolation).
- **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`) receives the
  validation/rejection classes and error taxonomy to stress for
  P6-V21-style evidence.
- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives
  lifecycle counters and the deferred-delivery accounting.
- **P7/P8** receive no frozen vGIC or scheduler API: P7 owns wakeup and
  runnable-state policy over pending state; P8 owns the Linux-visible vGIC
  model. Neither may infer an ABI from W07's internal contracts.

No consumer may treat W07's in-memory layout as a persistent or wire
format, and none may bypass the authorization path.
