# P3-W01 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W01 detailed design](README.md).

## 1. Validation matrix

Each item is recorded as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason in the verification
record. Planned items are not evidence. QEMU items appear only where W01's
scope produces them; the repeated matrix is P3-W13's.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W01-DV01 | Typed, validated hardware identity (→ P3-V01) | identity contract review + unit tests | review [03](03-code-contracts-cpu-identity.md) against code; run identity tests | construction authority enforced; reserved-bit rejection; ordering/equality per contract; no index semantics | identity typing is sound; not that any real CPU matches it |
| W01-DV02 | Stable logical mapping (→ P3-V01) | mapping-stability tests | run intake over fixture fact sets in multiple shuffled orders; compare full outputs | identical logical↔hardware mapping regardless of input order; dense ids; deterministic boot designation | rule determinism over fixture inputs; not platform discovery correctness (P2's) |
| W01-DV03 | Classification and exclusion (→ P3-V01) | classification tests + review | run classification over fixtures covering Present, Possible, every UnavailableReason, duplicates, bound exceed | every class and reason exercised; Unavailable/Possible CPUs absent from candidate set; intake all-or-nothing | intake rejects malformed input; not that firmware behaves |
| W01-DV04 | Boot-CPU designation (→ P3-V01) | boot-match tests + code review | fixture tests for BootCpuUnknown / BootCpuNotPresent; review check-before-bound ordering | unknown or non-Present boot identity fails with the named error before any candidate list exists | designation integrity; not EL2 entry correctness (P1's) |
| W01-DV05 | Enumeration diagnostic quality (→ P3-V01/P3-V11 interface) | diagnostic review | inspect emitted lines for logical id, identity, class, reason, boot flag, counts, start capability | every P3-V01 review field is present and CPU-attributed | observability of the topology decision; not lifecycle events (P3-W03/W11) |
| W01-DV06 | Cross-count mapping evidence (→ P3-V01) | QEMU enumeration capture | one boot per declared count (1/2/4/8) via the P0 QEMU entry path; compare enumeration output to expected mapping | stable mapping and boot-CPU identification per captured configuration; unavailable CPUs excluded | per-count correctness at capture time; not repeated-boot stability, not hardware platforms, not hotplug |
| W01-DV07 | Contract, limits, and consumers recorded (→ W01 closure) | closure review | verify implementation record against this design; verify handoff checklist below | no deviation unrecorded; consumers named; no completion claim | handoff readiness; not W02–W05 correctness |

Writing the types without W01-DV02/DV03/DV04 evidence does not satisfy the
package; capturing QEMU output does not prove hardware-platform behavior
(P3-V01's stated boundary) and never substitutes for the P3-W13 matrix.

## 2. Error, security, and observability model

- **Error model.** Two failure families: (a) intake validation
  (`TopologyError`) — fail-closed before any bring-up work, fatal per the
  P0 panic policy because a machine whose CPU model is unknown cannot run
  safely; (b) per-CPU classification reasons (`UnavailableReason`) —
  non-fatal exclusions that must remain individually diagnosable. Nothing
  in W01 retries or repairs.
- **Security.** P2-delivered platform facts are untrusted input at the
  intake boundary: malformed, duplicated, over-bound, or contradictory
  declarations are rejected, not coerced. Identity types are not
  authorization tokens. No guest-reachable surface exists. The MPIDR read
  is the package's only `unsafe` and is confined to the architecture side
  with a `SAFETY` obligation referencing the P1 entry contract.
- **Observability.** The enumeration diagnostic is W01's entire observable
  output plus its error paths; every line is CPU-attributed. Lifecycle,
  contention, and event telemetry belong to P3-W03/P3-W11; W01 emits no
  runtime events because it has no runtime.

## 3. Handoff checklist

Before handing W01 to a reviewer:

- the exact changed-file list, with crate/module placement decisions
  traced to the approved workspace design;
- W01-DV01–DV07 evidence paths with run status, including explicit not-run
  entries (e.g., counts not reachable in the environment; matrix deferred
  to P3-W13);
- confirmation that no CPU-start call, lifecycle transition, per-CPU
  storage, boot barrier, or guest-facing surface was added (entry README
  §Explicitly excluded interfaces);
- confirmation that new `unsafe` is limited to the arch-side MPIDR read
  and is inventoried per P0 unsafe governance;
- confirmation that no platform/board name or QEMU-specific constant
  appears in generic topology code;
- open items for consumers: W02 (candidate list and start-capability facts
  ready), W03 (class vocabulary handed over; runtime states undefined
  here), W04 (density invariant), W05 (boot designation and bound), W10
  (anti-assumption invariants as audit criteria) — without resolving
  their contracts;
- any recorded blocker (P2 handoff shape, workspace placement, bound
  raise requests) with its decision owner.
