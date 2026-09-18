# P6-W02 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README documents and inspects the tracked tree. Implementation may start
only when the entry review finds:

- the P6-ENTRY conditions of the task book §2 evidenced for the P3, P2, and
  P0 inputs W02 consumes ([01 §1](01-scope-and-foundations.md));
- the W01 capability decision record with `ReadyForP6` or `PhysicalOnly`
  grade and its pinned specification revisions;
- an existing EL2 runtime with the P3 lifecycle/per-CPU/rendezvous/
  synchronization contracts actually present (not merely planned).

Stop and obtain direction instead of guessing when any of the following
occurs:

- a probe disagrees with the W01 expected identity → stop before any
  state-changing write; record per the W01 fold-back taxonomy
  ([01 §1.1](01-scope-and-foundations.md));
- the P3 local-initialization invocation point or per-CPU storage is
  absent or different from the assumed contract → stop; record against the
  P3 contract (Architecture Change Request if the P6 entry table is
  implicated);
- an architectural register behaves contrary to the pinned GIC
  specification revision (completion bit never sets, WAKER handshake
  stalls, SRE not verifiable) → stop the affected sequence; record a
  Specification Investigation; do not add retries, alternate encodings, or
  QEMU-conditional paths;
- the implementation appears to need dispatch, classification, SGI send,
  route-change policy, timer/maintenance PPI enablement, or Guest-visible
  state → scope violation; those belong to W03/W04/W05/W07–W08;
- a new crate boundary, dependency, or target change appears necessary →
  owned by their P0 packages; raise, do not improvise.

## 2. Ordered implementation steps

### Step 1 — entry review and record skeleton

Target: `../p6-w02-physical-gic-bring-up-record.md` (created in this step).

Work: record the entry-review result — which upstream records and evidence
were inspected, the W01 decision grade consumed, the pinned specification
revisions, and the QEMU environment declaration for later DV rows. State
explicitly that nothing is yet implemented.

**Acceptance:** the record names every assumed contract actually inspected,
with its evidence location.  
**Failure/blocker:** a missing entry condition is a recorded stage block
(task book §2), not a local workaround.

### Step 2 — register-access surface

Target: the `gic-regaccess` module
([03](03-code-contracts-register-access.md)).

Work: implement the frame handles, volatile operations, RMW with field
masks, bounded polling, and barrier helpers; write the register-class table
of [03](03-code-contracts-register-access.md) §4 from the pinned revision;
create the fake-frame test backend for host-side sequence tests.

**Acceptance:** the `unsafe` inventory contains exactly this module's
blocks, each with a SAFETY comment; mask-preservation and bounds tests pass
under the host gates.  
**Failure/blocker:** a needed arch barrier primitive does not exist in the
established tree → blocker against the P1 package; do not inline assembly
here.

### Step 3 — Phase A distributor sequence

Target: the `gic-distributor` module
([04](04-code-contracts-distributor.md)).

Work: implement `bring_up_distributor` with probe, coverage check,
quiesce, residual survey-and-clear, SPI baseline, determinate initial
routing, enable, posture verification, and the failure paths.

**Acceptance:** host-side sequence tests on the fake backend cover: normal
path; probe mismatch (no write occurred — assert via backend log);
coverage miss; quiesce/enable timeout; residual clearing; posture
read-back mismatch.  
**Failure/blocker:** a sequence step cannot be expressed without a policy
decision reserved elsewhere (e.g. non-default priorities) → stop at the
design boundary, do not tune.

### Step 4 — readiness ledger

Target: the `gic-readiness` module
([02](02-architecture-and-state.md) §2.4).

Work: implement the global and per-pCPU slots with write-once,
release/acquire publication and the query surface.

**Acceptance:** unit tests show write-once enforcement and cross-thread
visibility ordering (host-side threaded test where the host test baseline
permits).  
**Failure/blocker:** the P3 atomic-ordering contract differs from the
assumed acquire/release discipline → stop and reconcile with the P3
contract owner.

### Step 5 — Phase B local sequence and P3 integration

Target: the `gic-local` module
([05](05-code-contracts-local-gic.md)) plus the P3 local-init invocation
wiring.

Work: implement `bring_up_local_gic` per
[05](05-code-contracts-local-gic.md); wire the invocation from the P3
per-pCPU local-initialization point (the P3-owned hook), ensuring the boot
pCPU's Phase B runs after Phase A and secondaries after rendezvous +
DistributorReady.

**Acceptance:** sequence tests cover identity mismatch, wake timeout,
SRE verify failure, residual baseline, double-invocation rejection; the
invocation point is the P3 contract's, not a new W02 thread/task.  
**Failure/blocker:** integration requires changing P3-owned code beyond the
declared hook → stop; the hook change belongs to the P3 contract owner.

### Step 6 — telemetry wiring

Target: `gic-telemetry` events
([02](02-architecture-and-state.md) §6).

Work: register the five event kinds under the P0 namespace contract;
bounded residual payloads.

**Acceptance:** events compile under the telemetry gates; payloads carry no
platform names.  
**Failure/blocker:** P0 telemetry contract gap → fall back to the
established diagnostic log and record the deferral for W13.

### Step 7 — QEMU bring-up execution and evidence

Target: verification record
(`../../verification/p6-w02-physical-gic-bring-up-verification.md`).

Work: run the P6-V02/V03 scenarios of
[07](07-validation-and-handoff.md) §2 in the declared QEMU environment:
single-CPU boot to `DistributorReady` + boot-CPU `LocalReady`; multi-CPU
boot with every online pCPU publishing its own `LocalReady`; read-back
verification of the disabled/default initial state; residual report
observation. Record commands, output, environment, timestamps, and
run/not-run per row.

**Acceptance:** every DV row has a recorded status; QEMU rows state their
proof boundary explicitly (QEMU success does not prove real-hardware
correctness).  
**Failure/blocker:** a QEMU failure is evidence — record as failed with
diagnosis; do not loosen a sequence, mask a check, or add a
platform-conditional path to pass.

### Step 8 — closure review

Work: run the review matrix, confirm the handoff checklist, verify against
the plan's work sequence and task-book rows P6-V02/V03. Completion is
claimed only in the verification record, only for what was actually run.

## 3. Implementer error-handling rules

- Never index a GICR frame list by position; always select by affinity
  match ([05](05-code-contracts-local-gic.md) §1 checks).
- Every bounded wait has a named timeout outcome; no loop without a bound.
- No `unsafe` outside `gic-regaccess`; no new `static mut`; no interrupt
  enabled outside the W03 registration surface.
- All diagnostics are bounded (aggregate counts, sampled IDs).
