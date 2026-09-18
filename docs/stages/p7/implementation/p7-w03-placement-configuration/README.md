# P7-W03 Placement and Scheduling Configuration — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Static pinned equivalence and the explicit affinity, pinning,
dedicated/shared, invalid-control, and telemetry-visible configuration
semantics required by
[P7-W03](../../plans/p7-w03-placement-configuration.md).  
**Owner/change context:** P7-W03 implementation handoff; placement
constraint authority for W05/W07/W08/W11 per the plan handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W03. The plan fixes *what*
must hold (pinned operation equivalent to the P4 static baseline; affinity,
pinning, dedicated/shared semantics; non-silent rejection of invalid
controls; telemetry-visible configuration state) and defers the data
representation and validation mechanics to this design. It defines the
placement constraint model as pure, host-testable validation logic plus a
frozen per-vCPU resolved placement consumed by the scheduler.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the supporting file for its assigned step:

- [Scope and foundations](01-scope-and-foundations.md): requirement ledger,
  prerequisite contracts with failure boundaries, itemized scope, and the
  mechanism/policy split.
- [Architecture and state](02-architecture-and-state.md): logical modules,
  the placement lifecycle, the pCPU-registry mapping, ownership, and the
  authorization boundary.
- [Placement code contracts](03-code-contracts-placement.md): full
  function/type/data-structure contracts with pseudocode (`CpuSet`,
  `PlacementSpec`, validation, eligibility predicate, ledger, telemetry
  fields, authorization hook).
- [Implementation workflow](04-implementation-workflow.md): ordered steps.
- [Validation and handoff](05-validation-and-handoff.md): validation matrix
  (P7-V05–V07), error/security/observability model, handoff checklist.

Before editing, the agent must follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P7 task book,
W01 register, P7-W02 lifecycle design, and the P7-W03 plan). This design
proposes only; it claims nothing.

## Authority, constraints, and scope classification

Governing order: ADR baseline → P7 task book → frozen P0–P6 contracts →
P7-W03 plan → this design. Binding constraints:

- **ADR-016:** CPU affinity and dedicated CPU are *scheduler policy*, not
  separate VM types; P7 evolves from static pinned to M:N while preserving
  pinned operation. This design realizes pinned/affinity/dedicated as
  placement modes of the one vCPU type.
- **ADR-015/041–045:** eligible host resources come from the P2/P3 platform
  facts and pCPU registry; no platform-name or board-name branching anywhere
  in placement logic.
- **ADR-013/P5:** configuration is a controlled action; entry points
  authorize through the P5 capability/rights model. This design fixes the
  hook, not the rights vocabulary (owned by P5; see
  [architecture and state](02-architecture-and-state.md) §5).
- **Task book:** "explicit invalid configuration errors, and no silent
  fallback" — every invalid control produces a typed error and a trace-
  visible rejection; a rejected configuration never widens silently.
- **W02 lifecycle authority**
  ([P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md)): placement
  attaches at configuration time to `Offline` vCPUs and is frozen during P7
  operation; the gate consumes the eligibility predicate defined here.

Classification:

- **Required:** `CpuSet` representation with a topology-derived capacity
  bound; `PlacementMode` (`Pinned`/`Shared`) with dedicated/pinned/shared
  semantics; `validate_placement` with the full typed error set;
  `is_eligible` predicate (the W02/W05 seam); the placement ledger;
  static-pinned equivalence statement; telemetry semantic fields;
  authorization hook; explicit rejection of reconfiguration attempts
  (`ReconfigurationNotSupported` — dynamic policy switching is Reserved).
- **Reserved:** dynamic reconfiguration, migration/load balancing, NUMA,
  weights/quotas, RT classes (ADR-017), CPU hotplug reaction policy. The
  representation and error set must not preclude them, but none is designed.
- **Out of Scope:** configuration ABI or Control Domain interface (task book
  Out of Scope; management ABI is P10+); runqueue/pick policy (W05);
  pause/stop flows (W07); remote reschedule and idle (W08); affinity-mask
  *naming/wire* formats (the in-memory representation is fixed here; any
  external encoding needs its own design); trace encoding (W09).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02 lifecycle and P3 eligible-pCPU contracts | [workflow](04-implementation-workflow.md) step 1 | W03-DV01 review |
| Static pinned equivalence to the P4 baseline | [contracts](03-code-contracts-placement.md) §4 (equivalence statement); [validation](05-validation-and-handoff.md) DV04 | P7-V05 |
| Placement behavior for pinned, affinity, dedicated/shared | [contracts](03-code-contracts-placement.md) §2–§3; [architecture and state](02-architecture-and-state.md) §3 | P7-V06 |
| Invalid-control outcomes, non-silent | [contracts](03-code-contracts-placement.md) §3 (error set); [validation](05-validation-and-handoff.md) DV05 | P7-V07 |
| Telemetry-visible configuration state | [contracts](03-code-contracts-placement.md) §6 | P7-V07/V19 seam (rendering W09) |
| Conformance with ADR-016 and capability authority | [architecture and state](02-architecture-and-state.md) §5; [scope and foundations](01-scope-and-foundations.md) §4 | W03-DV02 review |
| Placement matrix evidence planning; handoff | [workflow](04-implementation-workflow.md) steps 5–7; [validation and handoff](05-validation-and-handoff.md) §3 | P7-V05–V07 evidence |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs`):
P0 documentation scaffold; no code exists. P2/P3 platform and pCPU contracts
are plans only (P7-IN-03/04 in the W01 register); the P7-W02 lifecycle
design this package consumes is a proposed sibling design. Every consumed
contract is therefore an assumed contract cited by plan path, with a failure
boundary below.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| "Establish static pinned equivalence" (plan Goal) | P4 static binding exists as a plan-level behavior; P7 has no placement model at all | A `Pinned` mode whose eligibility set is exactly one online pCPU and whose scheduling behavior is provably the P4 baseline behavior under the W02 gate | "Equivalence" must be a stated property with a comparison basis, not an assertion | P7-W03 (model); P4 owns the baseline behavior | W03-DV04 baseline comparison review; P7-V05 |
| "explicit affinity, pinning, dedicated/shared … semantics" (plan Scope) | No representation exists | `CpuSet` + `PlacementMode` + `ResolvedPlacement` with defined semantics per mode | Semantics need concrete, bounded, testable data first | P7-W03 (this design) | W03-DV02/03 tests |
| "non-silent rejection of invalid scheduling controls" (plan Scope) | No validation exists | Total `validate_placement` with the §3 typed error set, each error trace-visible | "Non-silent" requires the rejection path to be designed, not incidental | P7-W03 | W03-DV05 negative tests; P7-V07 |
| "telemetry-visible configuration state" (plan Scope) | No placement events exist | Semantic fields for applied/rejected configuration (rendering W09) | Visibility is a designed interface (Plan guide), not debug prints | P7-W03 semantics; W09 encoding | W03-DV06 seam review |
| "Review conformance with … capability-based authority boundaries" (work sequence 4) | P5 rights model is an unevidenced assumed contract | Authorization hook contract naming the required upstream check | Without the hook, configuration would be an unguarded control path | Hook: W03; rights vocabulary: P5 | W03-DV02 review |

No row invents a crate, target, or runtime policy beyond this design's
recorded decisions.

## Resolved design decisions and their authority

1. **Two placement modes, three semantics.** `PlacementMode::Pinned(pcpu)` —
   vCPU may run only on that pCPU, and the pCPU is *exclusive* (dedicated):
   no other vCPU may be eligible on it. `PlacementMode::Shared(cpu_set)` —
   vCPU may run on any *online* pCPU in the set, which may also host other
   shared vCPUs. "Dedicated" is thus realized as the exclusivity side of
   `Pinned` (a dedicated-CPU VM is one whose vCPUs are all pinned);
   "static pinned" is `Pinned` under the W02 gate. Authority: ADR-016
   (affinity/dedicated are policy, one vCPU type); the two-mode collapse of
   the task book's four terms is stage-local design freedom recorded here
   because the task book defers the representation.
2. **Affinity representation.** `CpuSet`: fixed-capacity bitset over the
   logical pCPU id space, capacity = the platform CPU maximum delivered by
   the P2/P3 topology facts (P7-IN-03/04), compile-time constant per build
   configuration. No heap, `no_std`-safe, trivially testable. Rationale:
   bounded, audit-friendly, matches newtype/checked-arithmetic guidelines.
   Any external (wire/config) encoding of affinity is explicitly not this
   type and needs its own ABI design.
3. **Placement is configuration-time and frozen.** Placement validates and
   attaches only while the vCPU is `Offline` (W02 lifecycle authority);
   `validate_placement` on a non-`Offline` vCPU returns
   `ReconfigurationNotSupported` — an explicit typed rejection, satisfying
   "no silent fallback" for the dynamic case while keeping dynamic policy
   switching Reserved.
4. **Eligibility is a pure triple check.** `is_eligible(placement, pcpu,
   online)` = (pcpu ∈ placement set) ∧ online(pcpu) ∧ (not reserved by
   another pinned vCPU). It is stateless: the online re-check at pick time
   (W05/W08 consume it) lets a failed pCPU (P3 `Failed`) drop out without
   placement rewrite. Authority: P3-W03 registry semantics via P7-IN-04.
5. **Exclusive-pCPU bookkeeping lives in the ledger.** The placement ledger
   maps every configured vCPU to its `ResolvedPlacement` and answers "is
   this pCPU exclusively held?" in O(1); validation is atomic against the
   ledger (two concurrent validations cannot both claim exclusivity).
   Authority: single-writer ownership rule of the implementation checklist.
6. **Authorization hook, not vocabulary.** The configuration entry point
   requires a capability receipt from the P5 rights check whose rights
   include the scheduling-policy control right; the right's concrete
   identifier is owned by the P5 rights model (P7-IN-06). If P5 delivers no
   suitable right, that is a prerequisite mismatch recorded per the W01 §5
   procedure — P7 does not mint a new authority concept.
7. **Rejected configurations leave no residue.** A failed validation
   attaches nothing; the vCPU remains `Offline` and unrunnable; a
   rejection event is emitted. Partial application is impossible by
   construction (validation returns a complete `ResolvedPlacement` or an
   error, never a defaulted one) — this is the mechanical form of "no
   silent fallback".

## Work breakdown and loading order

1. Read this README; load supporting files per assigned step as listed
   above.
2. Implement in the order of
   [the implementation workflow](04-implementation-workflow.md): `CpuSet` →
   spec/validate → ledger → eligibility seam → telemetry fields →
   authorization hook → integration.
3. Findings go to `../p7-w03-placement-configuration-record.md`; evidence
   to
   `../../verification/p7-w03-placement-configuration-verification.md`.
   Neither is created by this design; no completion claim is made anywhere
   in it.

## Explicitly excluded interfaces

No public API, management ABI, configuration file format, wire encoding, or
trace binary format is designed or authorized. All names in
[the placement contracts](03-code-contracts-placement.md) are internal,
stage-local design names. Runqueue, pick policy, pause/stop, remote
reschedule, idle, counters, and guest workloads are owned by W05–W10
designs. Placement logic must not read timers, GIC state, or any
architecture register; a need to do so is a layering conflict to record.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md) consumer map:

- **W05** receives `is_eligible` as the enqueue/pick filter and the ledger
  snapshot for duplicate-dispatch exclusion; the enqueue-placement hint rule
  is W05 scope and must respect eligibility (see
  [P7-W05](../p7-w05-shared-mn-multivm/README.md)).
- **W07** receives the frozen-placement rule: pause/resume preserves
  `ResolvedPlacement` untouched; stop excludes the vCPU from the ledger's
  eligible answers via lifecycle state, not placement mutation.
- **W08** receives the online-re-check rule for idle/wake placement
  decisions and the rule that pCPU failure (P3 `Failed`) removes
  eligibility without placement edits.
- **W11** receives the placement matrix definitions
  ([validation](05-validation-and-handoff.md) DV04) as stress scenarios.
- **W09** receives the applied/rejected semantic fields for rendering.

W14 carries the placement semantics record into the P8 handoff; P8 may rely
only on evidenced semantics per the task book §7.
