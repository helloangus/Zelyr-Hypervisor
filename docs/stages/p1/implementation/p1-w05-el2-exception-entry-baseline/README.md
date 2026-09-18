# P1-W05 EL2 Exception Entry Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The EL2 exception entry baseline required by
[P1-W05](../../plans/p1-w05-el2-exception-entry-baseline.md): vector-table
coverage for all four exception categories and all sixteen vector origins,
bounded diagnostic-context capture, the syndrome interpretation boundary,
origin/context classification with a recoverable-versus-fatal boundary, the
unexpected and unhandled paths, and explicit `VBAR_EL2` ownership.  
**Owner/change context:** P1-W05 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W05. W05 makes every exception
the stage can take land in a valid, diagnosable EL2 entry path with a defined
outcome, and takes over `VBAR_EL2` — the one control
[P1-W04](../p1-w04-el2-architectural-state-baseline/README.md) deliberately
left unowned. It deliberately does **not** acknowledge or dispatch interrupts
(no GIC work of any kind), implement a return-from-exception path, handle
Guest or lower-EL exceptions semantically, deliver timer interrupts, or design
the future IRQ subsystem.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-architecture-and-state.md](01-architecture-and-state.md) — the logical
  module map, the vector/origin model, ownership of every piece of
  exception-path state, the exception-entry state machine (including the
  recursive-entry guard), the concurrency model, and the assumed-contract
  table with failure boundaries. Load this first for any step.
- [02-code-contracts-vector-install.md](02-code-contracts-vector-install.md) —
  contracts for the `exceptions` phase mechanism: baseline assertions, the
  `VBAR_EL2` write with read-back, the vector-table region, the audited
  `unsafe` boundary, and the vector-baseline declaration consumed by
  W07/W08/W09/W11.
- [03-code-contracts-entry-capture.md](03-code-contracts-entry-capture.md) —
  contracts for the sixteen vector stubs, the bounded `ExceptionFrame`
  capture, and the prohibited-content list for the entry assembly.
- [04-code-contracts-classification-routing.md](04-code-contracts-classification-routing.md) —
  contracts for origin/context classification, the syndrome interpretation
  boundary (the EC-class table), the disposition boundary, the routing to the
  fatal path or its pre-arm summary, and the diagnostic token classes.
- [05-implementation-and-review.md](05-implementation-and-review.md) — ordered
  workflow, validation matrix, error/security/observability model, and handoff
  checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W05
plan](../../plans/p1-w05-el2-exception-entry-baseline.md). This document is
the proposed detailed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W05 plan → this
design → Coding Guidelines. In particular:

- The task book requires "complete EL2 exception-vector baseline with
  synchronous/fatal diagnostics" with P1-V08/P1-V09: synchronous, IRQ, FIQ
  and SError have valid EL2 entry and origin/context classification, and
  intentional or unexpected synchronous faults expose syndrome and location
  before their defined outcome without unbounded recursive failure.
- The plan scopes vector coverage, origin/context classification, bounded
  context capture, the syndrome interpretation boundary, and the
  unexpected/unhandled paths — and explicitly excludes GIC initialization or
  dispatch, virtual interrupts, timer delivery, SMP notification, Guest
  exception handling, and a complete IRQ subsystem. A vector entry that
  acknowledges an interrupt controller or unmasks anything would violate that
  exclusion; none is designed here.
- The [W02](../p1-w02-minimal-rust-el2-runtime/README.md) runtime and
  [W09](../p1-w09-initialization-sequencing/README.md) lifecycle are accepted
  sibling contracts consumed directly: vectors install inside W09's
  `exceptions` phase, install failures route via W02's panic route (the
  matrix's `exceptions` row), and post-`stable` faults keep phase `stable`
  attribution (W09 T5/H6). The
  [W04](../p1-w04-el2-architectural-state-baseline/README.md) baseline is an
  accepted sibling contract consumed directly: W05 asserts the categories it
  builds on through W04's declaration API and takes over `VBAR_EL2` exactly
  where W04's ownership matrix assigns it. The
  [W07](../p1-w07-fatal-crash-diagnostics/README.md) report and
  [W06](../p1-w06-early-console-logging/README.md) channel are parallel
  designs consumed through recorded seams with failure boundaries.

Classification: the sixteen-entry vector table with all four category paths,
`VBAR_EL2` ownership and verification, the bounded capture frame, the
classification vocabulary with its recoverable-versus-fatal boundary, the
unexpected/unhandled paths, the recursive-entry guard, and the pre-arm
diagnostic summary are **Required**. A return-capable entry path (ERET with
state restore), a dedicated exception stack, an interrupt-acknowledge or
EOI path, selective synchronous-exception recovery, and growth of the capture
frame for later packages are **Reserved** with recorded triggers. GIC/timer
virtualization or dispatch, virtual interrupts, Guest exception handling,
EL1/EL0 exception delivery, SMP entry, a complete IRQ subsystem, and any
return-to-lower-EL semantics are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Establish vector coverage and the applicable origin/context categories (work seq 1) | [Architecture](01-architecture-and-state.md) §2–§4 | P1-V08 (W05-DV01, DV02) |
| Define minimum diagnostic context and classification outcomes (work seq 2) | [Entry capture](03-code-contracts-entry-capture.md) §2; [classification](04-code-contracts-classification-routing.md) §1–§3 | P1-V08, P1-V09 (W05-DV03) |
| Integrate valid-entry behavior with W04's architectural baseline (work seq 3) | [Vector install](02-code-contracts-vector-install.md) §1–§4 | P1-V08 (W05-DV04) |
| Review recursive-entry and unhandled-vector failure boundaries (work seq 4) | [Architecture](01-architecture-and-state.md) §5–§6; [capture](03-code-contracts-entry-capture.md) §3; [workflow](05-implementation-and-review.md) step 5 | P1-V09, P1-V12 (W05-DV05) |
| Define intentional synchronous and unexpected-vector acceptance evidence (work seq 5) | [Workflow](05-implementation-and-review.md) §3; [classification](04-code-contracts-classification-routing.md) §5 | P1-V09 (W05-DV06; NC3/NC6 execution via W11) |
| Hand off the exception contract to console, crash and fault validation (work seq 6) | Downstream handoff below; [workflow](05-implementation-and-review.md) §5 | W05 closure review (W05-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs`
at `4e631ee`): no exception-vector code, no `VBAR_EL2` write, no capture
frame, and no classification code exist anywhere in the tree; `hypervisor/src/`
and `crates/` contain only `.gitkeep`. The W02 runtime, W04 baseline, and W09
lifecycle this package runs between exist as accepted designs in this branch,
not as code; W06/W07 (named consumers) are parallel designs. Every executing
prerequisite is a contract, not a present artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Every category has a valid EL2 entry path (P1-V08) | No vector table, no `VBAR_EL2` write, no entry code | The sixteen-entry table and category handlers of [01-architecture-and-state.md](01-architecture-and-state.md) §2–§4 and [entry capture](03-code-contracts-entry-capture.md) §1 | An exception with no valid entry has no diagnostics and no defined outcome; coverage must exist before classification means anything | W05 (this design) | W05-DV01/DV02 |
| `VBAR_EL2` is explicitly owned | W04 records it unowned by design | The install-with-read-back and declaration of [vector install](02-code-contracts-vector-install.md) §2, §4 | An unowned vector base is exactly the firmware-residue pattern the stage removes elsewhere; W04's matrix names W05 the owner | W05 (ownership taken); W04 (boundary recorded) | W05-DV04 |
| Origin/context classification (P1-V08) | Nothing classified | The origin model and EC-class vocabulary of [classification](04-code-contracts-classification-routing.md) §1–§2 | "Classified" needs a fixed vocabulary fixed before any fault is taken | W05 | W05-DV03 |
| Syndrome/location exposed before the defined outcome (P1-V09) | No capture, no report seam | The bounded `ExceptionFrame` of [entry capture](03-code-contracts-entry-capture.md) §2 and the routing of [classification](04-code-contracts-classification-routing.md) §4 | Exposure requires both capture at entry and an established output seam at routing time | W05 (capture + summary); W07 (full report seam); W02 (early writer) | W05-DV03; NC3 execution via W11 |
| No unbounded recursive failure (P1-V09/P1-V12) | Nothing to bound | The recursive-entry guard and terminal-disposition rules of [architecture](01-architecture-and-state.md) §6 | Recursion containment must be an entry-path property, not caller discipline | W05 | W05-DV05 |
| Valid-entry behavior consistent with W04's baseline (work seq 3) | Baseline is a design, not code | The assertion set of [vector install](02-code-contracts-vector-install.md) §1 and the FP-free capture rule | Building on an unasserted baseline would be a hidden dependency (W09 H1) | W05 (assertions); W04 (declaration API) | W05-DV04 |
| Exception contract handed to W06/W07/W11 (plan handoff) | No contract exists | The declaration API, frame, and token classes defined across §02–§04 | Consumers must build on recorded surfaces, not assumptions | W05 | W05-DV07 |

No row requires designing a later-stage mechanism; the IRQ path ends at
classification by design, which is the plan's boundary, not a gap.

## Resolved design decisions and their authority

1. **`VBAR_EL2` ownership is taken explicitly, with W04-style write and
   read-back verification.** The install writes the vector-table base, reads
   it back, and routes a mismatch as a phase-attributed `exceptions` failure
   via the panic route — the same one-owner discipline W04 applies to its
   controls. The vector base is declared through a W05-owned declaration
   status so consumers build on the recorded fact, not on an assumption that
   the write happened. Authority: W04 ownership matrix (`VBAR_EL2` row);
   W04 decision 1/6 precedent; task book P1-V08.
2. **The table covers all sixteen vector entries, not just the four the P1
   runtime can legitimately take.** P1 executes at EL2 with `SPSel=1`, so
   only the four current-EL-with-SPx entries are legitimately reachable; the
   other twelve are wired to the `InvalidOrigin` classification and the fatal
   disposition. Rationale: P1-V08's coverage requirement is about valid entry,
   and an entry that falls through to arbitrary code would be an unowned
   failure surface. Authority: plan scope ("vector coverage",
   "unexpected and unhandled paths"); AArch64 vector architecture.
3. **Every classified event in P1 is terminal: the recoverable-versus-fatal
   boundary exists as a vocabulary, and P1's policy assigns every category
   the fatal disposition.** Synchronous exceptions during P1's own execution
   are invariant violations (W04's no-trap policy means none is expected);
   IRQ and FIQ arrivals are unexpected events (masks are set, nothing is
   enabled, no controller is configured); SError is fatal physical-error
   handling. The `Recoverable` disposition is reserved so P6+ inserts
   recovery without redefining the vocabulary. Authority: plan goal
   ("clear recoverable-versus-fatal boundary"); W09 matrix (`exceptions` row,
   post-`stable` row); P0-W14 classification semantics (assumed contract).
4. **No return path: every vector entry diverges; no ERET, no SPSR restore,
   no exception return code exists in P1.** A return path without a consumer
   would be untested machinery on the most safety-critical path in the stage;
   the first stage with a legitimate return (Guest entry, P4) designs it.
   Authority: plan out-of-scope list (no Guest/IRQ subsystem); stage-local
   design freedom with recorded rationale.
5. **The capture is a fixed-size frame stored in a dedicated static slot, not
   on the faulting stack.** A fault during stack exhaustion must still leave
   a capturable, reportable frame; one static slot suffices because P1 has
   one executing CPU, and the recursive-entry guard makes the slot
   single-consumer. The frame contains no FP/SIMD state — W04's deny posture
   (C4) means none can be live in P1 code, and the capture must not assume it.
   Authority: W04 handoff ("save areas must not assume FP state");
   [W02 architecture](../p1-w02-minimal-rust-el2-runtime/01-architecture-and-state.md)
   §4 concurrency model; stage-local design freedom.
6. **Pre-arm exceptions get a bounded summary through W02's early writer;
   post-arm exceptions get the full W07 report.** The fatal path is W07's
   mechanism and becomes the route only when its `fatal-path` phase completes
   (W09 H4: a phase's route must be established by an earlier phase). Until
   then the route is the panic-route era: a fixed summary line set (class,
   syndrome, PC, fault address, phase) through the channel-independent early
   writer, then the bounded stop — exactly the matrix's "syndrome and
   location context as available". NC3's console-phase insertion point
   (pre-arm) observes the summary; its later variant observes the full
   report. Authority: W09 matrix `exceptions`/`stage1` rows; W11 NC3;
   W02's early-writer contract §3 (panic-route-era fallback role).
7. **The syndrome interpretation boundary is a fixed EC-class table with
   FAR/HPFAR validity flags; no per-syndrome ISS decoding exists in P1.**
   The table names the exception classes P1 can take (by EC value class),
   gives each a static label, and records whether FAR_EL2 / HPFAR_EL2 are
   architecturally valid for it. ISS-level interpretation is later-stage
   work (trap handling, P4+); decoding it now would be unused surface on the
   fatal path. Authority: plan scope ("syndrome interpretation boundary");
   stage-local design freedom with recorded rationale.
8. **The audited `unsafe` boundary is W05-owned and minimal: the vector
   install/capture system-register and barrier instructions.** W04's
   `ControlId` set deliberately excludes the exception registers, so reuse is
   not available and not attempted; W05 defines its own named primitives
   (`vbar_write`, `exception_state_read`, barrier/instruction-cache
   wrappers) over exactly the registers its contracts name. The assembly
   stubs are the second boundary (architectural, compiler-untypeable).
   Authority: W04 boundary precedent; Coding Guidelines unsafe rules;
   P0-W10 governance (assumed contract).

## Work breakdown and loading order

1. Load [01-architecture-and-state.md](01-architecture-and-state.md) for the
   module map, vector/origin model, state ownership, exception-entry state
   machine, and assumed contracts. Every implementation step depends on it.
2. Load [02-code-contracts-vector-install.md](02-code-contracts-vector-install.md)
   for the phase mechanism and `VBAR_EL2` ownership,
   [03-code-contracts-entry-capture.md](03-code-contracts-entry-capture.md)
   for the entry assembly and capture, and
   [04-code-contracts-classification-routing.md](04-code-contracts-classification-routing.md)
   for classification and routing.
3. Execute the steps in the order given in
   [05-implementation-and-review.md](05-implementation-and-review.md):
   prerequisite confirmation, baseline assertion and install, entry/capture
   assembly, classification and routing, boundary reviews, evidence and
   handoff.
4. Record implementation decisions and deviations in
   `../p1-w05-el2-exception-entry-baseline-record.md` when implementation
   begins, and validation commands, environments, and outcomes in
   `../../verification/p1-w05-el2-exception-entry-baseline-verification.md`
   when evidence exists. Neither file may exist yet, and neither this design
   nor a record may claim W05 complete.

## Explicitly excluded interfaces

No interrupt-acknowledge, EOI, priority, or routing-table mechanism; no GIC
register access of any kind; no timer-delivery or virtual-interrupt path; no
Guest or lower-EL exception semantics (the twelve lower-EL entries classify
and terminate); no ERET/return machinery; no IRQ-subsystem trait or framework;
no console transport (W06) and no crash-report body (W07 — W05 renders only
its pre-arm summary through W02's writer and hands the frame to W07's seam).
No public ABI, wire format, or persistent layout is introduced; the token
classes of [classification](04-code-contracts-classification-routing.md) §5
are P1-internal boot diagnostics consumed by W10/W11 matching rules, not an
ABI. No allocator, no heap, no dynamic registration: the vector path is
static by construction.

## Downstream handoff

- **[P1-W06](../p1-w06-early-console-logging/README.md)** receives the
  guarantee it may rely on per its plan: from `exceptions.complete` an
  exception path exists that can emit context (pre-arm summary pre-MMU; full
  W07 report post-arm), and its channel must remain callable from exception
  context (integration contract in its design).
- **[P1-W07](../p1-w07-fatal-crash-diagnostics/README.md)** receives the
  captured `ExceptionFrame` (fields, ownership, lifetime), the disposition
  vocabulary, and the two call obligations its design must satisfy: the
  `report_fatal_exception` seam and the readiness/arming semantics its
  `fatal-path` phase body performs.
- **[P1-W08](../p1-w08-host-stage1-address-space/README.md)** receives the
  vector-table region identity and its required post-MMU attributes
  (executable, read-only, cacheable per the mapping-class table in its
  design); with the P1 identity mapping the recorded `VBAR_EL2` value stays
  valid across the transition, and any future relocation owns the VBAR
  update through its own design.
- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  `exceptions` phase body (the mechanism entry its `exceptions_step` adapter
  calls), the unowned-window narrowing it already records (vectors exist from
  this phase's completion), and confirmation of the post-`stable` route.
- **[P1-W10](../p1-w10-qemu-boot-regression/README.md)** receives the
  fatal-crash token classes produced by the pre-arm summary (post-arm tokens
  come from W07's report markers); a normal boot must contain zero of either.
- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives NC3's
  target (the synchronous-exception boundary: summary/report, phase
  attribution, terminal outcome), NC6's classification target (the
  unexpected-event vocabulary, with the recorded suggested category for the
  stable-state scenario), and the reproducibility-relevant property that
  classification is deterministic.
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  exception contract content (coverage, classification, dispositions,
  unowned-window boundary) for the future exception-diagnostics contract
  document, and the recorded limitations (no return path, no IRQ dispatch,
  twelve origin classes terminally classified as invalid).

A coding agent completing W05 must leave the handoff checklist in
[05-implementation-and-review.md](05-implementation-and-review.md) answerable
without inspecting W05 source code.
