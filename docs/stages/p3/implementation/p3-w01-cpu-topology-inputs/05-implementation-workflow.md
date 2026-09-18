# P3-W01 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing anything, the implementer verifies it has loaded the entry
README, the Coding Guidelines, and the documents they route to, and performs
read-only discovery (`git ls-files`; inspect the P2 handoff plan and
available P2 evidence). Implementation may proceed only if the task book §2
entry conditions hold: P0 governance, the P1 entry/runtime contracts, and the
P2 platform-discovery, boot-memory, and allocator contracts are available as
reviewed deliverables. Prerequisite gaps are stop conditions, not work
items: P3 must not re-derive P2 discovery, create a workspace, or select a
target triple to unblock itself.

Stop and obtain direction instead of guessing when:

- the P2 handoff does not deliver the assumed CPU-inventory semantics
  (Architecture Change Request against the P2 contract, per the entry
  README §Authority);
- the P1 design has already claimed an identity-interpretation role that
  conflicts with this design's split (README decision 1);
- an implementation appears to require a crate name, target, or workspace
  layout no approved design has fixed (blocker to record);
- evidence for a validation item cannot be produced in the available
  environment (record as not run with reason; never fabricate).

## 2. Ordered implementation steps

### Step 1 — implement identity types

Target: the logical module of
[03-code-contracts-cpu-identity.md](03-code-contracts-cpu-identity.md),
placed per the approved workspace design.

Work: implement `HardwareCpuId`, `MpidrValue`, `LogicalCpuId` with exactly
the stated construction authority, accessors, ordering, and reserved-bit
rejection. The MPIDR register read is the only `unsafe` and belongs to the
architecture side with a `SAFETY` comment referencing the P1 entry/runtime
contract.

Suggested observation: the project's format/lint commands; host-side unit
tests compile.

Acceptance: contracts of §1–§3 of the identity file hold field-for-field;
no arithmetic accessor; no `From<u64>`/`From<usize>` conversions.

Failure/blocker: a needed language or toolchain capability that is not part
of the pinned baseline is a blocker to record, never a feature-gate
addition.

Evidence: implementation record
(`../p3-w01-cpu-topology-inputs-record.md`).

### Step 2 — identity unit tests

Work: host-side tests for affinity extraction, canonical ordering, U/MT
bits, reserved-bit rejection, equality/ordering/display of
`HardwareCpuId`, and logical-id type behavior.

Acceptance: tests exercise every accessor and rejection path from the
contracts; they run under the P0 host-test gate.

Failure/blocker: a test that cannot be expressed without hardware is a
design smell to raise — identity logic must be host-testable.

Evidence: verification record
(`../../verification/p3-w01-cpu-topology-inputs-verification.md`).

### Step 3 — implement classification and intake

Target: logical modules B and C of
[04-code-contracts-topology-intake.md](04-code-contracts-topology-intake.md).

Work: implement `TopologyClass`, `UnavailableReason`, `TopologyError`,
`TopologyInputs` (frozen), and `build_topology_inputs` with the exact
pseudocode order: validate identities, classify, integrate the boot CPU,
bound-check, assign logical ids, freeze. The boot-CPU check precedes the
bound check (see §5 rationale). Implement the P2-facts adapter as a thin
translation with no acceptance decisions.

Acceptance: every anti-assumption rule of §3.4 is enforced by code or made
unrepresentable (e.g., no access to input ordering); intake is
all-or-nothing; `TopologyInputs` has no mutation API.

Failure/blocker: if P2's delivered facts cannot be translated without
guessing (missing identity, missing enablement), stop with the adapter
failure diagnostic; do not default values.

Evidence: implementation record.

### Step 4 — intake unit tests

Work: host-side tests covering: shuffled-order mapping stability; duplicate
and malformed identities; Possible/Unavailable classification with every
reason; boot-CPU unknown and not-Present failures; bound-exceeded failure;
dense logical-id invariant; all-or-nothing behavior (no aggregate on
error).

Acceptance: every error variant and every class is exercised; tests are
deterministic.

Failure/blocker: a case that cannot be expressed from the contracts is a
contract gap — raise it, do not extend the type ad hoc.

Evidence: verification record.

### Step 5 — enumeration diagnostic and boot wiring point

Target: logical module D; the single call site in boot global
initialization.

Work: emit the enumeration diagnostic — one line per entry (logical id,
hardware identity, class, reason if any), boot designation, class counts,
start-capability availability — per the P0 logging/trace governance, and
wire the single `build_topology_inputs` call at the point the approved
P3-W05 boot-phase design designates as global initialization. W01 owns the
output and the call contract; P3-W05 owns when the call may run.

Acceptance: the diagnostic renders every field a reviewer needs for P3-V01
mapping review; a fresh checkout's links and symbols resolve; the call site
is reachable in a QEMU `virt` boot via the P0 QEMU entry path for at least
one declared CPU count.

Failure/blocker: if the diagnostic or call cannot be exercised because no
P0/P1 boot path is available yet, record DV05/DV06 as not run with the
reason; do not build a private boot path.

Evidence: implementation record; boot output excerpt goes to the
verification record when captured.

### Step 6 — cross-count topology capture and closure review

Work: for each declared count (1, 2, 4, 8) reachable in the available
environment, capture one boot's enumeration output and verify stable
mapping and boot-CPU identification against expectations; record each
capture or its not-run status. Then run the closure review against
[06-validation-and-handoff.md](06-validation-and-handoff.md) and the
handoff checklist.

Acceptance: per-count evidence exists or is explicitly not run with reason;
the repeated matrix is explicitly deferred to
[P3-W13](../p3-w13-qemu-smp-regression/README.md) (stated, not claimed).

Failure/blocker: a mapping instability across boots of the same
configuration is a design failure — record as failed with diagnosis; do
not loosen the assignment rule to make it pass.

Evidence: verification record.

## 3. Evidence destinations

- Implementation record:
  `docs/stages/p3/implementation/p3-w01-cpu-topology-inputs-record.md`
  (created when implementation starts; decisions taken, deviations,
  changed files — no command logs).
- Verification record:
  `docs/stages/p3/verification/p3-w01-cpu-topology-inputs-verification.md`
  (created when evidence exists; commands, environment, run/not-run per
  validation item).

Neither file is created by this design; both are pointed to, not written.
