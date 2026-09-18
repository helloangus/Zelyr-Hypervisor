# P4-W02 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, the implementer verifies the Coding Guidelines preflight and
checks the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry
review status for rows R08/R09 (P2 memory), R02/R03 (P1 EL2), R14 (P3
CPU-local), R18/R19/R20 (P0 toolchain/newtypes/logging). Per
[01 §2](01-scope-and-foundations.md), missing evidence blocks the matching
acceptance, not necessarily the start of host-testable work: steps may proceed
against the assumed interfaces (M1/M2 signatures), but any step whose
acceptance needs missing upstream evidence is recorded as blocked, never as
passed.

Stop and record instead of improvising when:

- the pinned architecture reference contradicts a descriptor/TLBI assumption
  in [03](03-code-contracts-stage2-core.md) — fix the design first (documented
  erratum/change note in the implementation record), never the test;
- QEMU behavior diverges from the architectural expectation (for example,
  accepting a descriptor field the spec reserves) — record a Specification
  Investigation item (W01 A7); do not adopt the QEMU behavior as semantics;
- a required P3-era primitive (spin lock, CPU-local access) does not exist —
  the dependency is upstream (M6); do not build a parallel primitive in W02;
- the code wants an ADR-level `MemoryObject`/`MemoryRegion` — that is
  P2-ACR-01, unresolved; use the stage-local `MappingGrant` shape or stop.

## 2. Ordered implementation steps

### Step 1 — vocabulary types and descriptor encoding

Target: `s2-vocab` types and `s2-table::S2Desc`.

Work: implement the value types and the descriptor accessor per
[03 §1–§2.1](03-code-contracts-stage2-core.md), verifying the bit layout
against the pinned architecture reference revision. Add the exhaustive
encoding unit tests in the same step.

**Acceptance:** every `S2Flags` variant encodes to spec-checked field values;
reserved bits never set; round trips preserve values.  
**Failure/blocker:** a spec mismatch is a design erratum — update the design
note and tests together, and record the reference revision used.

### Step 2 — table memory and walk mechanics

Target: `s2-table` write path and `walk_to_leaf`.

Work: implement the two `unsafe` primitives and the walker per
[03 §2.2–§2.3](03-code-contracts-stage2-core.md) with host-side tests over
simulated table buffers. Every `unsafe` site carries the four-point SAFETY
justification; add the entries to the unsafe inventory (W01 R20).

**Acceptance:** walk tests cover allocation, failure atomicity, and
boundaries; `unsafe` surface is exactly the two primitives.  
**Failure/blocker:** any test-only loosening of SAFETY scope is prohibited;
extend the test scaffold, not the justifications.

### Step 3 — VMID allocator and space lifecycle

Target: `s2-vmid` and `GuestAddressSpace::create/destroy`.

Work: implement the allocator (D4) and the lifecycle per
[03 §3.1, §3.8](03-code-contracts-stage2-core.md), including zeroed root,
error-path cleanup, and destroy sequencing.

**Acceptance:** create/destroy unit tests pass including exhaustion and
repeat-lifecycle accounting restoration.  
**Failure/blocker:** allocator-interface mismatch with the delivered P2 shape
is an M2 conflict — record and reconcile through W01, do not wrap a new
allocator.

### Step 4 — mapping operations

Target: `s2-space` map/unmap/protect/query.

Work: implement per [03 §3.2–§3.5](03-code-contracts-stage2-core.md) with the
two-pass validation discipline and ledger maintenance. All negative tests
(already-mapped, not-mapped, misaligned, overflow, zero-size) run host-side
in this step.

**Acceptance:** mutation suites pass; ledger and descriptor states agree
after every operation and after every error path.  
**Failure/blocker:** an ordering bug found here (stale permission, partial
rollback) is fixed in the design's sequence first (§6 of
[02](02-architecture-and-state.md)), then in code.

### Step 5 — invalidation backend and activation

Target: `s2-tlb` seam and `s2-space::activate/invalidate_*`.

Work: implement the current-path backend (D7) with the capability gate for
range invalidation, then activation per
[03 §3.6–§3.7](03-code-contracts-stage2-core.md). Hardware steps here need
the AArch64 target from the P0 baseline (M8); if the target is not yet
available, this step is blocked and recorded, and host-testable work
continues.

**Acceptance:** unit tests for operation encoding; on-target activation
compiles and is exercised later at step 7.  
**Failure/blocker:** a missing range-invalidation extension selects the
page-loop fallback automatically; record which path is active (W09 fact).

### Step 6 — telemetry and negative-scenario scaffolding

Target: event emission points and the P4-V02/V07/V08 scenario scaffolding
consumed by [P4-W05](../p4-w05-validation-guest/README.md) and
[P4-W08](../p4-w08-qemu-integration-regression/README.md).

Work: route the [02 §8](02-architecture-and-state.md) events through the
P0 logging/trace baseline; define the mapping-side negative scenarios
(unmapped access, permission change without re-entry) that the Guest
scenarios will drive.

**Acceptance:** events observable on the established baseline; scenario
definitions handed to W05/W08 via their design interfaces.  
**Failure/blocker:** absent baseline (M7) — degrade per M7 boundary and
record; never introduce ad-hoc prints.

### Step 7 — on-target lifecycle evidence

Target: QEMU-run Stage-2 evidence (P4-V02, and the W02-shareable part of
P4-V07/V08).

Work: exercise create→map→activate→query→protect→unmap→destroy on the
reference platform through the W04/W05 consumers; record results in the
verification record.

**Acceptance:** P4-V02 fully evidenced; P4-V07/V08 mapped/unmapped/permission
behavior demonstrated through Guest access (with W05/W06 scenarios).  
**Failure/blocker:** on-target failures are diagnosed via W06's fault
context; a failure is recorded as failed with diagnosis — never masked by
relaxing an ordering rule.

### Step 8 — closure review

Work: run the [05](05-validation-and-handoff.md) matrix and handoff
checklist; write the implementation record facts (decisions taken, unsafe
delta, capability/limitation notes for W09).

**Acceptance:** matrix reviewed; handoff complete; no completion claim
outside the verification record.

## 3. Evidence destinations

- Implementation facts and decisions taken:
  `../p4-w02-stage2-address-space-record.md` (created when work starts).
- Commands, environments, results, run/not-run:
  `../../verification/p4-w02-stage2-address-space-verification.md`.
- Unsafe inventory delta: the P0 unsafe inventory location (W01 R20), linked
  from the implementation record.
