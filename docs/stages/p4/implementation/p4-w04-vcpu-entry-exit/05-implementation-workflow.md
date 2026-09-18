# P4-W04 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, the implementer verifies the Coding Guidelines preflight and
the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry review
status for rows R01–R05 (P1 EL2/exception/baseline/Stage-1/lifecycle), R13–R16
(P3 CPU identity/local state/synchronization/transport), and R18–R20 (P0
toolchain/newtypes/logging/unsafe governance). Per
[01 §2](01-scope-and-foundations.md): Rust-side contracts (context types,
construction, classification tables) can proceed against assumed interfaces,
but on-target entry evidence is gated on M2/M3/M4 (stable EL2, vector path,
EL2 baseline) and M5 (per-CPU frame discovery).

Stop and record instead of improvising when:

- the delivered EL2 vector path cannot host the Guest-exit tail (M3) — the
  exit boundary cannot be built safely; record the blocker; do not create a
  parallel vector table in W04;
- the P3 per-CPU mechanism differs from the assumed discovery form (M5) —
  adapt the stub's discovery seam to the delivered mechanism and record the
  change; the frame contract itself does not change;
- exact HCR/trap encodings disagree between the architecture revision and
  observed QEMU behavior — record a Specification Investigation item (W01
  A7), keep the architectural intent as the contract, and note the QEMU
  behavior in the implementation record;
- a needed W02/W03 contract changed shape (activation signature,
  `GuestInput` fields) — resolve by joint design note with that package;
  never bridge informally in code;
- the world-switch needs a new `unsafe` shape beyond
  [01 D9](01-scope-and-foundations.md) — that is a design change: extend the
  decision record and the unsafe inventory justification together, or stop.

## 2. Ordered implementation steps

### Step 1 — context types and construction

Target: `vcpu-ctx` types and `Vcpu::construct`.

Work: implement the EL1-only context type, the pure constructor, and the
validation per [04 §1–§2](04-code-contracts-vcpu-run.md), with the
determinism and rejection test suites.

**Acceptance:** construction is byte-deterministic; invalid scenario/entry/
stack inputs rejected; no non-EL1 state representable.  
**Failure/blocker:** a W03 `GuestInput` shape mismatch is resolved by joint
note (M8) before continuing.

### Step 2 — frame layout and sysreg helpers

Target: `ws-frame` and the `ws-switch` helper layer.

Work: define the fixed frame layouts and implement the sysreg helpers per
[03 §1–§2](03-code-contracts-world-switch.md); add inventory entries.

**Acceptance:** layout review passes against the register-ownership table;
helpers compile for the AArch64 target; SideEffect-bearing reads are named.  
**Failure/blocker:** a helper whose side effects cannot be scoped is split
or moved behind W02's activation contract; no silent side-effect reuse.

### Step 3 — guest-entry stub

Target: `ws_switch::ws_guest_enter`.

Work: implement the naked entry per [03 §3](03-code-contracts-world-switch.md)
with the normative order (host save → guest load → ISB → ERET) and the
generated-code review that no compiler output runs in the handoff window.

**Acceptance:** review checklist of [03 §5](03-code-contracts-world-switch.md)
passes; the stub appears in the unsafe inventory with SAFETY citations.  
**Failure/blocker:** compiler-generated spills inside the handoff window are
fixed by adjusting the stub (not by accepting them); if impossible, stop and
redesign the boundary split.

### Step 4 — guest-exit stub

Target: the Guest-exit vector tail per [03 §4](03-code-contracts-world-switch.md).

Work: implement unconditional frame capture, host restore, and the branch to
the Rust handler; integrate with the P1 vector path (assumed M3) without
modifying its fatal-path semantics.

**Acceptance:** capture completeness review (all fields written before any
fault-capable step); recursive-entry review passed; inventory updated.  
**Failure/blocker:** an M3 mismatch (vector path shape) blocks this step —
record and reconcile upstream; do not fork the vector table.

### Step 5 — run control and classification

Target: `vcpu-run` (`run_entry`, `classify_minimal`, `decide`, stop/teardown
sequencing).

Work: implement per [04 §3–§4](04-code-contracts-vcpu-run.md) with the
table-driven classification tests and the budget guardrail.

**Acceptance:** classification table tests pass host-side; state transitions
emit events; budget exhaustion stops cleanly; misuse states escalate as
designed.  
**Failure/blocker:** a frame field needed for classification but missing
from the capture design is a [03](03-code-contracts-world-switch.md) design
fix first (capture is unconditional), never a best-effort read in Rust.

### Step 6 — integration with W02/W03

Target: the construction-order integration of
[P4-W03 §02 §4](../p4-w03-guest-memory-image/02-architecture-and-state.md)
and W02 activation.

Work: wire the full setup → run → stop → teardown sequence; verify the
console-quiet duty and the sequencing duties (space destroy before RAM
release) with integration assertions.

**Acceptance:** the ordered integration runs on target; teardown restores
accounting exactly; second iteration (W07 repeat) starts clean.  
**Failure/blocker:** consumer-contract drift is a joint design note (M7/M8);
the integration step does not adapt unilaterally.

### Step 7 — on-target entry/exit/re-entry/stop evidence

Target: P4-V04 and the W04-shareable part of P4-V09, via
[P4-W05](../p4-w05-validation-guest/README.md) scenarios and recorded by
[P4-W08](../p4-w08-qemu-integration-regression/README.md).

Work: exercise first entry (EL1 marker), classified exit with EL2 recovery,
one re-entry, and the defined stop on the reference platform.

**Acceptance:** Guest PC/SP/state correct at EL1; every exit leaves EL2
live and diagnosable; re-entry and stop succeed with preserved diagnostics.  
**Failure/blocker:** failures diagnose via the captured frame and W06;
record as failed with diagnosis; never adjust capture or restore ordering to
make a symptom disappear.

### Step 8 — closure review

Work: run the [06](06-validation-and-handoff.md) matrix and handoff
checklist; record facts for W09 (protocol, limitations: interrupt masking,
EL1-baseline non-persistence, exit-class list).

**Acceptance:** matrix reviewed; handoff complete; no completion claims
outside the verification record.

## 3. Evidence destinations

- Implementation facts and decisions taken:
  `../p4-w04-vcpu-entry-exit-record.md` (created when work starts).
- Commands, environments, results, run/not-run:
  `../../verification/p4-w04-vcpu-entry-exit-verification.md`.
- Unsafe inventory delta (stubs, helpers): P0 inventory location (W01 R20),
  linked from the implementation record.
