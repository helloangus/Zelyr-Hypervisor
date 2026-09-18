# P5-W06 Implementation Workflow and Validation

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the actual tree and records:

- the W03, W04, and W05 detailed designs and their implementation records
  exist and are compatible with the assumed contracts of
  [01 §2](01-dispatch-pipeline-contract.md); the W02 design and, where it
  exists, the factual ABI artifact define the request/result vocabulary;
- W01's entry reconciliation
  (`../p5-w01-entry-contract-reconciliation/README.md`) is available and
  records no unresolved block against W06's inputs;
- the P4 handoff evidence named by
  [P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md) exists for Guest
  entry/exit, exception classification, and fault isolation;
- a crate/module tree actually exists in the implemented tree (delivered by
  the P0–P4 designs), so the dispatch module has a physical home.

Stop and obtain direction instead of guessing when: a prerequisite design or
record is absent or contradicts an assumed contract (record the block or
`Architecture Change Request`; do not adapt silently); the W02 supported-call
set contains no operation requiring reference + authority + Guest data
(the minimal closed loop cannot close — `Architecture Change Request`);
a boundary cannot be honored inside VM-exit context (contract defect — stop);
or implementing the chain appears to require a management ABI, VM lifecycle,
scheduler, or P6+ mechanism (scope violation — stop).

## 2. Ordered implementation steps

### Step 1 — reconcile prerequisites and record the integration baseline

Target: implementation record
(`../p5-w06-dispatch-permission-containment-record.md`, created in this
step).

Work: verify each row of the [01 §2](01-dispatch-pipeline-contract.md)
assumed-contract table against the delivered designs and records; record the
delivered logical operation names that W06 adapts to; confirm the P4
exception evidence; fix the physical placement of the dispatch module within
the established tree.

**Acceptance:** the record lists every assumed contract as confirmed,
adapted-with-reference, or blocked; no assumed contract is left implicit.  
**Failure/blocker:** a blocked row stops Steps 2–5 for the affected boundary
and is recorded per §1; the remaining independent work may proceed only if
the dependency map allows.

### Step 2 — implement the pipeline skeleton

Target: the dispatch integration module (placement from Step 1).

Work: implement `HypercallDispatchContext`, `GuestCallOutcome`, and
`dispatch_hypercall` per [01 §4–§5](01-dispatch-pipeline-contract.md), with
the S0–S8 order and the short-circuit rule; wire the S8 result encoding to
the W02 path; assign result categories per
[02 §4](02-containment-and-error-classification.md) as a pure mapping.

**Acceptance:** the pipeline compiles against the delivered boundary
contracts; every outcome class of
[02 §2](02-containment-and-error-classification.md) is constructible in
host-side tests; no stage after a failure executes (asserted by focused
unit tests).  
**Failure/blocker:** a mismatch with a delivered contract is a Step 1
blocker, not a local workaround.

### Step 3 — integrate the minimal permitted operation

Target: the object-attributes query service function
([01 §5.2](01-dispatch-pipeline-contract.md)).

Work: implement S7 for the query operation using only the W04 reference and
W03 buffer contracts; ensure read-only semantics (no persistent mutation);
bound the payload by the W02-declared maximum; encode success through S8.

**Acceptance:** a host-side unit path exercises decode → resolve → type →
authority → arguments → state → execute → result with the delivered
contracts; no state mutation is observable besides the result payload.  
**Failure/blocker:** if the W02 set lacks a suitable call, stop per §1
(`Architecture Change Request`); do not invent a call number.

### Step 4 — wire the denial and containment paths

Target: the same module.

Work: implement every row of the
[02 §2](02-containment-and-error-classification.md) mapping; verify the
denial paths write nothing to Guest memory; verify `ResourceExhausted` is
clean (no half-allocation); implement the invariant path boundary of
[02 §3](02-containment-and-error-classification.md) so it is reachable only
from internal-fault injection in host tests.

**Acceptance:** host-side negative tests for each class produce the mapped
class; Guest-buffer writes are absent on S1–S4 denials (checked by test
observation); the invariant path is unreachable from any Guest-controlled
input fuzzed or enumerated in tests.  
**Failure/blocker:** a reachable invariant path via Guest input is a
security defect — fix the pipeline, never by weakening the path's
classification.

### Step 5 — containment and classification review

Target: review evidence in the verification record.

Work: run the plan step-5 review: (a) Guest-caused input cannot panic the
Hypervisor — walk each class of Guest input (call number, version, flags,
lengths, addresses, references, states) through the mapping; (b) invariant
handling is separately diagnosable — confirm the P0-W14-classified path with
its own record; (c) authorization ordering — confirm no state interaction
before S4 by code reading; (d) concurrency context — confirm no blocking,
unbounded allocation, or dispatch-owned locks, and no pCPU-keyed behavior.

**Acceptance:** the review is recorded with findings per item and any
defects fixed before validation runs.  
**Failure/blocker:** a finding that requires reordering stages or changing
ownership is a design conflict — stop and record; do not implement a silent
reorder.

### Step 6 — validation runs

Target: verification record
(`../../verification/p5-w06-dispatch-permission-containment-verification.md`).

Work: execute the matrix in §3 that is in scope for W06 (DV01–DV07);
coordinate integrated QEMU evidence with the W07 Guest scenarios and record
W06's side (dispatch outcomes, categories) for the same runs; record every
run with command, environment, result, and explicit not-run entries.

**Acceptance:** each executed row shows its passing condition met with
evidence; each deferred row shows not-run with reason.  
**Failure/blocker:** a failed row is recorded as failed with diagnosis;
passing is never manufactured by narrowing the scenario.

### Step 7 — records and handoff

Target: implementation record; handoff section of the verification record.

Work: complete the record (changed files, new `unsafe` (expected: none
beyond what delivered boundary implementations already carry; any new
`unsafe` in W06-owned code is a reporting event per the Coding Guidelines),
ABI/public-API deltas (expected: none — all W06 interfaces internal),
dependencies, TODO/FIXME ownership); confirm the downstream handoffs of the
README are satisfied and hand stable outcomes to W07–W10.

**Acceptance:** the README's handoff bullets each point to a delivered
artifact; the closure review of the plan can locate them.  
**Failure/blocker:** a missing handoff artifact blocks closure review, not
the artifact's consumer.

## 3. Validation matrix

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. QEMU rows
prove behavior in the declared QEMU environment only; they do not prove
AArch64 hardware semantics, other hypercalls, or a management interface.

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W06-DV01 → P5-V09 | prerequisite and pipeline review | Step 1 + Step 2 review against [01](01-dispatch-pipeline-contract.md) | assumed contracts confirmed; S0–S8 order present; categories total | composition is designed as required; not that the chain runs |
| W06-DV02 → P5-V09 | authority-ordering evidence | host-side unit path: authorized caller succeeds; absent/right-insufficient/revoked authority denies; identity shortcuts impossible (code review + W05 evidence) | denial classes `NoAuthority`/`Revoked` distinguishable; no state touched before S4 | caller-associated enforcement through the integrated path; not W05's internal model |
| W06-DV03 → P5-V09 | integrated valid-call evidence | with W07's QEMU scenario: Validation Guest issues the query call for an authorized object and reads the result | structured result delivered; Guest and dispatch-side records agree | the full chain works once, end to end; not all future hypercalls |
| W06-DV04 → P5-V10 | denial-class evidence | host-side negatives for every row of [02 §2](02-containment-and-error-classification.md) plus W07 Guest scenarios for the same classes | every class returns its mapped outcome; Guest memory untouched on S1–S4 denials | containment per class in the declared environments; not abuse-policy behavior |
| W06-DV05 → P5-V10 | no-panic containment review | Step 5(a) walkthrough + fuzz-smoke handoff to W08 on the pipeline seams | no Guest-controlled value reaches the invariant path | containment property on the exercised input space; not absence of all bugs |
| W06-DV06 → P5-V10 | invariant-path separation evidence | host-side injected internal faults | invariant path classifies per P0-W14 and never emits a Guest outcome | diagnosability of real internal failures; not hardware fault behavior |
| W06-DV07 → P5-V14 (support) | concurrency-context review | Step 5(d) + W08 two-pCPU scenarios through the pipeline | no blocking/lock/pCPU assumptions; W08 race scenarios complete determinately | exit-context discipline; not final scalability or lock strategy |
| W06-DV08 → W06 closure | handoff and record review | Step 7 checklist; downstream consumers read their artifacts | W07/W08/W09/W10 handoffs each satisfied by a located artifact | handoff readiness; not downstream completion |

The permanent invariants this package serves (task book §6): INV-P5-01/02
(no Host pointer; checked Guest addresses), INV-P5-03 (invalid/stale/
wrong-type rejection), INV-P5-04/05/06 (rights, cross-VM, revoke
enforcement), INV-P5-07 (malformed-request containment), INV-P5-10 (no
VM-ID-derived authority). Their Guest-side proof lives in W07's matrix;
W06's proof obligations are DV02–DV06.

## 4. Error, security, and observability model

Summary of what is normative here versus designed in
[02](02-containment-and-error-classification.md): error classes and the
total mapping are design, not implementation discretion; the fatal path is
P0-W14-classified and separate; every call carries a result category for
W09's counters. New `unsafe` is not expected in W06-owned code; if the
implementation introduces any, it is reported with SAFETY documentation per
the Coding Guidelines and enters the P0 unsafe inventory via the P0-W10
governance ([plan](../../../p0/plans/p0-w10-unsafe-rust-governance.md)).
No ABI or public API is added; all W06 interfaces are internal and
P5-experimental.

## 5. Handoff checklist

Before handing W06 to a reviewer, provide:

- the exact changed-file list, with the dispatch module's placement and the
  reason it follows the established tree;
- evidence paths and run status for W06-DV01–DV08, including explicit
  not-run entries;
- the resolved assumed-contract table from Step 1 (confirmed / adapted /
  blocked);
- confirmation that no new ABI values, error numbers, call numbers, public
  APIs, management or machine-ABI surface, or P6+ mechanisms were added;
- confirmation that Guest-caused inputs cannot reach the invariant path, and
  that the fatal path is P0-W14-classified and separately recorded;
- the scenario-outcome table of
  [02 §2](02-containment-and-error-classification.md) as delivered to W07,
  the seam/oracle notes delivered to W08, and the category taxonomy
  delivered to W09;
- open items: any blocked assumed contract, any
  `Architecture Change Request` / `ADR Required` record, and any Reserved
  behavior deferred by this design — without resolving them here.
