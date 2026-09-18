# P3-W04 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing anything, the implementer verifies it has loaded the entry
README, the Coding Guidelines, and the routed documents, and performs
read-only discovery. Implementation proceeds only when:

- [P3-W01](../p3-w01-cpu-topology-inputs/README.md) types and
  `TopologyInputs` exist (density invariant, bound);
- [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)'s gate exists with
  the install-time eligibility semantics;
- [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)'s entry tail has an
  agreed call point for installation and the stack-transfer contract;
- [P3-W05](../p3-w05-smp-boot-synchronization/README.md)'s readiness-signal
  contract is agreed;
- the P2 page-allocation contract and the P1 address-space coverage
  statements are available as reviewed deliverables.

Stop and obtain direction when: the P1 design reserves `TPIDR_EL2`
(Architecture Change Request with the P1 owner — do not pick another
register silently); allocated memory is outside the P1-mapped regions
(same route); a consumer asks W04 for slot contents/protocols (boundary
violation — that is the consumer's design); or the register mechanism
cannot be tested without hardware (raise the seam design; do not embed
QEMU in unit tests).

## 2. Ordered implementation steps

### Step 1 — area layout and allocation

Target: [03-code-contracts-percpu-area.md](03-code-contracts-percpu-area.md)
§1–§4, §6.

Work: implement the layout (header, slots, reserved region with recorded
offsets and alignment), `allocate_all` against the P2 page allocator,
fill patterns, header validation, and the lookup table.

Acceptance: alignment rules hold; offsets recorded; allocation failure
and validation failure both publish nothing; the table is dense.

Failure/blocker: an alignment or sizing need that exceeds the recorded
constants is a design change — stop and record, do not grow the layout
locally.

Evidence: implementation record
(`../p3-w04-per-cpu-runtime-record.md`).

### Step 2 — layout and allocation tests

Work: host-side tests with a fake page source: header round-trip
validation, fill-pattern integrity, dense table construction,
allocation-failure path, bounds sanity (stack within its allocation,
areas distinct and non-overlapping).

Acceptance: every §6 error path asserted; layout invariants hold for
n = 1 and n = 8.

Evidence: verification record
(`../../verification/p3-w04-per-cpu-runtime-verification.md`).

### Step 3 — runtime stacks and transfer

Target: area file §2.

Work: fix `RUNTIME_STACK_SIZE` in the implementation record with
rationale; implement stack allocation and bounds recording; implement the
secondary's switch from the W02 provisional stack at the install point;
confirm the quarantine path for W02's scaffolding.

Acceptance: no CPU_ON-time stack sharing; transfer is one-way; boot CPU
records its P1 boot stack in its header without switching.

Failure/blocker: an address-space constraint (stack not reachable under
the P1 map) is an Architecture Change Request route — stop and record.

Evidence: implementation record.

### Step 4 — CPU-local access mechanism

Target: [04-code-contracts-cpu-local-access.md](04-code-contracts-cpu-local-access.md)
§1–§2, §4–§5.

Work: implement the register convention (`install_current`, `current`),
the debug validation hook, and `area_of`. The register read/write is the
package's `unsafe` boundary with `SAFETY` comments referencing the
convention and the install ordering.

Acceptance: `current()` has no fallback locality source; the unsafe
boundary is minimal and audited; debug validation catches an unloaded
register in test builds.

Failure/blocker: any need for a second locality source is a design
violation of the architecture file §6 — raise it.

Evidence: implementation record; verification record for tests.

### Step 5 — installation sequence

Target: access file §3.

Work: implement `install_local_state` with the exact ordering: gate
check → Installing CAS → header validation → register load → stack
switch (secondary) → isolation diagnostic → Installed release-store →
W05 signal. Wire the call into the W02 entry tail and the boot CPU's
local-init point.

Acceptance: order matches architecture §5; exactly-once enforced by the
CAS; a refused gate aborts without signaling.

Failure/blocker: if W02/W05 call points cannot be agreed (sibling design
not landed), record the coordination blocker; do not invent interim
ordering.

Evidence: implementation record.

### Step 6 — isolation evidence and closure review

Work: via the P0 QEMU entry path, capture per declared count (1, 2, 4, 8
as reachable): per-CPU isolation diagnostics showing correct identity,
distinct area addresses, and distinct stack addresses on every online
CPU; verify the boot CPU appears in the same invariants. Run host-side
boundary checks (no global current-CPU symbol; no self-lookup through
the table in kernel code). Then run the closure review against
[06-validation-and-handoff.md](06-validation-and-handoff.md) and the
handoff checklist.

Acceptance: per-count captures (or not-run entries with reason) show
distinctness and identity correctness; repeated/stress evidence is
explicitly deferred to W12/W13.

Failure/blocker: two CPUs reporting the same area or stack address is a
failed item — diagnose; do not mask with per-CPU reindexing tricks.

Evidence: verification record.

## 3. Evidence destinations

- Implementation record:
  `docs/stages/p3/implementation/p3-w04-per-cpu-runtime-record.md`
  (created when implementation starts; includes `RUNTIME_STACK_SIZE`,
  layout offsets, and the register-convention note).
- Verification record:
  `docs/stages/p3/verification/p3-w04-per-cpu-runtime-verification.md`
  (created when evidence exists).

Neither file is created by this design.
