# P8-W12 Linux Memory-Model Validation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Validation that Stage-2 and the approved Guest map can carry Linux
memory-management behavior, as required by
[P8-W12](../../plans/p8-w12-linux-memory-model.md).  
**Owner-change context:** P8-W12 implementation handoff; memory work is
validation-scoped over the evidenced P2/P4 ownership and Stage-2 contracts. It
owns no memory mechanism.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W12. The plan makes W12 the
package that proves the Guest address space behaves correctly under real Linux
memory management: RAM initialization, allocators, page tables, map/unmap, TLB,
COW, kernel/user switching, small/normal/larger RAM cases, reserved/MMIO
boundaries, and Stage-2 diagnostics. This design converts that into (a) a
bounded evidence-category model separating Linux-internal behavior from
Guest-map boundary behavior, (b) a RAM-class scheme that fixes classes but no
capacities, (c) a boundary and negative-case matrix that preserves
ownership-before-mapping invariants, and (d) probe and workload contracts with
failure routing. It deliberately does **not** redesign Stage-2, implement page
tables, select RAM capacities, or touch overcommit, ballooning, snapshot, or
Host allocator policy.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

- [Memory matrix and boundary contract](01-memory-matrix-and-boundaries.md) —
  read before declaring or reviewing the matrix; upstream contracts, evidence
  categories, RAM classes, boundary/negative cases, ownership invariants.
- [Probe and workload contracts](02-probe-and-workload-contracts.md) — read
  before building scenario storage, isolation probes, or Guest memory workload
  requirements; normative interface obligations.
- [Implementation workflow and review](03-implementation-workflow-and-review.md)
  — read before executing; ordered steps, validation matrix, failure/security/
  observability model, and handoff checklist.

Before editing, the agent must also satisfy the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index, ADR baseline, P8 task book, and
the P8-W12 plan. Nothing here claims that any matrix row has run, that P2/P4
deliver their planned behavior, or that the machine map is approved.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → evidenced P2/P4 handoff
contracts → P8-W12 plan → this design → Coding Guidelines. In particular:

- ADR-018 makes the Stage-2 address space the core memory object and ADR §19
  makes page ownership an invariant (`MUST`); the Plan Agent guide's
  "ownership before mapping" rule is load-bearing here: every validation
  artifact must treat the Guest RAM backing as an owned object whose mapping
  bounds never exceed the donation. Any validation need that would require a
  Stage-2 or ownership-mechanism change is an `Architecture Change Request`,
  not a W12 edit.
- The task book P8 boundary for P4 is "Linux compatibility gap closure; no
  Stage-2 or VM foundation redesign", and the plan excludes Stage-2 redesign,
  page-table implementation, concrete RAM sizes, overcommit, ballooning,
  snapshot, and Host allocator policy. This design encodes those exclusions.
- The machine map is not frozen (ADR section 18 via the task book). Map
  boundaries, reserved regions, and the MMIO window are parameters taken from
  the approved facts of [P8-W02](../../plans/p8-w02-machine-contract-governance.md)
  and the [P8-W04](../p8-w04-guest-dtb-contract/README.md) DTB contract; no
  address or capacity is fixed here.
- Linux is an untrusted Guest (ADR-007): every Linux-caused fault is a
  recoverable VM-facing error with actionable diagnostics, never an EL2 panic.

Classification:

- **Required:** the evidence-category model; the small/normal/larger RAM-class
  scheme without capacities; the boundary and negative-case matrix (unmapped,
  reserved, Host-owned, MMIO, permission-conditional); the Stage-2 diagnostic
  sufficiency requirement feeding [W13](../p8-w13-guest-fault-diagnostics/README.md);
  Guest memory workload requirements with stable markers; evidence requirements
  delivered to W16 and W18.
- **Reserved:** permission-differentiated Guest regions beyond what the
  approved v1 map declares (trigger: the approved map declares such regions);
  RAM-class capacities above the approved map's window (never authorized);
  memory telemetry beyond evidenced P4/P2 fields.
- **Out of Scope:** Stage-2 redesign or page-table implementation; concrete RAM
  capacities; memory overcommit, ballooning, snapshot; Host allocator policy;
  W16 harness mechanics; Linux version/configuration selection (W15); DTB
  content ownership (W04); security regression scenarios beyond the boundary
  matrix (W18's lane).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02/W05/W10 and P2/P4 memory ownership facts | [ledger](#current-state-findings-and-goal-to-baseline-ledger); [matrix contract](01-memory-matrix-and-boundaries.md) §1 | W12-DV01 upstream review |
| Define Linux memory behavior and Guest-map evidence categories | [matrix contract](01-memory-matrix-and-boundaries.md) §2–§3 | W12-DV02 category review |
| Establish small/normal/larger fixture classes without selecting capacities | [matrix contract](01-memory-matrix-and-boundaries.md) §4 | W12-DV03 class review |
| Define boundary and negative cases for reserved, MMIO, Host memory, Stage-2 faults | [matrix contract](01-memory-matrix-and-boundaries.md) §5–§6 | W12-DV05 boundary execution |
| Review results against ownership and Guest-untrusted constraints | [workflow](03-implementation-workflow-and-review.md) §2 step 6 | W12-DV06 invariant review |
| P8-V17 memory-size/isolation matrix with actionable diagnostics | [workflow](03-implementation-workflow-and-review.md) §2 step 5, §3 | W12-DV04/DV05 (executed via W16) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation scaffold only — no Cargo workspace, no Rust sources, no
implementation or verification records for P1–P7, and no approved P8 machine
facts. Stage-2, the ownership ledger, and Linux are planned, not observable.
Each ledger row states the missing foundation the plan outcome requires and who
owns it.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Stage-2 + approved map carry Linux memory behavior (P8-V17) | No Stage-2, no Guest map, no Linux | Evidenced Stage-2 capability boundary (map/unmap/protect/query/activation) from P4; approved Guest map facts from W02/W04 | Memory validation observes an existing capability against an approved map; neither exists to observe yet | P4-W02 + P4-W09 handoff (`../../../p4/plans/p4-w09-closeout-p5-handoff.md`); W02/W04 | P4 Stage-2 evidence; approved v1 map record |
| Ownership-before-mapping invariant holds | No ownership ledger exists; P2 planned | Evidenced P2 allocation/ownership contract with per-page owner state | Boundary verdicts depend on querying real ownership, not assumptions | P2-W10 handoff (`../../../p2/plans/p2-w10-p3-p4-handoff-contract.md`) | P2 ownership evidence; W12-DV06 |
| Host-memory isolation preserved | No Host memory manager exists | Evidenced P4 protected/Host-owned range exclusion (P4-W06 basis) | The negative cases must prove Guest access to Host ranges faults | P4-W09 handoff; [W18](../p8-w18-security-isolation-regression/README.md) reuses | P4-W06 isolation evidence |
| Small/normal/larger RAM cases | No RAM values anywhere; plan forbids selecting capacities | RAM-class scheme (01 §4) with capacities recorded at implementation from approved map bounds | The classes must exist as declared fixtures without pre-empting the map decision | W12 (classes, this design); capacities from approved facts | W12-DV03; executed matrix W12-DV04 |
| Actionable Stage-2 diagnostics retained | No diagnostics exist; P4-W06 planned | Evidenced P4-W06 fault classification and context fields | "Actionable" is defined by the W13 minimum-context contract over those fields | P4-W06 (`../../../p4/plans/p4-w06-fault-isolation-diagnostics.md`); W13 extension | W12-DV05 context sufficiency checks |
| Linux runs memory workloads | No Linux, no fixture | Evidenced W09/W10 Linux boot; W15 fixture with the 02 §5 memory workloads | Workloads are fixture content executed on booted Linux | W09/W10 records; [W15](../p8-w15-reproducible-linux-fixture/README.md) | P8-V12–V15; W15 manifest |
| Matrix executed automatically | No harness | Scenario descriptors + probe semantics as obligations on W16's design | W16 owns automation mechanics; W12 owns what must be evaluated | [W16](../p8-w16-automated-linux-regression/README.md) | W12-DV04/DV05 evidence |

No row authorizes W12 to build memory mechanisms; where a prerequisite delivers
differently than assumed, [the workflow](03-implementation-workflow-and-review.md)
§1 failure boundary applies.

## Resolved design decisions and their authority

1. **Two-layer evidence model.** Linux-internal memory behavior (allocators,
   Guest page tables, TLB, COW, kernel/user switching) is validated as
   "completes without unclassified Stage-2 events", while Guest-map boundary
   behavior (unmapped, reserved, Host-owned, MMIO windows) is validated with
   explicit per-case expectations. Rationale: conflating them makes Linux
   internals look like hypervisor obligations; the plan's scope split
   (memory-model behavior vs map boundaries) supports the separation.
2. **RAM classes without capacities.** Small/normal/larger are defined
   relationally (01 §4); capacities are recorded at implementation time within
   approved map bounds. Rationale: the plan forbids selecting capacities and
   the map is ADR-deferred; classes must still be bounded for P8-V17 to be
   checkable.
3. **Boundary cases are expected-fault cases, never crash cases.** Every
   negative case names its expected classification, its contained VM outcome,
   and its required diagnostic context. Rationale: ADR-007/§19 and the
   P8-V17 "actionable Stage-2 diagnostics" wording.
4. **Ownership checks are ledger-based.** Isolation verdicts consult the
   evidenced P2/P4 ownership query paths (single-owner, donation bounds,
   protected-range exclusion); W12 adds no new ledger mechanism. Rationale: the
   task book forbids Stage-2/ownership redesign; a missing query path is a
   `P4DependencyIssue`.
5. **Guest workloads are fixture needs.** `mem-exercise` and `fork-storm` are
   specified as functional requirements with stable markers (02 §5), realized
   by the W15 fixture and executed by W16; W12 writes no Guest code. Rationale:
   mirrors the W11/W15 division and the plan's fixture boundary.
6. **No positive mapping operation is designed.** W12 probes consume the
   approved map as loaded at boot; it performs no runtime map/unmap of Guest
   RAM. Dynamic mapping belongs to later stages. Rationale: keeps W12 inside
   the P8 single-VM, statically-loaded boot model of W03/W04.

## Work breakdown and loading order

1. Read [the memory matrix and boundary contract](01-memory-matrix-and-boundaries.md)
   to understand which upstream contracts are consumed, the evidence
   categories, the RAM classes, and every boundary/negative case with its
   expectation.
2. Read [the probe and workload contracts](02-probe-and-workload-contracts.md)
   when building or reviewing scenario storage, probes, or Guest workload
   requirements; it fixes the obligations W16's design realizes and W15's
   manifest serves.
3. Execute in the order given in [the implementation workflow](03-implementation-workflow-and-review.md):
   verify prerequisites, record class capacities, review the matrix, execute
   through W16, then review diagnostics and invariants.
4. Store actual commands, observations, and results in
   `../../verification/p8-w12-linux-memory-model-verification.md`, and record
   class capacities, changed artifacts, and deviations in
   `../p8-w12-linux-memory-model-record.md` only when implementation begins.
   Neither this design nor a written record may claim W12 complete.

## Explicitly excluded interfaces

W12 designs no Stage-2 API, page-table format, allocator, ownership ledger
mechanism, VMID policy, or Host memory policy; no machine map value, RAM
capacity, or DTB property; no new EL2 trace event or diagnostic field (the
diagnostic context is W13's contract over evidenced P4-W06 fields); no W16
harness internals; no Linux kernel configuration. The only interfaces fixed
here are the validation-side contracts in
[02-probe-and-workload-contracts.md](02-probe-and-workload-contracts.md),
obligations on consumer designs, not implementations. A validation need that
cannot be met without one of the excluded surfaces is an `Architecture Change
Request` or a prerequisite issue, recorded — never implemented here.

## Downstream handoff

- **W16** ([design](../p8-w16-automated-linux-regression/README.md)) receives
  the memory-size/isolation matrix descriptors, probe semantics, workload
  markers, and expected per-case outcomes as normative content for its
  automation rows (P8-V21/V22 referencing M12-*).
- **W18** ([design](../p8-w18-security-isolation-regression/README.md)) receives
  the boundary/negative-case expectations and ownership-review method as the
  memory-isolation foundation for its security regression; W12's cases are
  functional-isolation evidence, not a security certification.
- **W13** ([design](../p8-w13-guest-fault-diagnostics/README.md)) receives the
  Stage-2 diagnostic sufficiency requirement and the observed Stage-2 fault
  subclasses as inputs to the Linux-facing fault taxonomy.
- **W15** ([design](../p8-w15-reproducible-linux-fixture/README.md)) receives
  the Guest memory workload functional requirements (02 §5) as declared
  validation needs.
- **W17** may reuse the RAM-class scheme for its baseline runs; W12 records no
  performance figure.
- **P2/P4, via issue routing:** any `P4DependencyIssue`/`P2DependencyIssue` is
  recorded against the P4-W09/P2-W10 handoffs; resolution belongs to those
  stages' authorities.
- **W20** closeout receives W12's limitation statements (no advanced memory
  feature proven) per the plan's acceptance wording.
