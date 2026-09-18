# P2-W06 Platform and Memory Inspection — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** One read-only inspection result — platform summary, boot-map dump,
allocator/heap statistics, and active-state consistency checks — derived
exclusively from the normalized state produced by W02–W05, required by
[P2-W06](../../plans/p2-w06-platform-memory-inspection.md).  
**Owner/change context:** P2-W06 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W06. It converts the bounded
work-package plan into a code-bearing design for exactly one mechanism: a
pure projection over the four authoritative P2 sources — W02's
`PlatformInfo`, W03's sealed `BootMemoryMap`, W04's `PageAllocator` stats,
and W05's `Heap` stats — that composes one `InspectionReport` value, checks
cross-source consistency, and renders it to a caller-supplied text sink for
boot diagnostics and regression evidence. It deliberately does **not** create
an inspection command or CLI (the ADR's `hv-platform-inspect` mode stays
Reserved), does not add a second source of platform numbers (every rendered
value is derived from a source by reference; a hard-coded view is the defect
P2-H04 forbids), does not choose a logger API, command format, or telemetry
implementation (plan work sequence 4), does not initialize any device or
execute PSCI, and does not change any W02–W05 contract.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md), and the
[P2-W06 plan](../../plans/p2-w06-platform-memory-inspection.md). This
document is a proposed design; it contains no implementation or validation
claim.

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, assumed W02–W05/P0/P1 source contracts with failure boundaries, the single-source projection policy, and the no-heap/no-allocation rendering rule. |
| [02-architecture-and-state.md](02-architecture-and-state.md) | The report/section model, composition order, consistency-check registry, lifecycle, and the read-only concurrency boundary. |
| [03-code-contracts-inspection.md](03-code-contracts-inspection.md) | Exact function/type contracts with pseudocode for composition, consistency checks, and rendering. |
| [04-implementation-workflow.md](04-implementation-workflow.md) | The ordered implementation steps with acceptance and failure handling. |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | The validation matrix (P2-V08), error/security/observability model, and the handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W06 plan](../../plans/p2-w06-platform-memory-inspection.md) → this design
→ Coding Guidelines. Binding constraints:

- ADR-048 (telemetry as a designed interface) and ADR-010 principle 10 make
  observability structural, but the plan explicitly withholds command
  format, logger API, and telemetry design from W06. This design therefore
  fixes the *result and rendering* contracts and leaves every user-facing
  surface Reserved.
- ADR-043/052: the report may not name a platform, board, or machine type;
  everything variable appears as fact states and capabilities exactly as W02
  recorded them.
- P2-V08's acceptance wording — "inspection reports the active normalized
  result and allocator statistics, not an independent hard-coded view" — is
  the package's hard rule: W06 owns zero authoritative numbers. A divergence
  between rendered values and the sources is a named fatal diagnostic, not a
  reporting choice.
- Layering: inspection is a diagnostic consumer of W02–W05 outputs. It sits
  above the platform/memory modules, adds no dependency on arch registers
  beyond the assumed P1 boot-context record
  ([01 §2](01-scope-and-foundations.md) A5), and never reaches into W04/W05
  internals beyond the published stats queries.

Classification. **Required:** the `InspectionReport` composition from the
four live sources, the platform/map/allocator/heap section content
(P2-H01–H03), the consistency-check set with named check IDs (P2-H04), the
bounded allocation-free renderer with stable section ordering, and host
testability of everything. **Reserved** (recorded triggers, no P2
implementation): the `hv-platform-inspect` user-facing mode or CLI, a
versioned machine-readable export format, telemetry event emission,
diff-against-previous-boot comparisons, and any filtering/verbosity policy.
**Out of Scope:** discovery, map construction, or allocation semantics
(W02–W05 own them); driver initialization; external management ABI;
P3/P4 runtime mechanisms; Orange Pi 3B behavior; and CI wiring.

## Requirement-to-design mapping

The tracked sources define P2-H01–H04 at group granularity only; the rows
below are this design's reviewable enumeration from the plan's scope
wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-H01 | Platform summary section covers boot-context (architecture/EL, P1 record), boot CPU and CPU topology, GIC/timer/PSCI/console facts with their five states, and the capability summary | [03 §3](03-code-contracts-inspection.md) | P2-V08 (W06-DV01, DV08) |
| P2-H02 | Map-dump section covers RAM, protected (per `ProtectedSourceId` class), allocatable spans, and the `MapSummary` accounting totals | [03 §4](03-code-contracts-inspection.md) | P2-V08 (W06-DV02) |
| P2-H03 | Allocator statistics section covers `AllocationStats` (managed/free/used/reserved per region) and `HeapStats` (per-class, budget headroom, large allocations), live-derived | [03 §5](03-code-contracts-inspection.md) | P2-V08 (W06-DV03, DV04) |
| P2-H04 | Consistency: every rendered value derives from the active sources; named cross-source checks C1–C4 detect divergence; no hard-coded view exists | [02 §4](02-architecture-and-state.md), [03 §6](03-code-contracts-inspection.md) | P2-V08 (W06-DV05, DV07) |
| Rendering discipline (plan step 4) | Renderer writes through a caller-supplied `fmt::Write` sink; stable section/line order; no allocation; output bounded | [03 §7](03-code-contracts-inspection.md) | P2-V08 (W06-DV06, DV09) |
| Handoff (plan step 6) | W09/W10/P3/P4 reviewers can state inspection expectations without new W06 work | [05 §4](05-validation-and-handoff.md) | W06 closure review (W06-DV10) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no workspace, no Rust sources, and no W01–W05
implementation. W06 is designed against the sibling designs'
[READMEs](../p2-w02-platform-discovery-normalization/README.md) and their
published contracts as assumed prerequisites: `PlatformInfo`/`FactState`
([W02 contracts](../p2-w02-platform-discovery-normalization/04-code-contracts-facts.md)),
`BootMemoryMap`/`MapSummary`/`ProtectedSourceId`
([W03 contracts](../p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md)),
`AllocationStats`
([W04 contracts](../p2-w04-physical-page-allocation/03-code-contracts-pagealloc.md)),
`HeapStats`/`audit`
([W05 contracts](../p2-w05-dynamic-small-allocation/03-code-contracts-heap.md)).
If any source lands differently, W06's composition adapters change but the
projection policy does not.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Inspection derives from the active normalized state (P2-V08, P2-H04) | No inspection code exists | Read-only composition over the four source values; zero owned numbers | Any independent data path would create the forbidden hard-coded view | W06 (this design), consuming W02–W05 outputs | W06-DV01–DV04, DV07 host tests and review |
| Summary content covers architecture/EL, boot CPU/topology, RAM/protected/allocatable, GIC/timer/PSCI/console, capabilities (plan step 2) | No summary exists | Fixed section schema per source, incl. P1 boot-context record (assumed contract A5) | The plan enumerates the content; each item needs exactly one source owner | W06 sections; W02–W05/P1 own the data | W06-DV01/DV02 |
| Map dump and allocator statistics use the same authoritative state as allocation/discovery consumers (plan step 3) | Nothing | Composition signatures taking `&PlatformInfo`, `&BootMemoryMap`, `&PageAllocator`, `&Heap` (stats queries only) | Passing the live values by reference is what makes the property structural | W06; W02–W05 publish the queries | W06-DV02–DV04 |
| Consistency between reported and active platform information (plan scope) | Nothing | Named check set C1–C4 over composed sections | Without checks, divergence would be silent | W06 | W06-DV05 |
| Inspection evidence for host and QEMU validation, absence of a divergent view (plan step 5) | Nothing | Render determinism + no-hard-coded-view review rows in the validation matrix | W06-DV06/DV07 are the reviewable form of the plan requirement | W06 | W06-DV06/DV07; QEMU observation deferred to W09 |
| W02–W05 outputs available (plan step 1) | Designed, not implemented | Assumed contracts with failure boundaries ([01 §2](01-scope-and-foundations.md)) | Task book §2 upstream-defect rule | W02–W05 owners; W06 consumer | Source verification records when they land; host fixtures meanwhile |

No ledger row invents a crate, target, command surface, or telemetry policy;
the absent upstream implementations are ordered prerequisites handled with
host fixtures.

## Resolved design decisions and their authority

1. **Inspection is a pure projection with zero authoritative state.** It
   owns no counters, caches, or copies that could diverge; it holds borrowed
   references only during composition and stores derived values for one
   report. Rationale: P2-H04 makes "independent hard-coded view" a defect
   class; the only structural cure is no second authority. `audit()`-style
   recomputation stays with W04/W05; W06 only renders and cross-checks.
2. **One composed value, one render.** `compose` runs once per inspection
   point and produces `InspectionReport`; `render` writes it through a
   caller-supplied `core::fmt::Write` sink. Rationale: text is an evidence
   surface, not the contract — host tests assert on the structured value, so
   the renderer can evolve without touching semantics; no logger API or
   command format is frozen (plan step 4 constraint).
3. **Section content is fixed per source; ordering is stable.** Sections:
   boot-context, platform, memory map, page allocator, heap, consistency.
   Rationale: W09's QEMU expectations and repeated-boot comparison need a
   stable section/line order; the text format itself remains stage-local
   (not an ABI) and may be revised by a later design with W09 expectation
   updates.
4. **Cross-source consistency is a named check set, and a broken check is a
   fatal invariant stop.** C1 (platform RAM total == map `ram_frames`), C2
   (map equation `ram == allocatable + protected` re-verified from rendered
   fields), C3 (allocator `managed == allocatable − metadata`, conservation
   vs table recount already W04's), C4 (heap pages ⊆ allocator used
   frames). Rationale: P2-H04 requires detectable divergence; per P0-W14 a
   broken accounting invariant is fatal, and P2 has no recovery consumer.
   Checks are cheap recomputations over published values, never a second
   bookkeeping system.
5. **The boot-context section renders the P1 record or an explicit
   `not recorded` marker; it never fabricates EL values.** Rationale: the
   plan requires architecture/EL content, but P1 owns the facts (assumed
   contract A5); W06 must not read architecture registers itself — that
   would duplicate P1-W03's inventory and add an arch dependency to a
   diagnostic module.
6. **No allocation anywhere in W06.** Composition uses fixed-capacity
   members and the renderer writes through the sink without buffering.
   Rationale: inspection must be callable at boot-diagnostics time (pre- or
   post-heap) and must stay host/QEMU-identical; W05's heap exists by then,
   but a diagnostic that can OOM is a diagnostic that cannot be trusted in
   the failure paths it exists to explain.
7. **`hv-platform-inspect` as a user-facing mode is Reserved.** The ADR §11
   names the mode; nothing in P2 authorizes a CLI, subcommand, or output
   contract for it. W06 delivers the report/render contracts a later mode
   design would consume. Rationale: the W06 plan lists an
   inspection-command implementation as out of scope.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   source contracts and the projection policy.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   section model, check registry, and lifecycle before writing any code.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md)
   in order; each step points into
   [03-code-contracts-inspection.md](03-code-contracts-inspection.md) for
   the exact contracts and pseudocode.
4. Validate per [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Record implementation decisions and deviations in
   `../p2-w06-platform-memory-inspection-record.md`, and commands,
   environments, and results in
   `../../verification/p2-w06-platform-memory-inspection-verification.md` —
   both files are created only when the corresponding work starts; neither
   this design nor a written record may claim W06 complete.

## Explicitly excluded interfaces

No CLI, subcommand, or user-facing command surface (Reserved, Decision 7);
no logger, tracing, or telemetry API — the `fmt::Write` sink parameter is
the only output seam; no machine-readable export or serialization of
`InspectionReport` (a versioned export would be a later design; raw struct
serialization is forbidden by the Coding Guidelines); no mutation of any
source (all inputs are shared references; stats queries are `&self`); no
revalidation of the DTB, no map rebuild, no allocator calls with side
effects; no platform/board names; no crate or workspace manifests (physical
module placement is pending P0-W03's workspace, recorded as an open item).
Any consumer need beyond the contracts in
[03](03-code-contracts-inspection.md) is a design change, not a local
addition.

## Downstream handoff

- **W09** ([../p2-w09-qemu-integration-regression/README.md](../p2-w09-qemu-integration-regression/README.md))
  receives the stable section schema and the consistency-check set as
  integration expectations: every QEMU boot must render a passing
  inspection (C1–C4 hold) and repeated boots must render identical sections.
- **W10** ([../p2-w10-p3-p4-handoff-contract/README.md](../p2-w10-p3-p4-handoff-contract/README.md))
  receives inspection as a P3/P4 review input: the report is how P3/P4
  reviewers assess P2 input readiness. It is explicitly not a control API
  (W10 records that boundary).
- **P3** consumes the recorded contract via W10 (p3-w01 topology inputs,
  p3-w11 observability planners): reviewers read the report; P3's own
  telemetry is a separate design.
- **P4** consumes the map/allocator sections via W10 (p4-w02/p4-w03
  planners) to confirm host-RAM topology and accounting before Stage-2
  design work.
- **W08** inherits the render-determinism and consistency-check fixtures as
  regression material (identical state ⇒ identical render; injected
  divergence ⇒ named check failure).
