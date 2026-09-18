# P7-W03 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W03 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none is claimed to have run. Requirement ids
map to the P7 task book §6.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W03-DV01 → P7-V05–V07 | prerequisite review | inspect W01 register rows P7-IN-03/04/06 and Step-1 record | every consumed contract cited with failure boundary; none blocked for W03 | assumed-contract basis explicit; not that P2/P3/P5 are implemented |
| W03-DV02 → P7-V06 | model/authority review | review modules against [architecture and state](02-architecture-and-state.md) §1/§5 and ADR-016 | one vCPU placement type; no VM-kind split; single validation path; authorization upstream | the model conforms to ADR-016 and the capability boundary; not runtime behavior |
| W03-DV03 → P7-V06 | unit/property tests | host: `CpuSet` ops, validation matrix, ledger concurrency, eligibility properties | closed error set fully covered; concurrency test shows exactly-one attach; eligibility agrees with validation on all generated cases | representation and validation correct as specified; not target scheduling behavior |
| W03-DV04 → P7-V05 | pinned-equivalence comparison | run the P4 static-baseline asset pre-activation (E2) and a `Pinned` vCPU under the activated scheduler; compare pCPU residency and progress | identical residency (p only) and declared progress characteristics within test tolerances; no other vCPU ever ran on p | static pinned equivalence; not affinity/shared behavior (DV06) |
| W03-DV05 → P7-V07 | invalid-control tests (target) | exercise empty set, unknown id, offline/failed pCPU, exclusivity conflict, duplicate config, reconfiguration attempt, missing authority through the real entry path | every case returns the exact typed error, emits `placement_rejected`, attaches nothing, leaves the vCPU non-runnable | non-silent rejection on target; not dynamic-reconfiguration semantics (Reserved) |
| W03-DV06 → P7-V06/V07 | placement-matrix + telemetry review | QEMU matrix over pinned, affinity, and mixed dedicated/shared configurations (with W05/W11 scenarios); verify `placement_applied`/`placement_rejected` render via W09 | no vCPU ever dispatched on an ineligible pCPU in any matrix cell; configuration state observable | placement holds under scheduling; counter/trace quality is W09 evidence |
| W03-DV07 → W03 closure | consumer consumability review | read the design as W05 (can I filter enqueue/pick?), W07 (is placement preserved across pause/resume?), W08 (can I re-check online?), W11 (are matrix cells defined?) | each consumer finds its seam named | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command/input, reviewer, environment, timestamp, and reason.
Host passes do not substitute for the QEMU rows; nothing here proves
P7-V02–V04 or P7-V08–V30.

## 2. Error, security, and observability model

**Errors.** Closed `PlacementError` set (see
[contracts](03-code-contracts-placement.md) §2.1 plus `NotAuthorized`).
Guarantee: every rejection is typed, trace-visible, and residue-free; the
vCPU stays `Offline` and non-runnable until a valid configuration lands.
A pCPU failing after configuration is a runtime condition, not an error:
eligibility simply turns false and the containment flow is W07/W08's.

**Security.** Configuration is a controlled action: the P5 capability
receipt gates the only entry point; receipts are recorded for audit and
never interpreted here. `PlacementSpec` is untrusted management input and
every field is validated. No Guest-reachable path constructs or mutates a
placement. The exclusivity rule is bidirectional and enforced at validation
under a lock, closing the double-claim race. Silent fallback — the classic
scheduler misconfiguration failure — is structurally excluded: validation
returns a complete placement or an error, never a repaired one.

**Observability.** Applied and rejected configurations both emit semantic
events; ledger snapshots answer "where may each vCPU run" for diagnostics
(P7-V21 input) and for W11/W12 matrix verification. A pinned vCPU whose
pCPU failed is observable as (configured, not eligible, undispatched) via
ledger + registry — no silent disappearance.

## 3. Handoff checklist

Before handing W03 to a reviewer, provide:

- the exact changed-file list (expected: placement modules and tests per
  the approved layout; no crate/workspace/ABI additions);
- W03-DV01…DV07 evidence paths and run status, including explicit not-run
  entries (expected not-run at design acceptance: target rows, matrix
  scenarios awaiting W05/W11);
- confirmation: no new `unsafe`, dependencies, public API, wire encoding;
  `CpuSet` capacity traceable to platform facts;
- confirmation: single validation path, single eligibility function,
  attach-only-at-`Offline`, bidirectional exclusivity all hold in code;
- the recorded default-placement rule and its rationale for reviewer
  sign-off; and
- open items: W05 (enqueue/pick filter + hint rule), W07 (preserve frozen
  placement; undispatchable pinned vCPU containment), W08 (online
  re-check on wake/idle), W09 (event rendering), W11 (matrix cells), W14
  (semantics into the P8 handoff) — without resolving their contracts.
