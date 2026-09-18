# P6-W06 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W06 design entry](README.md).

## 1. Preconditions and failure boundary

Complete the Coding-Guidelines preflight and load
[01](01-scope-and-foundations.md), [02](02-architecture-and-state.md), and
the contract file for the step at hand. Because W05, W07, W09, P4, and P5
are planned but not implemented at design time, step 1 is a real entry
review against the assumed-contract table ([01] §5). Stop and record a
blocker instead of guessing when: an upstream contract is missing,
contradictory, or delivered with differences that matter (owner: the
conflicting package — W06 never repairs a predecessor); the P1 trap
posture differs from D1; the W03 lifecycle cannot express the D6 split
completion (Architecture Change Request path); or a step appears to
require scheduler wakeup, LR programming, or machine-layout decisions
(scope violation — P7/W08/P8 respectively).

## 2. Ordered implementation steps

### Step 1 — entry review and cross-design reconciliation

Target: implementation record (`../p6-w06-guest-generic-timer-record.md`,
created in this step).

Work: inspect delivered evidence for every §5 contract of
[01](01-scope-and-foundations.md): P4 entry/exit and vCPU extension area,
P5 error boundary, W05 time basis, W07 inject/completion contracts, W03
split-completion semantics, W09 completion hook, and (for P6-V19) any
multi-vCPU capability evidence. Record each as available / with-
differences / missing, and reconcile the D6 alignment with the W03/W09
designs explicitly.

**Acceptance:** every contract has a recorded status; the D6 split-
completion expectation is either confirmed by the delivered W03/W09
contracts or formally raised. **Failure/blocker:** missing or
contradictory contracts stop W06 at this step with a recorded blocker
naming the owning package.

### Step 2 — place the logical modules and declare the unsafe boundary

Target: workspace layout chosen by the workspace-owning packages (logical
modules `vcpu-timer-state`, `vtimer-expiry`, `vtimer-entry-exit`,
`vtimer-ppi-handler` per [02](02-architecture-and-state.md) §2).

Work: map logical modules onto the actual tree respecting ADR layering
(Core-domain: state and pure evaluation, hostable; Arch-domain: all
system-register access). Declare W06 `unsafe` inventory entries with draft
`SAFETY` justifications.

**Acceptance:** no register access outside the Arch-domain modules; the
pure evaluator compiles and tests on the host. **Failure/blocker:** a
workspace that cannot express the split is a workspace-package blocker.

### Step 3 — typed domains, control image, and pure evaluator

Target: `vcpu-timer-state` values and `vtimer-expiry` (contracts
[03](03-code-contracts-vcpu-timer-state.md) §1–§2,
[04](04-code-contracts-expiry-and-delivery.md) §2).

Work: implement `VirtCounter`/`VirtDeadline`/`VirtualOffset` conversions,
`GuestTimerCtl`, `VcpuTimerState` with initialization, and `evaluate()`;
add host-side unit tests for the full verdict matrix and conversion
boundaries.

**Acceptance:** evaluator is total and pure; all Guest-domain arithmetic
is typed and checked; unit tests pass under the pinned toolchain.
**Failure/blocker:** any need for domain-mixing arithmetic is a design
conflict — stop and record.

### Step 4 — entry/exit sequences

Target: `vtimer-entry-exit` (contracts
[03](03-code-contracts-vcpu-timer-state.md) §3).

Work: implement `vtimer_on_entry` / `vtimer_on_exit` with the §3.3 barrier
table; wire them into the P4 transition points; verify the impossible-
state recovery path on entry.

**Acceptance:** after every exit the record is authoritative, the
hardware timer is disabled, and no physical interrupt is held; after every
entry the registers match the record. **Failure/blocker:** an unexpressed
transition point in the P4 delivery is a P4 boundary blocker.

### Step 5 — deferred delivery and PPI conversion

Target: `vtimer-ppi-handler` and delivery contracts
([04](04-code-contracts-expiry-and-delivery.md) §3–§4).

Work: implement `request_deferred_timer_event` over the W07 contract,
`vtimer_ppi_handler` with the hold-and-release rule, and
`on_guest_completion` over the W09 hook; bind the handler through the W03
dispatch for the resolved INTID.

**Acceptance:** exactly one outstanding physical INTID per vCPU at all
times; every conversion/coalescing/unexpected outcome counted; no loop or
allocation in IRQ or maintenance context. **Failure/blocker:** W07/W09
contract gaps are step-1-class blockers or cross-design amendments.

### Step 6 — trapped-access Guest-fault path and intake

Target: `handle_trapped_timer_access` and `vtimer_intake`
([04](04-code-contracts-expiry-and-delivery.md) §5–§6).

Work: implement the trapped-access disposition over the P5 error boundary
and the intake checks (trap posture, INTID resolution, identity-mapping
cross-check).

**Acceptance:** trapped access produces one controlled Guest fault and a
counter increment; intake failure disables delivery with a precise
diagnosis while save/restore stays safe. **Failure/blocker:** a P5
taxonomy gap blocks the fault path; a posture mismatch is a P1 boundary
conflict.

### Step 7 — acceptance scenarios and evidence

Target: verification record
(`../../verification/p6-w06-guest-generic-timer-verification.md`).

Work: run the validation matrix ([06](06-validation-and-handoff.md) §1):
Guest programs/reads/enables the timer and observes the handled event;
exit/HVC preservation; deferred delivery after an expired-at-exit window;
masking honor; multi-vCPU isolation (or the recorded stage block).
Record commands, environment, outputs, timestamps, explicit not-run
entries; complete the implementation record (changed files, unsafe delta,
deviations).

**Acceptance:** every W06-DV row has a run status with evidence or an
explicit reason; P6-V09/P6-V10/P6-V19 are claimable only from the
verification record.

### Step 8 — closure review and handoff

Work: run the [06](06-validation-and-handoff.md) §3 handoff checklist;
verify no scheduler/machine-ABI semantics leaked into W06's surface; check
the handoff statements to W10–W13 and P7/P8 against delivered evidence.

**Acceptance:** checklist complete; handoff wording matches evidence.

## 3. Evidence destinations

- Implementation record: `../p6-w06-guest-generic-timer-record.md`
  (created at step 1).
- Verification record:
  `../../verification/p6-w06-guest-generic-timer-verification.md` (created
  when evidence exists).
- No completion claim may appear in any design file.

The validation matrix, error/security/observability model, and handoff
checklist that close this workflow are in
[06-validation-and-handoff.md](06-validation-and-handoff.md).
