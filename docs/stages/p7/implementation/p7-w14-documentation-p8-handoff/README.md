# P7-W14 Documentation and P8 Handoff — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The factual scheduler behavior documentation, evidence indexing,
closure review, and bounded P8 handoff required by
[P7-W14](../../plans/p7-w14-documentation-p8-handoff.md).  
**Owner/change context:** P7-W14 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W14, the P7 closure package.
The plan requires factual scheduler behavior documentation, an evidence
index, a reconciliation of validation/exit criteria/limits/unsafe/dependency
status, and a bounded P8 handoff (P7-V30) — without creating evidence before
execution, declaring P7 complete, freezing the P8 machine ABI, or defining
Linux boot. The design fixes three authoritative artifact groups, their
required content and locations, the ordered closure workflow, and the
P8-consumer mapping. It produces no facts itself: every statement it
governs must trace to an implementation or verification record that already
exists when W14 runs.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
[the closure and handoff workflow](01-closure-and-handoff-workflow.md) for
the ordered steps and validation matrix — this is the entire supporting-file
set; the package is documentation work and needs no further split. Before
executing anything, the agent must also follow the Coding Guidelines
preflight, including the repository `AGENTS.md`, documentation index, ADR
baseline, P7 task book, P7-W14 plan, and the P7-W01 recorded input boundary.
This document authorizes no hypervisor source change.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W14 plan → this
design → Coding Guidelines. In particular:

- Task book §7 fixes what P8 may rely on: evidenced scheduler-controlled
  entry, lifecycle/placement/blocking/pause behavior, preemption/context
  isolation, SMP M:N progress, scheduler Validation Guest coverage, and
  scheduler observability/regression entry points. P8 still owns machine
  ABI, guest SMP presentation, and Linux boot; the handoff record must not
  pre-empt them (ADR-024/ADR-040 versioning and the machine contract remain
  P8-W02 scope).
- ADR-049 makes layered validation the closure basis: the evidence index
  maps every task-book validation row P7-V01–V30 to actual evidence or an
  explicit not-run/blocked status. Planned evidence is never presented as
  evidence (the same rule P8-V26 later enforces on P8).
- ADR-006 and P0 unsafe/dependency governance apply: the unsafe delta and
  dependency status are factual inventories over the P7 implementation
  records, not new policy.
- The plan's out-of-scope list is binding: no evidence creation, no
  completion claim, no machine-ABI freeze, no Linux boot design.

Classification:

- **Required:** the three artifact groups with their fixed locations and
  required content; the fact/limitation/planned classification discipline;
  the complete P7-V01–V30 evidence index; the unsafe/dependency inventory;
  the P8 consumer mapping and handoff record; the closure workflow and
  review.
- **Reserved:** a re-organization of P7 factual documentation into a
  long-term documentation taxonomy if a later stage defines one (semantic
  ownership moves, paths may change); machine-readable behavior
  specifications. Each requires a new design decision.
- **Out of Scope:** creating or altering verification evidence; hypervisor
  source changes; P8 mechanisms (machine contract, DTB, PSCI, Linux boot,
  guest SMP presentation); management/policy design; completion claims.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Collect W12 automation and W13 baseline inputs with implementation and verification records | [Workflow](01-closure-and-handoff-workflow.md) step 1 | W14-DV01 |
| Produce behavior documents distinguishing implemented facts from planned work | [Workflow](01-closure-and-handoff-workflow.md) step 2; behavior record content in §2 there | P7-V30 (W14-DV02) |
| Reconcile validation, exit criteria, limitations, unsafe/dependency changes, unresolved conflicts | [Workflow](01-closure-and-handoff-workflow.md) step 3 | P7-V30 (W14-DV03) |
| Review P8 handoff against the task-book boundary and ADR constraints | [Workflow](01-closure-and-handoff-workflow.md) step 4; mapping below | P7-V30 (W14-DV04) |
| Record closure readiness or outstanding evidence without an unsupported completion claim | [Workflow](01-closure-and-handoff-workflow.md) step 5 | P7-V30 (W14-DV05) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs`
at 4e631ee): the repository is a P0 documentation scaffold — no workspace, no
sources, no implementation records, no verification evidence;
`docs/stages/p7/implementation/` contains only the stage index README and
`docs/stages/p7/verification/` only a `.gitkeep`. Sibling P7 designs
W01–W13 are being prepared in parallel; P7 implementation has not begun, so
every input W14 will close over is a future record created by earlier
packages. The P8 stage exists as planning documents only; its task book is a
named consumer, not a source of P7 content.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Behavior documentation distinguishes fact from plan | No behavior documentation exists anywhere | The behavior record artifact group with the three-way classification rule | P7-V30 requires reviewable behavior docs; without a fact/plan discipline they would be fiction | W14 (this design); facts from W02–W13 records | W14-DV02 |
| Evidence index covers closure | No verification content exists (`.gitkeep` only) | Evidence index mapping P7-V01–V30 to records with status | Exit review requires every validation row to be locatable and statused | W14; evidence from W01–W13 verification records | W14-DV03 |
| W12/W13 inputs collected | Neither package has run | Evidenced W12 matrix results and W13 baseline record, consumed per plan work sequence | Closure consumes the regression and performance inputs the task book names | P7-W12/W13; W14 consumes | W14-DV01 |
| P8 handoff is bounded and reviewable | P8 planning exists; no P7 facts to hand | The consumer mapping below, instantiated with evidenced facts in the handoff record | P8-W01/P8-W20 must be able to consume P7 without inheriting unproven or out-of-boundary claims | W14 (mapping); P8 task book §7 (boundary) | W14-DV04 |
| No unsupported completion claim | n/a — discipline | Closure-readiness statement listing outstanding evidence explicitly | P7 closes only with evidence for P7-V01–V30 (task book §7) | W14 | W14-DV05 |

No row requires inventing a mechanism; the runtime/documentation inputs are
blocking-if-absent per the P7-W01 boundary, and an absent input becomes an
explicit outstanding item in the closure record rather than a silent gap.

## Resolved design decisions and their authority

1. **Compact two-file design.** The plan is single-threaded documentation
   work with no implementation modes to separate, so one supporting file
   beside this entry is sufficient (checklist: do not split a short design
   merely to create folders).
2. **Three authoritative artifact groups with fixed locations:** the
   scheduler behavior record
   (`docs/stages/p7/implementation/p7-w14-scheduler-behavior.md`), the
   evidence index
   (`docs/stages/p7/verification/p7-w14-evidence-index.md`), and the
   closeout/handoff record
   (`docs/stages/p7/implementation/p7-w14-documentation-p8-handoff-record.md`).
   Rationale: the stage layout separates implementation traceability from
   verification; behavior facts belong with implementation traceability,
   evidence indexing with verification, and the handoff with the package
   record. All three are created only when W14 work starts.
3. **Three-way classification discipline in the behavior record:** every
   statement is either an implemented fact with a linked evidence location,
   a recorded limitation, or planned-but-not-implemented work. Rationale:
   the plan's second work item requires exactly this distinction, and
   P8-V26 will apply the same rule to P8's own closeout.
4. **The evidence index covers P7-V01–V30 completely**, each row carrying
   status passed / failed / blocked / not run with a pointer to the record
   that holds the evidence or the reason none exists. Rationale: the task
   book's exit criteria are defined over the full P7-V01–V30 set.
5. **The P8 consumer mapping is fixed here and instantiated in the handoff
   record** (table below). Rationale: the plan requires a bounded handoff;
   fixing the mapping in the design lets the closure review (step 4) check
   the handoff against a declared boundary rather than an ad-hoc list.
6. **Closure is recorded as readiness, never as completion.** The record
   states either "closure review passes with evidence for all gates" or the
   explicit list of outstanding/failed items. Rationale: plan out-of-scope
   ("declaring P7 complete") and task book §7.
7. **Unsafe delta and dependency status are inventories, not policy.** They
   aggregate what P7 implementation records already report (new `unsafe`,
   new dependencies, ABI/public-API changes) and link the governing P0
   records. Rationale: P0 governance owns policy; W14 owns factual closure.
8. **Documentation-location reservation.** If a later documentation baseline
   re-homes P7 factual documents (as P0-W05-style taxonomies evolve), the
   behavior record's semantic ownership moves with it and links are updated;
   the P8 handoff record must remain reachable through the stage
   implementation index. Rationale: prevents path churn from breaking the
   handoff.

## Work breakdown and loading order

1. Read [the closure and handoff workflow](01-closure-and-handoff-workflow.md)
   once; it contains the artifact-group content requirements (§2), the
   ordered steps (§3), the validation matrix (§4), and the handoff checklist
   (§5).
2. Execute the steps in order: collect W12/W13 and all-package records,
   write the behavior record, build the evidence index and reconcile exit
   criteria, review the P8 handoff against the mapping and boundary, record
   closure readiness.
3. Keep every factual statement linked to its evidence; a statement without
   a link is rewritten as planned work or removed. Neither this design nor
   any record may claim P7 complete; closure is claimed only in the
   verification record, and only for gates with real evidence.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
Guest-visible interface, scheduler mechanism, script, or CI workflow is
designed or authorized by W14. W14 writes documentation only, in the three
artifact groups of decision 2 plus truthful status rows in the stage
implementation index if it maintains one. It does not modify task book,
plans, ADRs, or any other stage's documents; unresolved conflicts it finds
are recorded as issues, never absorbed by editing the governing source.

## Downstream handoff — P8 consumer mapping

P8 consumes P7 only through evidenced facts mediated by the P8 entry review
(P8-W01) and closure (P8-W20). The mapping below names, for each P7
deliverable, the P8 packages that may rely on it and the boundary of that
reliance; the handoff record instantiates this table with links and the
current evidence status of each row.

| P7 deliverable (producer) | P8 consumers | What they may rely on — evidenced facts only |
|---|---|---|
| Scheduler-controlled entry and lifecycle semantics (P7-W02) | P8-W01, P8-W05, P8-W06, P8-W11 | Guest entry/exit is scheduler-controlled; vCPU lifecycle states and transition rules hold as evidenced |
| Placement, pinning, affinity, dedicated/shared semantics (P7-W03) | P8-W01, P8-W10, P8-W11 | Linux-vCPU placement obeys evidenced configuration semantics; no silent fallback |
| Preemption and context isolation (P7-W04) | P8-W01, P8-W05, P8-W08, P8-W11 | CPU-bound Guests are preempted; switch isolation preserves context, address space, timer, vIRQ, events |
| M:N and multi-VM progress with no-starvation fairness (P7-W05) | P8-W01, P8-W11 | M>N vCPU configurations make progress; fairness is bounded to the evidenced no-starvation claim — no proportional-fairness or performance claim transfers |
| Block/wakeup semantics (P7-W06) | P8-W01, P8-W08, P8-W11 | WFI/WFE blocking releases pCPUs; timer/vIRQ/Notification wakeup works without lost/duplicate wakeup as evidenced |
| Pause/stop/fault containment (P7-W07) | P8-W01, P8-W13, P8-W18 | Pause/resume and stop/fault behave as evidenced; Guest faults are contained to their VM context |
| SMP placement, remote reschedule, idle behavior (P7-W08) | P8-W01, P8-W10, P8-W11 | Cross-pCPU scheduling, remote reschedule, and idle behavior hold as evidenced |
| Accounting, trace, switch reasons, diagnostics (P7-W09) | P8-W01, P8-W13, P8-W17, P8-W20 | Scheduler observability surfaces exist with the evidenced fields; P8-W17 consumes the surface, not P7 numbers |
| Validation Guest scheduler suite (P7-W10) | P8-W01, P8-W19 | The suite is the retained mechanism coverage for scheduler interaction; P8-W19 keeps it alongside Linux (dual track) |
| Stress and invariant evidence incl. limits (P7-W11) | P8-W01, P8-W10, P8-W18 | Stability/containment evidence and its declared non-coverage; supports P8 stability and isolation scenario design, transfers no formal guarantee |
| Automated QEMU regression entry points (P7-W12) | P8-W01, P8-W16, P8-W19 | The matrix, case declarations, result taxonomy, and evidence locations are reusable entry points; P8 composes its own Linux matrix and inherits method and limits, not results |
| Performance baseline and method incl. no-KPI rules (P7-W13) | P8-W01, P8-W17 | The method, environment declarations, and limitations as prior art for the Linux non-KPI baseline; no P7 number is a P8 target or gate |
| P7 closeout, evidence index, limits (P7-W14 itself) | P8-W01, P8-W20 | The reviewable input set for entry reconciliation and closure; outstanding items transfer as named risks |

Boundary statements the handoff record must carry: no machine-ABI, DTB,
PSCI, console, or Linux-boot content is handed off (P8-W02–W06 own those);
nothing in the handoff freezes a P8 decision; every row above is void for a
consumer unless the linked evidence exists at handoff time; QEMU-environment
evidence never transfers as hardware semantics (ADR-003/ADR-045).

Mediation rule: the P8 plan index declares four direct P7 consumers —
P8-W01 (P0–P7 factual handoffs), P8-W05 (P4–P7 execution facts), P8-W08
(P6–P7 timer facts), and P8-W11 (P7 scheduler facts). Every other row in the
table is a mediated consumer that inherits these facts along P8's own
dependency graph (P8 task book §4); the handoff record must preserve that
mediation and must not present mediated rows as direct inputs.
