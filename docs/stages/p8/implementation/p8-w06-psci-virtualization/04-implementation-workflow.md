# P8-W06 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
(preflight included), and the prerequisite evidence named there. Useful
read-only discovery: `git ls-files` (confirm the crate layout that the
approved workspace decision actually produced — never assume one from names)
and a search of tracked documents for any existing PSCI statement that this
design must not silently contradict.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 machine-contract gate has not approved the PSCI rows (subset,
  conduit, reported version) — implementing code must not treat them as
  frozen; record the blocker;
- the P7 admission/stop seams cannot express the transitions in
  [03](03-code-contracts-cpu-lifecycle.md) §2–§3 — record an `Architecture
  Change Request` against the seam; do not patch scheduler policy locally;
- the P5 hypercall numbering collides with the PSCI function-ID namespace —
  stop per [02 §1](02-code-contracts-psci-dispatch.md); escalate to
  `ADR Required` if resolution forces an ABI change;
- W03/W04 contracts disagree with the entry-state or DTB facts consumed here —
  raise the conflict at the W03/W06 or W04/W06 seam; do not edit their
  contracts from this package;
- implementation appears to require EL3/firmware interaction, an SMC-conduit
  variant, or CPU_SUSPEND — all Reserved (README classification); a trigger
  is a decision request, not a local choice.

## 2. Ordered implementation steps

### Step 1 — freeze gate check and fact registration

Target: `VmPsciConfig` population path; implementation record.

Work: confirm the W02 gate has approved the PSCI machine facts; register the
frozen subset, conduit, and reported version as the only source of
`VmPsciConfig` values, built from approved machine-contract data (no literal
constants in dispatch logic for subset membership or version). Record the
approved values and their authority location.

Suggested observation: review that a single declaration feeds dispatch and
query handlers ([02 §0.3](02-code-contracts-psci-dispatch.md)).

**Acceptance:** all PSCI Guest-visible values derive from the approved
machine facts; no handler hard-codes subset membership or version.  
**Failure/blocker:** missing gate approval is a recorded blocker (§1); no
temporary values.

### Step 2 — dispatch and query layer (M1/M2)

Target: the HVC exception path classification and the query handlers.

Work: implement `psci_handle_hvc` with its classification ordering, the
VERSION/FEATURES handlers, the error-code mapping of
[02 §0.2](02-code-contracts-psci-dispatch.md), and the validation helper.
Verify function-ID constants against the pinned DEN 0022 revision (a mismatch
is a bug fix, not a design change). Keep the path free of allocation,
blocking, and Host-state reads.

**Acceptance:** unknown IDs answer `NOT_SUPPORTED`; FEATURES probe matrix
agrees exactly with the frozen subset; every path writes x0 only and emits
one telemetry event.  
**Failure/blocker:** namespace collision with P5 numbering stops per §1.

### Step 3 — vCPU lifecycle bridge (M3)

Target: `psci_cpu_on`, `psci_cpu_off`, `psci_affinity_info` and their seams
to P7 lifecycle and the W03 entry state.

Work: implement target validation, the pending-entry commit, admission via
the P7 seam, the stop path with its quiesce hooks ([03 §3](03-code-contracts-cpu-lifecycle.md)
calls the timer [W08] and vGIC [W07] hooks in their defined order), and the
read-only affinity query. Re-check target state under lock (the race window
between validation and commit is where a duplicate-ON would slip through).

**Acceptance:** S2/S3 behaviors hold at unit level where host-side tests
exist; error paths leave no residue; no handler waits on another vCPU.  
**Failure/blocker:** seam mismatch stops per §1; do not fake admission.

### Step 4 — VM system lifecycle bridge (M4)

Target: `psci_system_off` / `psci_system_reset` and the VM-transition seam.

Work: implement the VM-scoped transition requests with the non-joining
stop-request fan-out of [03 §5](03-code-contracts-cpu-lifecycle.md). Confirm
no code path touches Host power state or other VMs.

**Acceptance:** transition refusal returns DENIED with the VM unchanged;
success never returns control to the calling Guest; other-VM state provably
unread on this path (code review).  
**Failure/blocker:** if the VM lifecycle lacks a Reset/PowerOff transition,
stop — this is the ADR §4.1 lifecycle owner's seam, not a local state bit.

### Step 5 — diagnostics and fault integration (M5)

Target: telemetry events and the W13 escalation hook.

Work: wire the one-event-per-dispatch rule, bounded counters, and the
internal-failure escalation ([02 §0.2](02-code-contracts-psci-dispatch.md),
README decision D7). Verify events are prunable and filterable (ADR-048) and
that telemetry failure cannot alter a response.

**Acceptance:** every dispatch outcome has exactly one event; counters
bounded; W13 receives the escalation path in its inventory.  
**Failure/blocker:** a telemetry design that cannot run in VM-exit context is
a P0-contract conflict to raise, not a local workaround.

### Step 6 — integration with W07/W08 quiesce hooks and W10

Target: the cross-package seams only.

Work: confirm the CPU_OFF quiesce hook signatures expected here match what
[W07](../p8-w07-linux-vgicv3/README.md) and
[W08](../p8-w08-linux-timer-integration/README.md) design, and that W10 can
reach CPU_ON as its only start path. Mismatches are resolved at the seam in
the owning design's direction; this package does not redefine hooks owned
elsewhere.

**Acceptance:** hook names/semantics agree with the sibling designs; no
duplicated lifecycle logic.  
**Failure/blocker:** disagreement stops the integration step and records the
conflict for the coordinator.

### Step 7 — scenario execution and closure review

Work: run the validation matrix in
[05](05-validation-and-handoff.md) that the current integration state
permits; record evidence (commands, output, environment) in
`../../verification/p8-w06-psci-virtualization-verification.md` and decisions
in `../p8-w06-psci-virtualization-record.md`. Completion is claimed only in
the verification record, only for what actually ran.

**Acceptance:** matrix entries are passed/failed/blocked/not-run with
evidence; P8-V09 rows marked run have real output.  
**Failure/blocker:** a failed containment scenario is evidence of a failure —
record and fix through the design's error paths, not by weakening validation.

## 3. Ordering rationale

Dispatch (Step 2) precedes lifecycle (Step 3) because validation and error
mapping are load-bearing for containment; the system bridge (Step 4) follows
because it reuses the stop seam; telemetry (Step 5) is wired before scenario
execution so every observed outcome is attributable.
