# P6-W01 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree (read-only
`git ls-files`; confirm which P1/P2/P3 designs and records actually exist at
implementation time). Because every upstream fact is an assumed contract, the
implementer must first locate the *actual* upstream deliverables:

- the P2 platform-fact types and their validation rules (per
  [p2-w02](../../../p2/plans/p2-w02-platform-discovery-normalization.md));
- the P1 CPU capability record shape (per
  [p1-w03](../../../p1/plans/p1-w03-aarch64-capability-inventory.md));
- the P3 possible-pCPU set source (per
  [p3-w01](../../../p3/plans/p3-w01-cpu-topology-inputs.md)).

Stop and obtain direction instead of guessing when any of the following
occurs:

- an upstream fact type does not exist or differs from the assumed shape →
  record a Platform Investigation item against the upstream plan; do not
  define a local substitute type in W01;
- upstream facts contradict each other or the architecture → classify
  `Contradictory` per [01](01-scope-and-foundations.md) §4 and stop the
  affected decision; label `Architecture Change Request` when the conflict is
  between governing documents;
- the GIC specification revision cannot be resolved or two candidate
  revisions disagree on an identity-register field this design consumes →
  record a Specification Investigation item; do not pick a revision silently;
- implementation appears to require MMIO/system-register code, a crate
  boundary change, or a new dependency → scope violation; those belong to
  W02+ or their owning packages.

## 2. Ordered implementation steps

### Step 1 — pin the authoritative GIC specification revision

Target: implementation record
(`../p6-w01-gic-capability-discovery-record.md`, created in this step).

Work: resolve the authoritative AArch64 + GICv3 specification revisions the
project will implement against (architecture reference manual revision and
GIC architecture specification revision, per the ADR baseline §20 rule that
revisions are locked at implementation). Record both, with where they were
resolved from and the date. All identity-register field interpretation in
this package cites these revisions.

Suggested observation: the specification index on the official Arm
documentation site; the exact lookup method is not normative.

**Acceptance:** the record names one architecture revision and one GIC
specification revision with provenance.  
**Failure/blocker:** unresolvable revision → recorded blocker (§1); no
substitution from a secondary source.

### Step 2 — implement the capability model types

Target: the `gic-capability-model` logical module
([02](02-architecture-and-state.md) §1), placed in the crate layer the
established tree provides for platform-independent interrupt knowledge.

Work: implement the type inventory of
[03](03-code-contracts-capability-model.md) §1–§2, reusing the P0 semantic
newtype baseline for addresses and IDs. No I/O, no `unsafe`, no allocation
beyond the boot-phase contract.

**Acceptance:** types compile under the pinned toolchain gates; every field
is typed (no naked integers for IDs/addresses); the aggregate invariant
helper exists for tests.  
**Failure/blocker:** a required upstream newtype does not exist → blocker
against its owning package (§1), not a local `u64`.

### Step 3 — implement reconciliation

Target: the `gic-capability-reconcile` module.

Work: implement `reconcile()` and `derive_expected_identity()` per
[03](03-code-contracts-capability-model.md) §3–§4, including the full
per-row classification table exercised by host-side fixture tests: usable
baseline; GICv2 input; missing Redistributor frame; malformed SPI range;
contradictory reservation overlap; virtualization absent. Fixtures are
constructed from in-memory fact values — no DTB, no QEMU, no hardware.

**Acceptance:** all fixture classes produce the exact verdict set and grade
stated in the design; the function is pure (same input → same output).  
**Failure/blocker:** a fixture cannot be expressed without inventing an
upstream field → blocker against the upstream plan (§1).

### Step 4 — wire report emission and telemetry events

Target: the `gic-capability-report` module; event registration under the P0
trace-event namespace contract.

Work: implement `emit_capability_report()` per
[03](03-code-contracts-capability-model.md) §6; register the three event
kinds of [02](02-architecture-and-state.md) §7 with payload schemas that
carry no board names.

**Acceptance:** events compile under the telemetry gates; the report renders
the decision without editorializing findings.  
**Failure/blocker:** the P0 telemetry contract does not yet define the
needed event mechanism → record the gap; emit via the established diagnostic
log path only, and note the deferral for W13.

### Step 5 — publish the decision artifact path and record skeleton

Target: the W01 record artifact
(`../p6-w01-gic-capability-discovery-record.md`).

Work: record the factual implementation status — modules added, fixture
classes run (as unit tests, not runtime evidence), deviations, and the
open-investigation list. Do not claim P6-V01; the runtime decision artifact
exists only after boot-time execution, which requires W02's chain.

**Acceptance:** the record states what exists, what does not, and the exact
remaining path to P6-V01 evidence.  
**Failure/blocker:** none; honesty failures fail review.

### Step 6 — closure review

Work: run the review matrix in
[validation and handoff](05-validation-and-handoff.md), confirm the handoff
checklist, and verify the package against the plan's work sequence and the
task-book entry conditions. Completion is claimed only in the verification
record, with evidence, and only for what was actually run.

## 3. Error handling model for implementers

- Every consumed platform field is re-validated (`Malformed` before
  `Unsupported`/`Usable`); never index, slice, or unwrap platform data.
- `Contradictory` findings name both sources; never resolve by precedence.
- The function returns a decision even when every input is defective —
  "cannot decide" is `Rejected` with findings, not a panic and not an
  `Option::None` escape.
- Diagnostics are bounded: one finding event per input, no unbounded loops.

## 4. Validation matrix

See [05-validation-and-handoff.md](05-validation-and-handoff.md) §2 for the
authoritative matrix (W01-DV01…W01-DV07). Implementation steps 2–4 map to
W01-DV02/DV03; step 5 maps to W01-DV06; step 6 runs DV01–DV07 and records
run/not-run per row.
