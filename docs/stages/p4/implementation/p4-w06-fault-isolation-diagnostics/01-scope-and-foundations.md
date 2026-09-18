# P4-W06 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W06 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-E02, P4-F01–F04, P4-G01–G04 W06 share)

- Exit categorization for every P4 exit class with retained raw syndrome and
  decoded detail (access type, fault-status family, exception class).
- Translation-fault diagnosis: faulting-IPA reconstruction, mapping-state
  cross-check, expected/unexpected verdict.
- Permission-fault diagnosis: write vs execute access distinction with
  mapping-flag agreement.
- Guest-versus-Hypervisor fault separation: structural fault-domain model,
  containment ordering, escalation split per the P0 failure classification.
- Faulting-address inspection: reported IPA vs W02 `query` snapshot, with an
  explicit agreement/disagreement verdict.
- The IS-series isolation expectation matrix covering unmapped gap,
  Guest-RAM boundary, selected Hypervisor-owned range, read-only write, and
  execute-permission negatives, each with a determinate expected
  `ExitDiagnostic` and post-stop EL2-liveness expectation.
- Bounded human-readable diagnostic report and a ledger-derived mapping-state
  dump available after stop.

### Reserved (must not be precluded; not implemented in P4)

- A hardware page-table walk for diagnostics (re-entry: a future W02-owned
  seam; the ledger dump keeps the call sites ready for it).
- Symbolized or interactive debug tooling (re-entry: a later diagnostics
  design; the fixed-format report keeps output parseable until then).
- Host-fault diagnosis beyond the P1 crash path (re-entry: P1-W07 contract
  evolution).
- Cross-pCPU fault correlation and per-pCPU diagnostic aggregation (re-entry:
  first multi-pCPU Guest design; correlation fields already carry the vCPU
  identity slot).
- Fault-injection tooling beyond the W05 triggers and host-side frame
  synthesis in unit tests.

### Out of Scope

The final `ExitReason` API and management error ABI (P5 and the Arch exit
contract), vGIC/IRQ/timer fault classes (P6), scheduler-visible fault policy
(P7), device/DMA/IOMMU isolation, a persistent telemetry transport or metrics
backend (P0-W12/W13 own the baseline; W07 consumes), Guest-side
self-diagnosis (W05's markers are data, not diagnosis), security
certification, and any policy for non-P4 Guest error handling.

## 2. Assumed upstream contracts and failure boundaries

Per the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry
review. If an upstream delivers differently than assumed, the assumption
becomes a recorded conflict (`Blocked prerequisite`, `Architecture Change
Request`, or `ADR Required` per W01 §4) and the affected W06 step stops; W06
never patches around an upstream change silently.

| ID | Assumed contract | Source (plan/design path) | Relied-on behavior | Failure boundary if delivered differently |
|---|---|---|---|---|
| M1 | Typed addresses and the error/panic classification split (Guest-caused recoverable vs invariant fatal) exist | [P0-W15](../../../p0/plans/p0-w15-address-identifier-type-safety.md), [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md) plans; W01 R19 | `GuestPhysAddr` values in diagnostics; the two failure classes are type-distinguishable | absent → W06 blocks (Coding-Guidelines shape, not a free choice) |
| M2 | W04 `GuestExitFrame` (gp, sp_el1, guest_pc, guest_pstate, esr, far, hpfar), captured unconditionally by the exit stub before any Rust runs; `classify_minimal`/`decide` unchanged; Stopped retains the last frame | [P4-W04](../p4-w04-vcpu-entry-exit/03-code-contracts-world-switch.md) §2, §4; [04-code-contracts-vcpu-run.md](../p4-w04-vcpu-entry-exit/04-code-contracts-vcpu-run.md) | W06 reads the frame verbatim; the stop decision precedes diagnosis; post-stop frame readable | frame field changes → joint design note (both designs); W06 never re-captures or re-interprets registers independently |
| M3 | W05 scenario table (`VG-T<n>`) with `VG-<id>:BEGIN/OK/FAULT:<KIND>/FAIL:<REASON>` markers, single marked trigger per fault scenario, no console output between marker and trigger | [P4-W05](../p4-w05-validation-guest/02-guest-architecture-and-scenarios.md) §4–§5 | last marker identifies the fault site; IS-series expectations key to VG ids and trigger kinds | marker/table changes follow W05 §6 joint-change rule; a divergence is a recorded conflict, not a re-labeled expectation |
| M4 | W02 `GuestAddressSpace::query(ipa) -> QueryResult` snapshot (`Unmapped | Mapped { flags, frame_base }`) under the space lock; ledger is the authoritative mapping state; protect uses two-step invalidate-then-set ordering | [P4-W02](../p4-w02-stage2-address-space/03-code-contracts-stage2-core.md) §3.4–§3.5; [02-architecture-and-state.md](../p4-w02-stage2-address-space/02-architecture-and-state.md) §6 | cross-check reads mapping state only through `query`; agreement expectations are well-defined | if `QueryResult` shape changes → joint design note; W06 never walks descriptors itself |
| M5 | Logging/trace baseline and event namespace (P0-W12/W13) exist with levels, structured events, and release-trim rules; build/version identity attached to diagnostics | P0-W12/W13 plans; W01 R20/A8 | `diag.*` events and the report route through it with build identity | absent → degrade to the P0 logging fallback; never ad-hoc prints |
| M6 | Stable EL2 fatal path (P1-W07 crash diagnostics, non-recursive) exists for host-context faults | P1-W05/W07 plans; W01 R02; W04 M3 | faults during EL2 execution terminate in the fatal path, preserving the Guest/Hypervisor domain split | absent → the domain model has no host side; W06 stops at Guest-domain evidence and records the blocker |

Entry-order boundary: decode/match/format/dump logic with host-side unit tests
can proceed against assumed signatures (M1–M4 shapes), but P4-V06–P4-V09-class
on-target evidence requires M2–M6 delivered; the
[workflow](05-implementation-workflow.md) marks where each boundary applies.

## 3. Authority analysis for contested areas

- **Classification authority vs W04.** W04's plan-level contract names it the
  owner of "minimal" classification and explicitly defers "full exit
  classification and isolation diagnostics" to W06. The seam is therefore:
  W04 assigns the P4 class and the action; W06 owns everything more detailed.
  No contract conflict exists; the `ExitClass` enum remains W04-owned and W06
  reuses it as a field, adding detail around it. This design records the
  boundary rather than moving it.
- **P2-ACR-01 (ADR Required, unresolved).** W06 consumes W02's ledger-derived
  `QueryResult` and asserts nothing about ADR-level `MemoryObject`/
  `MemoryRegion` semantics. The stage-local `MappingGrant`/`QueryResult`
  shapes remain W02/W03-owned; P2-ACR-01 stays visible and unresolved and is
  carried to [P4-W09](../p4-w09-closeout-p5-handoff/README.md) in the
  unresolved-conflict list. W06's decisions create no new pressure on it.
- **Hypervisor-owned-range probes without new Guest mechanisms.** P4-V07
  requires a "selected Hypervisor-owned-range" negative. In P4's identity
  mapping, any IPA outside the W02-granted ranges is unmapped, so a
  Hypervisor-owned-range access produces the same fault class as a gap probe;
  the isolation property proven is "not mapped, therefore blocked, therefore
  diagnosable." W06 therefore defines probe *classes* with class-membership
  assertions (which P2-sourced range the probe lies in) rather than new trap
  mechanisms. Probe address carriage reuses the W03 boot-info channel; if the
  delivered channel cannot carry the three probe configurations, that is a
  joint W03/W05/W06 change under their recorded rules (open item O1), never a
  private side channel.
- **QEMU syndrome behavior vs AArch64 semantics.** Decode tables and IPA
  composition follow the pinned architecture revision; QEMU-observed
  divergences are recorded as Specification Investigation items (W01 A7) and
  never encoded as Core semantics. The VG-011 routing expectation is
  W05-owned; W06's matcher consumes whatever class the delivered W04 routing
  produces, and a mismatch is a joint W04/W05/W06 review item, not a silent
  re-label.
- **Requirement-wording provenance.** The fine-grained E/F/G wording of the
  superseded root source task book is not tracked (W01 A9 pattern). This
  design fixes requirement meaning from the tracked task book §5 rows and the
  W06 plan scope; the implementation record cites this basis. This is a
  documentation-provenance finding, not an architecture conflict.

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | Two-layer classification: W04's `classify_minimal`/`decide` stays the sole action authority; W06 adds a purely descriptive `ExitDiagnostic` produced from the same frame | separation of mechanism (action) and diagnosis (report); no behavior depends on diagnostic richness; avoids two action policies | W04 plan/design scope; W06 plan scope (P4-E02) |
| D2 | Diagnosis is a pure, total, allocation-free function of frame data; unknown syndrome values fail closed to `Unclassified` with the raw ESR retained; `Unclassified` never re-enters and never panics | untrusted-input discipline; unknown conditions stay diagnosable (plan item 5); fail-closed beats guesswork | ADR-007; ADR §19; W01 A2 |
| D3 | Fault-domain separation is structural: a `GuestExitFrame` exists only via the Guest-exit stub, so Guest domain is a capture-path property; faults in EL2 context belong to the P1 fatal path and are never classified as Guest faults | the split must not depend on runtime guesses about fault origin; matches the stub's unconditional-capture design | W04 [03](../p4-w04-vcpu-entry-exit/03-code-contracts-world-switch.md) §4; ADR §19; P0-W14 (M5) |
| D4 | Faulting-IPA reconstruction (FAR/HPFAR composition) is W06-owned, per the pinned architecture revision; unavailability is explicit (`Unavailable`), never a guessed address; QEMU divergences → Specification Investigation | exactly one owner for the composition avoids divergent duplicates; honesty about missing hardware data | Arm A-profile architecture (pinned revision); W01 A7 |
| D5 | Mapping cross-check reads only W02 `query` snapshots; `Agreement | Mismatch` is recorded per fault; a mismatch marks the run failed as an isolation-invariant evidence failure, retains full diagnosis, and raises an invariant-review item — it is not a runtime panic and not an ignorable anomaly | P4-V07/V08 require "does not rely on stale translations"; a ledger/hardware contradiction is exactly that property failing, so it must surface as failed evidence | W02 D9; task book P4-V07/V08; P0-W14 escalation semantics (M5) |
| D6 | W06 owns the IS-series expectation matrix as a refinement of W05's expected-outcome column; refinements take effect only through W05's joint-change rule; probe addresses ride the W03/W05 boot-info channel | one stable expectation source for W07/W08; respects sibling artifact ownership | W05 §6 change rules; W06 plan work sequence item 4 |
| D7 | Containment-first ordering: W04's stop decision executes before report/dump; heavy diagnostics run after the stop in quiescent context; nothing diagnostic is added to the capture path | bounds exit-path work (Coding Guidelines); the retained frame already carries everything the report needs | W04 stub discipline; Coding Guidelines (never block/extend VM-exit paths) |
| D8 | The mapping dump is ledger-derived (iterating W02's authoritative mapping state); a hardware page-table walk is Reserved behind a future W02 seam | W02 D9 reserves the hardware walk; diagnostic fidelity equal to the ledger is sufficient for P4 | W02 D9; task book P4 "page-table dump" intent |
| D9 | All W06 diagnostics route through the P0 logging/trace baseline with stage-local `diag.*` names and build identity; the human report is a bounded, truncation-annotated write into a caller-provided buffer; no heap allocation | ADR-048; W01 A8; Coding Guidelines (bounded allocation in exception paths) | ADR-048; P0-W12/W13 (M5) |
| D10 | W06 adds no `unsafe`; implementation-time discoveries of an `unsafe` need are recorded deviations with SAFETY justification and an inventory entry, reviewed before merge | the diagnosis layer has no hardware access of its own; keeps the audited unsafe surface at W04/W02/W03's boundary | ADR-006; P0-W10 governance (assumed, W01 R20); Coding Guidelines |

## 5. Open items recorded by this design

| ID | Item | Owner / path | Handling |
|---|---|---|---|
| O1 | The W03 boot-info channel must carry the three IS probe configurations (unmapped gap, RAM boundary, Hypervisor-owned range). The tracked W03/W05 designs imply probe/window carriage but do not enumerate a three-probe field set. | [P4-W03](../p4-w03-guest-memory-image/README.md) / [P4-W05](../p4-w05-validation-guest/README.md) joint change rule; W06 records the requirement | resolved before W06 on-target isolation evidence; a capacity gap is a joint design note, never a W06-private encoding |
| O2 | IS-series refinements of W05 expected outcomes (finer access-type, IPA, and agreement assertions) must be acknowledged under the W05 §6 joint-change rule so the two tables cannot drift. | W05/W06/W07/W08 joint review at implementation | recorded as a review row in the [workflow](05-implementation-workflow.md) step 5; a silent divergence fails review |
