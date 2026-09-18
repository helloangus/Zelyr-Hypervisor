# P5-W01 Entry Contract Reconciliation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewable P5 entry boundary required by
[P5-W01](../../plans/p5-w01-entry-contract-reconciliation.md): reconciliation
of the P0–P4 handoff and contract inputs, routing of future P5 factual
artifacts, and classification of missing or conflicting prerequisites.  
**Owner/change context:** P5-W01 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W01. It converts the bounded
work-package plan into a documentation review with a fixed reconciliation
ledger, a fixed gap-classification vocabulary, and governed output locations.
It deliberately does **not** design HVC/ABI values, Guest-data mechanisms,
object APIs, or capability semantics; it does not repair any upstream stage;
and it does not claim that any upstream stage closed. Those remain with
[P5-W02](../p5-w02-hypercall-abi-error-boundary/README.md) (P5-W02),
[P5-W03](../p5-w03-guest-data-safety/README.md) (P5-W03),
[P5-W04](../p5-w04-handle-lifecycle-type-safety/README.md) (P5-W04), and
[P5-W05](../p5-w05-capability-rights-bootstrap-revocation/README.md)
(P5-W05) respectively.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). Because W01
is a documentation-review package, the Coding Guidelines apply to *behavior*
(no code, no ABI, no upstream repair), not to Rust authorship; the review
procedure itself is fixed in the
[reconciliation ledger](01-reconciliation-ledger.md) and the
[workflow, validation, and handoff](02-workflow-validation-handoff.md) file.
Load `01` to perform or review the reconciliation itself; load `02` to run the
steps, judge acceptance, or hand off. Before editing, the agent must also have
read the repository `AGENTS.md`, [documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P5 task book](../../task-book-v0.1.md), and the
[P5-W01 plan](../../plans/p5-w01-entry-contract-reconciliation.md). This
document is a proposed design; it contains no implementation or validation
claim and no completion statement.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → P5-W01 plan → this
design. Binding constraints observed from those sources:

- The task book requires P5 implementation to begin only after a review
  identifies evidence for, or records the absence of, every P0–P4 input it
  lists; absent or contradictory inputs are blocked prerequisites or
  `Architecture Change Request` / `ADR Required` issues, never silently
  repaired upstream or silently re-chosen here (task book §2, §8).
- The plan scopes W01 to inspection, reconciliation, routing, and
  classification. Its out-of-scope list (repairing an upstream stage,
  designing HVC/ABI values or object APIs, freezing an ABI, implementing P5
  mechanisms, claiming an upstream closure) is restated as this design's
  Out of Scope class below.
- ADR-007 (Guest untrusted), ADR-013 (capability/handle + rights +
  generation; no `vm_id == 0` privilege), ADR-040 (independent versioning of
  management/machine contracts), ADR-048/ADR-049 (structured observability,
  layered validation), and the ADR §19 invariants bound every statement every
  P5 package may make. W01 fixes how those constraints are *referenced* by
  later packages; it does not restate or reinterpret them.

Classification used throughout this design:

- **Required** — the reconciliation ledger and its five fact domains; the gap
  classification vocabulary; the artifact routing table; the per-consumer
  input assignment; the entry-review record and its evidence; the inherited
  open-item register (including P2-ACR-01).
- **Reserved** — the *content* of the future factual ABI and security
  artifacts (owned by the implementing packages and W10 closeout); any change
  to an upstream contract that reconciliation reveals to be necessary; the
  re-run of evidence-level reconciliation when P0–P4 verification records
  change after P5 entry.
- **Out of Scope** — any HVC register, call-number, error, handle, rights, or
  object-table value; any Rust module, type, or function; any repair or
  redesign of P0–P4 scope; any completion claim for P0–P4 or P5; the P5
  stage implementation index (coordinator-owned) and stage verification
  records.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect ADR, task book, plan index, P0–P4 records | [workflow](02-workflow-validation-handoff.md) step 1 | W01-DV01 |
| Reconcile P4 Guest-EL1, HVC/exception, Stage-2, Guest-memory, fault, diagnostics, Validation Guest, regression facts | [ledger](01-reconciliation-ledger.md) §2 (FD-1, FD-2, FD-3) | P5-V01 (W01-DV02) |
| Identify P2 ownership/address and P3 synchronization/pCPU constraints for later packages | [ledger](01-reconciliation-ledger.md) §2 (FD-2, FD-5) | P5-V01 (W01-DV03) |
| Route future factual ABI, security, design, implementation, verification artifacts | [ledger](01-reconciliation-ledger.md) §4 | P5-V01 (W01-DV04) |
| Review and classify missing/ambiguous/contradictory inputs without silent redesign | [ledger](01-reconciliation-ledger.md) §5; [workflow](02-workflow-validation-handoff.md) step 5 | P5-V01 (W01-DV05) |
| Record the entry review and hand compatible inputs and blocks to W02–W10 | [workflow](02-workflow-validation-handoff.md) steps 4–6 and handoff checklist | P5-V01 (W01-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p5-implementation-designs`):
the repository is a **P0 documentation scaffold**. There is no Cargo
workspace, no Rust source file, no `hypervisor/` content (`hypervisor/src/`
holds only a `.gitkeep` marker), and no board, SoC, or guest implementation.
Every `docs/stages/p{0..8}/verification/` directory holds only a `.gitkeep`
except P0, whose single verification record covers P0-W01 repository-baseline
documentation work. All P0–P4 work packages exist as **plans**, not as
implementation or verification records. Consequently every "P0–P4 fact" named
by the P5 task book is currently an **assumed contract** recorded in a plan,
not observed evidence; the only observable P0 artifacts are governance
documents (ADR, guides, plans, the P0-W01 repository-baseline record, and the
P0-W02 design set).

This state does not block W01, because W01's own plan gates only a
documentation review (P5-V01: "this is a documentation review, not runtime
evidence"). It does force one structural decision, recorded below: W01
operates in two modes — **contract-level reconciliation** (possible now,
against plan text) and **evidence-level reconciliation** (runs when P0–P4
implementation/verification records exist, and again at any later P5 entry).
The task book's entry condition ("P5 implementation may begin only after a P4
review identifies evidence for, or records absence of, all of these inputs")
is satisfied by the evidence-level mode, not by this planning-time pass.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Reviewable P5 starting boundary from evidenced P0–P4 handoffs | No P5 entry-review artifact exists; P0–P4 verification records absent except P0-W01 | Entry-review record at `../p5-w01-entry-contract-reconciliation-record.md` (created when W01 starts), structured by the ledger in file 01 | A boundary that cannot be pointed to cannot gate W02–W10 | W01 (this design) | W01-DV01/DV06 linked review |
| Every prerequisite named with evidence location or absence | Upstream evidence absent; upstream plan contracts present and locatable | Ledger §2 rows carrying source plan path, evidence status, and consumer per fact | "Location or absence" must be stated per input, not averaged | W01 | W01-DV02/DV03 |
| No P5 package assumes undocumented upstream behavior | P5 plans route inputs by name only; no P5 design existed at planning time | Assumed-contract model: each P5 design cites upstream plan paths as assumed contracts with explicit failure boundaries (ledger §3) | Designs written from names would invent crates/modules/targets the scaffold forbids | W01 rule; W02–W10 comply | W01-DV05; per-package design review |
| ABI/security routing for future factual artifacts | `docs/abi/` and `docs/security/` are index stubs; task book makes the artifacts implementation-stage deliverables | Routing table (ledger §4) fixing governed locations and the approval precondition | Routing decided during implementation would be invented under time pressure | W01 routing; W02–W05 produce; W10 publishes | W01-DV04 |
| Missing/ambiguous/contradictory inputs classified without silent redesign | P2-ACR-01 is recorded unresolved upstream; no P4 deviation records exist yet | Gap-classification vocabulary and inherited open-item register (ledger §5) | Classification needs a fixed vocabulary to be reviewable and non-silent | W01 vocabulary; conflicts escalated per ADR rules | W01-DV05 |
| Handoff to W02–W10 | Plans name consumers but no reviewed input set exists | Per-consumer input assignment (ledger §2 consumer column) and handoff checklist (file 02 §4) | Consumers must be able to act without re-deriving the entry boundary | W01 | W01-DV06 |

No row above requires inventing a remote, license, crate, target, or runtime
policy, so no decision blocker is outstanding for this design. The absence of
P0–P4 *evidence* is a recorded condition of the entry review, not a blocker to
writing the review procedure.

## Resolved design decisions and their authority

1. **Two reconciliation modes.** W01 performs contract-level reconciliation
   now (against the P0–P4 plan and handoff-contract texts, which are tracked
   and locatable) and defines evidence-level reconciliation as the gate that
   runs when upstream implementation/verification records exist. Rationale:
   the task book's entry condition speaks of *evidence identified or absence
   recorded*; in the current scaffold the honest record of absence is itself
   the required output, and contract-level compatibility is the only review
   that can be performed truthfully today. Authority: task book §2; plan
   acceptance wording ("documentation review, not runtime evidence");
   observable repository state.
2. **Five fact domains.** The reconciliation is organized as FD-1 execution
   and trap boundary, FD-2 Guest-data and memory boundary, FD-3 object and
   lifecycle foundation, FD-4 authority and security boundary, FD-5
   engineering and evidence boundary. Rationale: the P5 packages consume
   upstream inputs through exactly these seams (task book §2 table); a
   domain-per-seam ledger makes the consumer mapping one-to-one reviewable.
   Authority: task book §2; P5 plan index; design-owned organization.
3. **Assumed-contract model for P5 designs.** Every P5 package design treats
   an upstream input as an *assumed contract* cited by upstream plan path,
   with an explicit failure boundary stating what the package must do if the
   implemented upstream delivers differently (block, or raise
   `Architecture Change Request` / `ADR Required`). Rationale: P0–P4 are
   unimplemented; inferring crates, module trees, APIs, or behavior from
   names is prohibited by `AGENTS.md`. Authority: `AGENTS.md`; task book §2,
   §8; this design owns the model's uniform form.
4. **Gap-classification vocabulary.** Exactly four labels: `Blocked
   Prerequisite` (input absent or not evidenced at entry),
   `Contract Conflict` (upstream contract contradicts another; record and
   stop the affected decision), `ADR Required` (resolution would alter an
   accepted architecture decision), `Documentation Gap` (input exists but is
   under-specified for its P5 consumer). Rationale: the task book uses
   blocked-prerequisite and ACR/ADR language (§2, §8) and P5-V01 requires
   "evidence location or absence" per input; a closed four-label set keeps
   the review binary and auditable. Authority: task book §2/§8; design-owned
   vocabulary.
5. **Caller identity vocabulary for all P5 packages.** "Caller" means the
   Guest vCPU execution context of a specific VM at trap time; caller
   identity is recorded per request and is never authority. Fixed here
   because W02–W06 must use one term consistently and the task book reserves
   the right to change pCPU placement later (no permanent caller↔pCPU
   binding). Authority: ADR-007, ADR-013, ADR-051; task book §1 Reserved;
   design-owned wording.
6. **Artifact routing.** Future factual HVC ABI artifacts go under
   `docs/abi/` per [`abi/README.md`](../../../../abi/README.md); security
   invariant wording and reviews under `docs/security/` per
   [`security/README.md`](../../../../security/README.md); approved detailed
   designs and implementation records under
   [`../README.md`](../README.md); evidence under `../../verification/`.
   Exact file names of ABI/security artifacts are chosen by their producing
   packages and indexed by W10 closeout, not by this design. Rationale: the
   task book §1 and the stage implementation index already assign these
   homes; W01 only makes the routing explicit per consumer. Authority: task
   book §1, §3; `docs/README.md` layout rules.
7. **Inherited open items remain visible, unresolved.** P2-ACR-01 (recorded
   unresolved by the P2 plans, constraining P4 memory-object planning) is
   carried into the P5 open-item register because FD-2 consumers depend on
   the memory-ownership foundation it may affect. P4 architecture-deviation
   or ACR records are to be checked when P4 records exist; none are assumed.
   Rationale: `AGENTS.md` requires conflicts to be preserved, not silently
   resolved; P5 must not absorb an upstream architectural decision.
   Authority: P2-W10 / P2-W03 plan text; `AGENTS.md` scope rules.

## Work breakdown and loading order

1. Read [the reconciliation ledger](01-reconciliation-ledger.md) §1–§3 to
   understand the fact domains, the per-fact input rows, and the
   assumed-contract failure-boundary form that W02–W10 designs must repeat.
2. Read [the ledger](01-reconciliation-ledger.md) §4–§5 for artifact routing
   and the gap-classification vocabulary before classifying any finding.
3. Execute the review in the order given in
   [workflow, validation, and handoff](02-workflow-validation-handoff.md) §2:
   inventory sources, reconcile each domain, route artifacts, classify gaps,
   record, and hand off.
4. Store the review's commands, citations, and per-row findings in
   `../../verification/p5-w01-entry-contract-reconciliation-verification.md`,
   and record decisions taken and deviations in
   `../p5-w01-entry-contract-reconciliation-record.md` only when W01 work
   starts. Neither this design nor any record may claim P5-W01 or P5
   complete; completion evidence belongs to `../../verification/` and the
   stage gate (P5-V01 via W10 closeout).

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, or public API; no HVC register,
immediate, call number, error number, handle layout, rights bit, or object
representation; no ABI or wire format; no test command spelling; no repair,
re-scoping, or reinterpretation of any P0–P4 contract; no completion or
evidence claim for any stage. The only artifacts W01 owns are the entry-review
record and its verification record, both structured by the ledger. If
reconciliation appears to require any of the excluded items, that need is
classified per ledger §5 and routed to the owning package or recorded as an
architecture issue — never implemented inside W01.

## Downstream handoff

Per the [P5 plan index](../../plans/README.md) consumer map:

- **W02** ([hypercall ABI and error boundary](../p5-w02-hypercall-abi-error-boundary/README.md))
  receives FD-1 (trap recognition and EL2 recovery facts), FD-4 (authority
  and guest-fault/invariant classification vocabulary), and FD-5 (host-test,
  unsafe, and documentation-governance inputs), plus the ABI/security
  artifact routing of ledger §4.
- **W03** ([guest data safety](../p5-w03-guest-data-safety/README.md))
  receives FD-2 (P2 protected-range/ownership/checked-address constraints and
  P4 Stage-2 capability and Guest-memory facts) and the P2-ACR-01 visibility
  obligation.
- **W04** ([handle lifecycle and type safety](../p5-w04-handle-lifecycle-type-safety/README.md))
  receives FD-3 (P4 object/lifecycle facts that the object-reference model
  must not redesign) and FD-4 (ADR-013 constraints as assumed contracts).
- **W05** ([capability, rights, bootstrap, revocation](../p5-w05-capability-rights-bootstrap-revocation/README.md))
  receives FD-4 plus the W02 error-routing and W04 identity boundaries as
  declared sibling dependencies, under the same assumed-contract model.
- **W06–W10** receive the entry boundary through the plans' prerequisite
  edges; W10 additionally receives the routing and open-item registers for
  closeout. Every consumer receives the same rule: an upstream input that
  fails its assumed contract at implementation time is a blocked
  prerequisite or a recorded conflict, never a local redesign.
