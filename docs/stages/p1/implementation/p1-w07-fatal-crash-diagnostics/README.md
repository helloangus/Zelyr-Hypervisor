# P1-W07 Fatal Crash Diagnostics — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The bounded fatal path required by
[P1-W07](../../plans/p1-w07-fatal-crash-diagnostics.md): one report model
rendering three entry kinds — panic (superseding
[W02](../p1-w02-minimal-rust-el2-runtime/README.md)'s minimal body through
its recorded extension seam), classified exception (consuming
[W05](../p1-w05-el2-exception-entry-baseline/README.md)'s captured frame),
and phase failure (serving [W09](../p1-w09-initialization-sequencing/README.md)'s
`fail_phase` Stage-1/Stable routes) — with fixed field sets, output
ordering, marker classes, transport preference across the MMU transition,
and the recursion/partial-init boundaries.  
**Owner/change context:** P1-W07 implementation handoff.  
**Supersedes:** None. (It *consumes* the W02 extension seam as recorded;
no accepted design is superseded.)

## Purpose and use

This is the implementation-level design for P1-W07. W07 makes every P1
terminal failure routed to it produce one bounded, field-complete,
non-recursive report and makes
the panic route's owner explicit. It deliberately does **not** recover from
invariant violations, store crashes persistently, define Guest fault policy,
add remote logging, or design a production observability pipeline.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-architecture-and-state.md](01-architecture-and-state.md) — the logical
  module map, the three-kind entry model, ownership of every piece of
  fatal-path state (including the guard transfer from W02), the arming
  lifecycle, the transport-preference model across the MMU transition, the
  concurrency model, and the assumed-contract table. Load this first for
  any step.
- [00-implementation-reconciliation.md](00-implementation-reconciliation.md)
  — mandatory preflight corrections to failure-class escalation, safe guard
  storage, and bounded-line sizing; read before implementing any entry.
- [02-code-contracts-report-model.md](02-code-contracts-report-model.md) —
  the report model: field list per kind, availability classes, fixed
  ordering, marker-class prefixes, and the sizing arithmetic.
- [03-code-contracts-fatal-path.md](03-code-contracts-fatal-path.md) —
  contracts for the panic entry (the superseding `p1_panic` body), the
  exception entry, the phase-failure entry, the arming contract, the guard,
  the transport seam, and the terminal behavior.
- [04-implementation-and-review.md](04-implementation-and-review.md) —
  ordered workflow, validation matrix, error/security/observability model,
  and handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W07
plan](../../plans/p1-w07-fatal-crash-diagnostics.md). This document is the
proposed detailed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W07 plan → this
design → Coding Guidelines. In particular:

- The task book requires "panic/crash dump: `ESR_EL2`, `ELR_EL2`,
  `FAR_EL2`, `HPFAR_EL2`, `SPSR_EL2`" diagnostics with P1-V11/P1-V12:
  fatal output includes build, CPU/EL, PC/return, syndrome, fault address,
  phase and useful register context, and panic/fault paths remain bounded,
  useful and non-recursive.
- The plan scopes build/version, CPU/EL, PC/return state, syndrome, fault
  address, startup phase, panic message, and useful general-register
  context across normal panic, EL2 fault, and early-init failure — and
  explicitly excludes recovery of invariant violations, persistent crash
  storage, Guest fault policy, remote logging, and a production
  observability pipeline. A retry, a watchdog, or a persisted dump would
  violate that exclusion; none is designed here.
- The [W02](../p1-w02-minimal-rust-el2-runtime/README.md) runtime is an
  accepted sibling contract consumed through its recorded extension seam
  ([04-code-contracts-panic-identity.md](../p1-w02-minimal-rust-el2-runtime/04-code-contracts-panic-identity.md)
  §5): this design supersedes the report body and takes over the handler
  registration, the single-entry guard, and the bounded-stop discipline.
  [W05](../p1-w05-el2-exception-entry-baseline/README.md) and
  [W09](../p1-w09-initialization-sequencing/README.md) are accepted
  sibling contracts consumed directly (the frame seam and the
  `fail_phase`/arming seams). The
  [W06](../p1-w06-early-console-logging/README.md) channel and
  [W08](../p1-w08-host-stage1-address-space/README.md) mapping are
  parallel designs consumed through recorded seams.
- The P0 contracts are **assumed contracts**:
  [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md)
  (failure-class semantics and the explicit escalation rule; a terminal
  report is an FC-INVARIANT exit, but missing capabilities are first
  FC-UNSUPPORTED and false platform premises first FC-PLATFORM; see the
  [reconciliation](00-implementation-reconciliation.md)), and
  [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)
  (minimal crash-information principle and identity association). W07
  cites their semantics; it does not restate or re-own them.

Classification: the report model, the three entry kinds with their field
sets and ordering, the marker-class prefixes, the superseding panic body,
the exception and phase-failure entries, the arming/readiness contract,
the guard, and the transport preference are **Required**. Symbolization of
reported addresses, additional register captures, a persistent or
structured crash format, and recovery insertion points are **Reserved**
with recorded triggers. Recovery behavior, crash storage, Guest fault
policy, remote logging, telemetry transports, and any second report path
are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Derive required fields from W05 and P0 panic/diagnostic contracts (work seq 1) | [Report model](02-code-contracts-report-model.md) §1–§3; [workflow](04-implementation-and-review.md) step 1 | W07 closure review (W07-DV01) |
| Define fatal-path capture, output ordering and terminal behavior (work seq 2) | [Report model](02-code-contracts-report-model.md) §4–§5; [fatal path](03-code-contracts-fatal-path.md) §1–§4 | P1-V11 (W07-DV02) |
| Integrate pre-MMU and post-MMU diagnostic availability (work seq 3) | [Architecture](01-architecture-and-state.md) §5; [fatal path](03-code-contracts-fatal-path.md) §5 | P1-V11, supports P1-V14 (W07-DV03) |
| Review recursion, partial-init and unsupported-capability cases (work seq 4) | [Architecture](01-architecture-and-state.md) §6; [fatal path](03-code-contracts-fatal-path.md) §6; [workflow](04-implementation-and-review.md) step 5 | P1-V12 (W07-DV04) |
| Define crash-report and panic acceptance evidence (work seq 5) | [Workflow](04-implementation-and-review.md) §3 | P1-V11, P1-V12 (W07-DV05; NC4 execution via W11) |
| Hand off the diagnostic contract to MMU, negative validation and docs (work seq 6) | Downstream handoff below; [workflow](04-implementation-and-review.md) §5 | W07 closure review (W07-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs`
at `4e631ee`): no crash-report code exists anywhere in the tree. The only
fatal-path machinery designed is W02's minimal panic route (bounded report
with identity/message/location through the early writer) and W01's
rejection reporter (pre-transfer, separate path by contract). The W05
frame, W06 channel, and W09 `fail_phase` seams this design consumes exist
as sibling designs, not code. Every executing prerequisite is a contract,
not a present artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Reports contain the required context (P1-V11) | No report model exists | The field sets, availability classes, and ordering of [report model](02-code-contracts-report-model.md) | "Required context" is only reviewable when the field list and its availability rules are fixed before implementation | W07 (this design) | W07-DV02 |
| Bounded fatal path with defined capture and ordering (work seq 2) | W02's minimal body is panic-kind only; no exception or phase-failure report exists | The three entry kinds and terminal discipline of [fatal path](03-code-contracts-fatal-path.md) §1–§4 | Normal panic, EL2 fault, and early-init failure are three different inputs; one model must own all three or each grows its own path | W07 | W07-DV02 |
| Diagnostics remain useful before and after the MMU transition (P1-V14 support, work seq 3) | No mapping exists (W08 parallel); the early writer is PA-based | The transport-preference model of [architecture](01-architecture-and-state.md) §5 | A report path tied to one transport dies at the transition; the preference order and fallback must be fixed in advance | W07 (preference); W06 (channel); W08 (windows); W02 (writer) | W07-DV03; NC5 execution via W11 |
| Panic/fault paths are non-recursive (P1-V12) | W02's guard discipline exists as a contract, not code | The guard transfer and recursion rules of [fatal path](03-code-contracts-fatal-path.md) §6 | Non-recursion must be structural on the one path that runs when everything else is broken | W07 (guard, transferred); W05 (entry-path guard) | W07-DV04 |
| Partial-init and unsupported-capability behavior defined (work seq 4) | Nothing defined | The degradation rules of [report model](02-code-contracts-report-model.md) §3 | Early failures are the ones this package exists for; their reports must not depend on what failed to initialize | W07 | W07-DV04 |
| Panic ownership seam resolved (W02 extension seam) | W02 owns the handler with a recorded replacement trigger | The transfer mechanics of [architecture](01-architecture-and-state.md) §4 | Two owners of `#[panic_handler]` do not link; the transfer must be one recorded change | W07 (new owner); W02 (recorded seam) | W07-DV01 |

No row requires designing recovery, storage, or a later-stage mechanism.

## Resolved design decisions and their authority

1. **One renderer, three entry kinds.** Panic (from `PanicInfo`),
   exception (from W05's `ExceptionFrame` + classification), and
   phase-failure (from W09's `fail_phase` Stage-1/Stable routes) all render
   through one model with one common context block. Rationale: P1-V11's
   field list is one requirement; three divergent renderers would drift.
   Authority: W09 `fail_phase` contract (Stage1/Stable → W07 fatal path);
   W05 routing seam; P1-V11.
2. **Panic-handler ownership moves to W07 through W02's recorded extension
   seam — one recorded design change, not a local edit.** The registration,
   the single-entry guard, and the bounded-stop discipline transfer with
   it; W02's establishment assertion ("panic route ready" at stage 8)
   remains valid because the handler is linked from image start. The
   pre-transfer W01 rejection reporter is untouched (separate path by W01
   §3). Authority: W02
   [04-code-contracts-panic-identity.md](../p1-w02-minimal-rust-el2-runtime/04-code-contracts-panic-identity.md)
   §5 extension seam; W01 §3.
3. **Marker-class prefixes are W07-owned: one panic-class prefix and one
   fatal-class prefix, spanning all eras and transports.** W10's marker
   table names W07 as the fixing authority for both classes; the pre-arm
   summary of [W05](../p1-w05-el2-exception-entry-baseline/README.md)
   reuses the fatal-class prefix so W10's forbidden-class matching covers
   pre-arm and post-arm uniformly. Exact literals are recorded in the
   implementation record before the first verdict-bearing run (the W10
   token precedent) and never adjusted afterwards. Authority: W10 §3.1
   marker-class table; W05 classification contracts §5.
4. **Arming is a readiness declaration, not a code swap.** The report code
   is linked from image start; `fatal_path_ready()` becomes true when the
   `fatal-path` phase body completes its readiness assertions. Pre-arm,
   exception events route via W05's pre-arm summary (W09 H4: a phase's
   route must be established by an earlier phase) while panics already
   render the full report (the panic route is established at phase 2).
   Authority: W09 state machine §4 matrix; W05 routing contracts §4.
5. **Transport preference: W06's channel when available, W02's early
   writer otherwise — one preference, evaluated per report, never
   re-negotiated mid-report.** This is exactly the consumption W02's
   relationship table records ("consumes §3 or W06's channel per its
   design"). Both transports are callable pre- and post-MMU under W08's
   recorded windows, so the preference needs no MMU-state input. Authority:
   W02 §5 relationship table; W06 integration contracts §2/§3; W08
   continuity obligation.
6. **Fields unavailable to a kind are labeled, never fabricated — and the
   panic kind captures what its entry point genuinely has.** The panic
   handler records SP and LR at its own entry (a bounded, honest
   approximation of the panicking call site, labeled as
   handler-entry-captured), so every kind reports PC/return state and an
   entry-state pair; syndrome, fault address, and the full general-register
   set remain unavailable to the panic and phase-failure kinds and print
   the fixed not-available label. Identity degrades to W02's recorded
   literal; the tracker position decodes to `Unknown` with its raw value
   rather than panicking (W09 §2 decode rule). Authority: W02
   BuildIdentity rule; W09 `LifecyclePosition` contract; P1-V11's uniform
   required-context list; honesty rules of the Plan Agent guide.
7. **Bounded by construction, guarded by one flag.** Fixed-capacity line
   buffers with truncation (never panic), no allocation, no unwraps, no
   unwinding, fixed line count; a second fatal entry while the guard is
   held executes the silent bounded stop. The guard is the single
   fatal-path synchronization story, transferred from W02's panic-entry
   guard (decision 2) and coordinated with W05's entry-path guard through
   the contracted call direction (W05 sets its own entry guard before its
   router, then every W07 entry acquires W07's distinct report guard;
   see the [reconciliation](00-implementation-reconciliation.md)).
   Authority: W02 guard contract; W05 state machine R1; P1-V12.
8. **No symbolization, no storage: addresses print as raw fixed-width hex;
   nothing is written anywhere but the selected transport.** Symbolization
   needs host-side tooling (Reserved trigger, `llvm-tools` class per the
   P0-W02 component reservation); a persistent format is an ABI decision
   outside P1. Authority: plan out-of-scope list; P0-W02 component
   reservation; stage-local design freedom with recorded rationale.

## Work breakdown and loading order

1. Load [01-architecture-and-state.md](01-architecture-and-state.md) for
   the entry model, ownership, arming lifecycle, and assumed contracts.
   Every implementation step depends on it.
2. Load [02-code-contracts-report-model.md](02-code-contracts-report-model.md)
   for the field sets, ordering, and prefixes, and
   [03-code-contracts-fatal-path.md](03-code-contracts-fatal-path.md) for
   the entry contracts and boundaries.
3. Execute the steps in the order given in
   [04-implementation-and-review.md](04-implementation-and-review.md):
   prerequisite confirmation, report model, entries and arming, transport
   and transition wiring, boundary reviews, evidence and handoff.
4. Record implementation decisions and deviations in
   `../p1-w07-fatal-crash-diagnostics-record.md` when implementation
   begins, and validation commands, environments, and outcomes in
   `../../verification/p1-w07-fatal-crash-diagnostics-verification.md`
   when evidence exists. Neither file may exist yet, and neither this
   design nor a record may claim W07 complete.

## Explicitly excluded interfaces

No recovery, retry, watchdog, or degraded-continue behavior; no persistent
or structured crash format, storage device, or snapshot; no Guest fault
policy (P0-W14's guest-caused class is untouched); no remote logging or
telemetry transport; no symbolizer or host-side decoding; no second panic
handler or second report path; no public ABI, wire format, or persistent
layout — the report is console text matched by fixed token classes, not a
machine interface. W07 configures no hardware and reads only the
registers its contracts name (via W05's boundary for exception state and
W02's writer for output); any additional register read is a scope
violation.

## Downstream handoff

- **[P1-W08](../p1-w08-host-stage1-address-space/README.md)** receives the
  diagnostic-continuity obligation its plan names (work seq 4): the two
  transport windows must be mapped before MMU enablement; its premise and
  step failures report through the phase-failure entry with
  `InitPhase::Stage1` attribution and a step reason string; the full
  report must remain producible across the transition (W09 matrix
  `stage1` row).
- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  `fatal-path` phase body (the mechanism entry its `fatal_path_step`
  adapter calls — the arming contract), the phase-failure entry its
  `fail_phase` Stage1/Stable arms call, and the misuse-routing
  confirmation its matrix needs.
- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives the
  concrete report field list and marker classes its scenario expectations
  reference (NC3/NC5/NC6 exception reports; NC4 panic report), and the
  guard/non-recursion properties its terminal-outcome checks rely on.
- **[P1-W10](../p1-w10-qemu-boot-regression/README.md)** receives the
  panic-class and fatal-class prefixes as the forbidden marker classes of
  a passing boot (its §3.1 table); exact literals arrive via the
  implementation record before the first verdict-bearing run.
- **[P1-W05](../p1-w05-el2-exception-entry-baseline/README.md)** has its
  deferred seam resolved by this design: `report_fatal_exception` and the
  readiness declaration are fixed here; W05's armed branch consumes them
  as recorded.
- **[P1-W06](../p1-w06-early-console-logging/README.md)** has its
  transport role confirmed (preferred post-availability transport, with
  its §2 producer obligations binding the renderer).
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  report model, kind/field availability, marker classes, and the recorded
  limitations (no recovery, no storage, no symbolization, silent
  second-entry stop) for the exception-diagnostics contract document.

A coding agent completing W07 must leave the handoff checklist in
[04-implementation-and-review.md](04-implementation-and-review.md)
answerable without inspecting W07 source code.
