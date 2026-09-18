# P8-W08 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W08 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
(preflight included), and the P6/P7 contracts cited there. Useful read-only
discovery: `git ls-files` (confirm the actual crate layout produced by the
approved workspace decision) and a search for any existing timer handling
this design must not contradict.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 gate has not approved the frequency, offset, or exposed timer
  class — no value may be embedded ([02 §1](02-code-contracts-timer-regs.md));
  record the blocker;
- P6-W06 cannot express the state operations M2 needs, or P6-W05 lacks a
  cancellable per-pCPU deadline primitive — record an `Architecture Change
  Request` against the P6 contract; never store timer state locally;
- P7-W06 cannot accept the declared wakeup-source set, or P7-W04 lacks the
  transition hooks ([01 §6](01-architecture-and-state.md)) — raise at the P7
  seam; do not poll or spin as a substitute;
- W04's DTB timer node disagrees with the machine facts (class, frequency) —
  the P8-V05 consistency review fails; fix belongs to the machine contract,
  not this code;
- implementation seems to require the physical timer class, physical-counter
  exposure, or wall-clock semantics — Reserved/Out of Scope (README
  classification).

## 2. Ordered implementation steps

### Step 1 — machine facts registration

Target: `VmTimeFacts` population from approved machine-contract data;
consistency check against the W04 DTB timer-node facts.

Work: register frequency, per-VM offset, and the exposed class from the
approved gate values; fail VM creation on absence; verify the offset choice
makes the counter subtraction wrap-free for the VM lifetime.

**Acceptance:** all Guest-visible time values derive from one approved
source; creation fails cleanly without facts (DV01).  
**Failure/blocker:** missing gate approval is a recorded blocker (§1).

### Step 2 — time source (M1)

Target: `guest_read_vcounter`, `guest_read_freq`, and the access routing of
[02 §2](02-code-contracts-timer-regs.md)/§3.4.

Work: implement counter/frequency semantics and the W05-classified routing
table for time registers. Verify I1/I2 by construction (immutable offset,
single source).

**Acceptance:** counter reads are monotonic and coherent by construction
review; disallowed accesses yield the classified contained outcome.  
**Failure/blocker:** a CNTHCTL/trap-policy conflict with W05/P1 stops at
that seam.

### Step 3 — timer registers (M2)

Target: the CTL/CVAL/TVAL contracts of
[02 §3](02-code-contracts-timer-regs.md) over P6-W06 operations.

Work: implement the three register handlers with atomic commit+re-arm under
the vCPU timer lock; verify reserved-bit masking, checked arithmetic, and
the immediate-assertion path (elapsed CVAL at write time).

**Acceptance:** readback semantics exact (TVAL never stored); assertion
condition exact at the contract level; telemetry one-per-program.  
**Failure/blocker:** a missing P6-W06 operation blocks per §1.

### Step 4 — expiry, delivery, and transition hooks (M3/M4)

Target: [03 §1–§3 and §5](03-code-contracts-expiry-wakeup.md) — re-arm,
expiry, delivery-on-entry, and the P7-W04 hook pair.

Work: implement the fail-toward-delivery rule, stale-expiry re-validation,
level-configured PPI injection through the W07 bridge, and the
out/entry hook pair. Confirm hook order with P7-W04's contract.

**Acceptance:** no lost wakeup by construction (I4); no cross-vCPU arm
residue (§5); repeated switch cycles state-identical.  
**Failure/blocker:** a P6-W07/P6-W05 seam gap blocks per §1.

### Step 5 — WFI integration (M5)

Target: [03 §4](03-code-contracts-expiry-wakeup.md).

Work: implement the WFI trap path with the exact wakeup-source set and the
WFI-as-noop rule for pending events; WFE per W05 classification.

**Acceptance:** blocked vCPUs wake on timer assertion, vIRQ pending, or Host
event; a Guest cannot add sources; no busy-loop (verified against W17
observations later).  
**Failure/blocker:** a P7-W06 mismatch blocks per §1.

### Step 6 — telemetry and consumer seams

Target: GTT events and cross-package agreement.

Work: wire program/expire/deferred/wakeup events (P6-W13 method) and confirm
seams with [W07](../p8-w07-linux-vgicv3/README.md) (injection entry),
[W09](../p8-w09-virtual-console-single-cpu-linux/README.md),
[W10](../p8-w10-linux-smp-bringup/README.md), and
[W11](../../plans/p8-w11-scheduler-linux-integration.md) (hook order).

**Acceptance:** events present/shape-agreed; seam agreement recorded.  
**Failure/blocker:** disagreement stops integration and is recorded for the
coordinator.

### Step 7 — scenario execution and closure review

Work: run the [05](05-validation-and-handoff.md) matrix as far as current
integration permits; evidence to
`../../verification/p8-w08-linux-timer-integration-verification.md`;
decisions to `../p8-w08-linux-timer-integration-record.md`. Completion
claims only in the verification record, only for what ran.

**Acceptance:** matrix entries passed/failed/blocked/not run with evidence;
P8-V11 rows marked run have real output.  
**Failure/blocker:** a failed invariant row (I1–I6) is recorded and fixed
through the design's state model, never by weakening the scenario.

## 3. Ordering rationale

Facts (Step 1) → time source (Step 2) → registers (Step 3) → expiry/hoooks
(Step 4) → WFI (Step 5) mirrors Linux's own bring-up order (clocksource →
clockevent → idle), so each step is exercisable against the next stage of a
real Linux boot log.
