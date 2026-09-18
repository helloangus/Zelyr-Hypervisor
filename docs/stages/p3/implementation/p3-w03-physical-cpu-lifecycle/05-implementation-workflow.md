# P3-W03 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing anything, the implementer verifies it has loaded the entry
README, the Coding Guidelines, and the routed documents, and performs
read-only discovery. Implementation proceeds only when:

- [P3-W01](../p3-w01-cpu-topology-inputs/README.md) types and
  `TopologyInputs` exist per its design (identity fields, dense logical
  ids, class vocabulary);
- [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)'s transition-request
  flow is agreed at contract level (which operations it calls, from where);
- [P3-W05](../p3-w05-smp-boot-synchronization/README.md)'s coordinator
  contract is agreed (it calls `admit_online`);
- the P2-W05 small-allocation contract and P0 diagnostics governance are
  available as reviewed deliverables.

Sibling-design coordination note: W03 sits between W02 and W05, whose
designs are being prepared in parallel on this branch. If a counterpart's
agreed contract cannot be located when implementation starts, that is a
coordination blocker to record — resolve the contract first; do not
improvise transition semantics to keep moving.

Stop and obtain direction when: an edge outside the table is requested by
any consumer (design conflict — resolve between designs); a consumer asks
for post-boot record mutation (out of scope; Reserved path); or an
implementation step appears to need a lock (design error in W03 — the
state machine is single-word CAS by contract).

## 2. Ordered implementation steps

### Step 1 — implement the state vocabulary and encoding

Target: [03-code-contracts-cpu-state-machine.md](03-code-contracts-cpu-state-machine.md)
§1–§2.

Work: implement `CpuLifecycleState` with fixed packed discriminants,
round-trip rejection of unknown encodings, and the build-time
initial-state mapping with exclusion notes.

Acceptance: encoding is total over the eight states and rejects
everything else; unreachable-at-P3 states are represented but edgeless.

Failure/blocker: any need to renumber encodings during implementation is
a contract change — stop and record, never renumber silently.

Evidence: implementation record
(`../p3-w03-physical-cpu-lifecycle-record.md`).

### Step 2 — transition table and operations

Target: state-machine file §3–§5.

Work: implement the five transition operations over the shared CAS
template, the refusal diagnostics, and event emission. The
`admit_online` contention path takes the fatal diagnostic route; it must
not return an error.

Suggested observation: host-side tests under the P0 gate.

Acceptance: every legal edge accepted exactly once; every illegal edge
refused naming the observed state; events emitted for accepted edges
only; emission failure cannot alter transition results.

Failure/blocker: if an operation cannot be expressed without a lock, the
state machine design is wrong — raise it; do not add synchronization
beyond single-word CAS.

Evidence: verification record
(`../../verification/p3-w03-physical-cpu-lifecycle-verification.md`).

### Step 3 — registry build

Target: [04-code-contracts-registry-and-admission.md](04-code-contracts-registry-and-admission.md)
§1–§2.

Work: implement `CpuRegistry::build` from `TopologyInputs` with the
initial-state mapping, immutable identity fields, dense-index table, and
boot-flag; wire the single build call at the global-initialization point
the W05 design designates.

Acceptance: record set equals the discovered inventory; build failure
publishes nothing; identity fields have no mutation path.

Failure/blocker: a topology shape the build cannot consume is a W01
contract conflict — record and stop.

Evidence: implementation record.

### Step 4 — admission, gate, snapshot, dump

Target: registry file §3–§6.

Work: implement `admit_online`, `Eligibility`, `OnlineSet`, and
`render_state_dump`; expose the coordinator-only path for admission.

Acceptance: gate matches the architecture §4 table exactly; snapshot is
allocation-free and consistent with a scan; dump renders every field
P3-V03 review needs.

Failure/blocker: a consumer needing a stale-tolerant cached online count
as *authority* is a design violation — refuse and point them at the
snapshot.

Evidence: implementation record.

### Step 5 — registry unit tests

Work: host-side tests: full transition-table sweep (legal/illegal);
admission exactly-once including the duplicate-detection path (expressed
via the test seam, e.g., two sequential admission attempts); gate matrix
over all states; snapshot consistency before/after transitions;
round-trip encodings; build mapping from every W01 class.

Acceptance: every invariant P3-V03 names is asserted by a test that
would fail on a plausible bug.

Failure/blocker: a race the test seam cannot express (true parallel CAS
contention) is covered by review plus the stress package
([P3-W12](../p3-w12-smp-stress-failure-tests/README.md)) — record the
boundary, do not fake concurrency in unit tests.

Evidence: verification record.

### Step 6 — QEMU evidence and closure review

Work: via the P0 QEMU entry path, capture at least one boot with the
SMP-ready state dump (all-attempted-online case) and one with a failed
start (the W02 absent-CPU input) showing the failed CPU outside the
online set; verify per-count behavior for counts reachable in the
environment; then run the closure review against
[06-validation-and-handoff.md](06-validation-and-handoff.md) and the
handoff checklist.

Acceptance: captures show no CPU usable before initialization, no double
online, failed CPUs excluded; repeated-matrix evidence is explicitly
deferred to [P3-W13](../p3-w13-qemu-smp-regression/README.md).

Failure/blocker: a dump showing a CPU Online without a corresponding
rendezvous event is a failed item — diagnose; do not adjust the table to
make the dump look right.

Evidence: verification record.

## 3. Evidence destinations

- Implementation record:
  `docs/stages/p3/implementation/p3-w03-physical-cpu-lifecycle-record.md`
  (created when implementation starts).
- Verification record:
  `docs/stages/p3/verification/p3-w03-physical-cpu-lifecycle-verification.md`
  (created when evidence exists).

Neither file is created by this design.
