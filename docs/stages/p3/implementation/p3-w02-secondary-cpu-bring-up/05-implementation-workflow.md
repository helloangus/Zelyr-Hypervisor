# P3-W02 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing anything, the implementer verifies it has loaded the entry
README, the Coding Guidelines, and the routed documents, and performs
read-only discovery. Implementation proceeds only when:

- [P3-W01](../p3-w01-cpu-topology-inputs/README.md) identity/topology types
  and the `StartCapabilityFacts` record exist per its design (or the
  coordinated module location the workspace design fixed);
- the P1 entry/runtime contracts and the P1-W04 per-CPU-reproducible
  baseline statement are available as reviewed deliverables;
- the P2 allocator contract and the P2 handoff's PSCI facts are available;
- the W03 transition operations and the W05 phase gate exist at least as
  agreed cross-design contracts (sibling designs land on the same branch;
  a missing counterpart contract is a coordination blocker to record, not
  a license to improvise semantics).

Stop and obtain direction when: the P1 baseline is documented as not
per-CPU reproducible (Architecture Change Request against the P1
contract); the P2 facts lack conduit/function identifiers (fail closed
with `StartUnavailable`, and record the platform gap — do not guess); or
an implementation step seems to require hotplug, retry, guest PSCI, or a
lock primitive (scope violation — raise it).

## 2. Ordered implementation steps

### Step 1 — implement the PSCI call boundary

Target: module B of
[04-code-contracts-start-requester.md](04-code-contracts-start-requester.md)
§2–§3.

Work: implement `psci_cpu_on` and `PsciCallError` with the conduit
dispatch from recorded facts. This is the package's only new `unsafe`; the
`SAFETY` comment must name the register contract, the exception-level
assumption inherited from the P1 baseline, and the audit owner.

Acceptance: no platform name or literal function id outside the recorded
facts; exhaustive status mapping; single call site in the sequencer.

Failure/blocker: a firmware behavior the recorded facts cannot express is
a P2-contract gap — record and stop.

Evidence: implementation record
(`../p3-w02-secondary-cpu-bring-up-record.md`).

### Step 2 — outcome model and sequencer (host-testable core)

Target: §1, §4, §5, §6 of the requester file.

Work: implement `StartPhase`, `SecondaryStartOutcome`,
`bring_up_secondaries`, and the bounded poll. Structure the firmware call
and mailbox reads behind internal seams so the sequencer logic is
host-testable with a mocked call/read boundary (a test seam, not a public
abstraction).

Suggested observation: host-side test run under the P0 gate.

Acceptance: mapping rules of §4 hold exhaustively in tests (error,
success, late secondary failure, timeout); the loop never re-dispatches;
per-candidate isolation holds (one failure does not stop others); the
phase-gate precondition is asserted.

Failure/blocker: a test that requires real firmware is misplaced — raise
the seam design, do not embed QEMU in unit tests.

Evidence: verification record
(`../../verification/p3-w02-secondary-cpu-bring-up-verification.md`).

### Step 3 — provisional environment

Target: module D; entry-parameter block and stack allocation per
[03-code-contracts-secondary-entry.md](03-code-contracts-secondary-entry.md)
§4.

Work: implement pre-release allocation from the P2 page allocator, block
population, and the ownership transfer/quarantine rules. Fix
`PROVISIONAL_STACK_SIZE` in the implementation record with rationale.

Acceptance: no CPU_ON before all allocations complete; page alignment per
the P2-W04 contract; quarantine leaves no free path.

Failure/blocker: allocator contract mismatch (no alignment guarantee) is a
P2-contract conflict — record, do not work around with image arrays.

Evidence: implementation record.

### Step 4 — secondary entry stub

Target: module C;
[03](03-code-contracts-secondary-entry.md) §1–§3, §5.

Work: implement the assembly entry (stack load, branch), the Rust
revalidation/identity/transition/report path, and `park_failed`. The
assembly is the minimal necessary boundary per ADR-006; every check
ordered as in §2.

Acceptance: every failed check terminates in `park_failed`; the mailbox
write is unreachable without W03's accepted transition; no assumption
beyond the PSCI-specified entry state.

Failure/blocker: an entry-state item that cannot be checked without
privileges the baseline does not grant is a P1-contract issue — record
and stop.

Evidence: implementation record.

### Step 5 — diagnostics wiring

Work: emit per-CPU bring-up diagnostics (requester side and parked-failure
side) per the P0 logging/trace governance; ensure each candidate CPU
produces at least one attributable line; coordinate event identifiers with
the P3-W11 catalog owner if the namespace requires registration.

Acceptance: success, rejected-request, and secondary-failure paths each
produce CPU+phase-attributed output.

Evidence: implementation record; captures go to the verification record.

### Step 6 — QEMU evidence capture

Work: via the P0 QEMU entry path, capture bring-up on each declared count
(1, 2, 4, 8) reachable in the environment: all expected secondaries
report `Started`, distinct provisional stacks, correct CPU attribution.
Then exercise the induced failure input: a start request targeting an
identity outside the inventory (absent CPU) and record the
`RequestRejected` diagnostic. Record every capture or its not-run status
with reason; repeated-matrix execution is
[P3-W13](../p3-w13-qemu-smp-regression/README.md)'s.

Acceptance: per-count capture (or not-run entry) plus the induced-failure
capture exists; timeout-on-target is recorded as a stated limit proven
only host-side.

Failure/blocker: a non-deterministic bring-up failure is evidence of a
failure — record as failed with diagnosis; do not widen `POLL_BOUND` to
make flakiness disappear silently (a bound raise is a recorded decision
with rationale).

Evidence: verification record.

### Step 7 — closure review

Work: run the review matrix in
[06-validation-and-handoff.md](06-validation-and-handoff.md); verify the
handoff checklist; confirm the W03/W05 consumer contracts match what was
implemented (transition names, gate call points); record deviations.

## 3. Evidence destinations

- Implementation record:
  `docs/stages/p3/implementation/p3-w02-secondary-cpu-bring-up-record.md`
  (created when implementation starts).
- Verification record:
  `docs/stages/p3/verification/p3-w02-secondary-cpu-bring-up-verification.md`
  (created when evidence exists).

Neither file is created by this design.
