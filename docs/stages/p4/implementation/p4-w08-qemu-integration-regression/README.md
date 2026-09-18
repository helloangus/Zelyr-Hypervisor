# P4-W08 QEMU Integration Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The automated QEMU `virt` integration regression for P4: the
scenario matrix, determinate pass/fail conditions, the automation contract
over the P0 runner boundary, and the evidence layout required by
[P4-W08](../../plans/p4-w08-qemu-integration-regression.md) (P4-K01–K05).  
**Owner/change context:** P4-W08 implementation handoff; this design owns the
P4 scenario matrix (SM-series), the verdict taxonomy and rules, and the
evidence layout. It designs no hypervisor or Guest mechanism.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W08. It converts the bounded
work-package plan into the validation scenario matrix with per-scenario
expected observables and proof boundaries, the automation artifact contracts
(manifest, entry point, collection, verdicts), and the evidence layout — it
defines what the regression must determine, not results; no run is claimed
anywhere in this design. It deliberately does **not** implement Guest or
hypervisor mechanisms (its input packages W05–W07 own those seams), build the
QEMU runner itself (the [P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md)
entry is extended, never duplicated), treat QEMU behavior as an architecture
contract, add CI provider policy (P0 CI package), prove real-hardware
behavior, or cover Linux Guests, performance benchmarking, or Orange Pi
runtime validation.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scope-and-foundations.md](01-scope-and-foundations.md) — scope
  classification, assumed upstream contracts with failure boundaries,
  authority analysis, and resolved design decisions (D1–D8). Load first.
- [02-scenario-matrix.md](02-scenario-matrix.md) — the SM-series scenario
  matrix: input/preconditions, expected observable, pass condition, and what
  each scenario proves and does not prove. Load for scenario work.
- [03-automation-contract.md](03-automation-contract.md) — the automation
  artifact groups: manifest schema, single entry point, collection rules,
  verdict taxonomy and determination rules, timeout handling, and the
  evidence layout. Load for the automation work area.
- [04-implementation-workflow.md](04-implementation-workflow.md) — ordered
  implementation steps with acceptance and failure handling.
- [05-validation-and-handoff.md](05-validation-and-handoff.md) — the
  validation matrix (P4-V13–P4-V15), the error/security/observability model,
  and the handoff checklist.

Before editing, the agent must also follow the reading order in the
[plan index](../../plans/README.md) (ADR, P4 task book, the P4-W08 plan, and
the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry review
result). This document is a proposed design; it contains no implementation
or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W08 plan](../../plans/p4-w08-qemu-integration-regression.md) → this
design → Coding Guidelines. Binding constraints include:

- ADR-003 and ADR-052: QEMU `virt` is the reference/CI platform and QEMU is
  never the architecture definition; Core must not branch on QEMU names.
  Regression expected values come from Guest markers and Hypervisor run
  records (host-authored contracts), never from QEMU-internal behavior.
- ADR-049: QEMU integration is one layer of the validation strategy; a QEMU
  pass proves the stated reference environment only (task book P4-K scope:
  "Passing proves P4 behavior in the stated QEMU environment only; it does
  not prove real-hardware correctness").
- P0-W09 plan (W01 row R06): exactly one QEMU automation entry exists; P4
  extends it with parameters within its reserved parameter surface; no
  second, drifting QEMU command path may be created.
- P1-W10 plan (W01 row R06): boot regression conventions — bounded markers,
  timeout/exit semantics, panic detection, evidence retention, no
  output-order-only criteria — are the pattern this design inherits at P4
  scope.
- W05 §6: marker grammar changes require W08 agreement; W08 must not redefine
  or weaken the VG/IS expectations it consumes.
- W07 D6/D7: the P4-RR v1 record and the declared repeat minimums are the
  machine-readable repeat surface; W08 parses and applies them as given.
- Task book §1 Out of scope for W08: treating QEMU as a complete hardware
  proof, Orange Pi runtime validation, CI policy changes, Linux regression,
  performance benchmarking, and implementing Guest mechanisms its input
  packages have not established.

Classification: the scenario matrix, verdict rules, manifest schema, entry
point contract, and evidence layout ([02](02-scenario-matrix.md),
[03](03-automation-contract.md)) are **Required** for P4-K01–K05. Environment
matrices beyond the single declared reference configuration, CI provider
wiring, real-hardware runs, Linux Guests, and performance timing are
**Reserved** with recorded re-entry points. Hypervisor/Guest mechanism
implementation, QEMU-emulator feature development, hardware proof claims,
and completion claims of any kind are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-K01 build Hypervisor + Guest, prepare image, boot QEMU | [automation contract](03-automation-contract.md) §2–§3 | P4-V13 (determinate positive result) |
| P4-K02 collect expected serial/telemetry markers | [automation contract](03-automation-contract.md) §3–§4 | P4-V13/V14 |
| P4-K03 positive EL1-entry determination | [scenario matrix](02-scenario-matrix.md) SM-01 | P4-V13 |
| P4-K04 translation/permission fault + survival/stop determination | [scenario matrix](02-scenario-matrix.md) SM-02–SM-06 | P4-V14 |
| P4-K05 repeated-run stability | [scenario matrix](02-scenario-matrix.md) §3; [automation contract](03-automation-contract.md) §5–§6 | P4-V15 |
| Timeout / incomplete / unsupported outcomes diagnosable | [automation contract](03-automation-contract.md) §4 (verdict taxonomy) | P4-V13–V15 (non-success is determinate) |
| Evidence destinations | [automation contract](03-automation-contract.md) §6 | evidence land in `../verification/` only |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs`):
documentation-only repository — no runner, no build, no Guest image, no CI
workflow content (`.github/workflows/` has only `.gitkeep`), and no prior
P4 run evidence of any kind. The P0-W09 runner and P1-W10 boot-regression
conventions are plans, not delivered automation. The fine-grained P4-K
requirement wording of the superseded root source task book is not tracked
(W01 item A9 provenance pattern); requirement meaning here comes from the
tracked task book §5/§6 rows and the W08 plan scope sentences. Everything
below is an assumed contract or a P4-internal deliverable.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Automation builds Hypervisor and Guest, prepares the image, boots QEMU | No build exists; no runner exists (P0-W09 planned, W01 R06/R18) | The declared build/image/boot boundary over the P0-W09 entry ([03 §2–§3](03-automation-contract.md)) | "automation builds and boots" requires a fixed, single command path with declared inputs; two paths would drift | W08 contract; P0-W09 entry (assumed M3); toolchain baseline (assumed M4) | DV01 review; P4-V13 execution when unblocked |
| Expected serial/telemetry markers collected | Marker grammar (W05) and run record (W07) exist only as proposed designs | Collection rules keyed to the VG marker grammar and P4-RR v1 record ([03 §3–§4](03-automation-contract.md)) | collection must parse the agreed, versioned surfaces, not free text | W08 rules; W05/W07 grammars (assumed M1/M2) | DV02; DV03 |
| Positive EL1 entry determinate (P4-V13) | Nothing exists | SM-01 row with named markers and pass condition ([02 §2](02-scenario-matrix.md)) | a determinate result needs an exact expected observable, not a "looks fine" rule | W08 matrix; W05 VG-001 expectations (M1) | DV03; P4-V13 |
| Translation/permission fault and survival/stop determinate (P4-V14) | Nothing exists | SM-02–SM-06 rows incl. EL2-survival observation ([02 §2](02-scenario-matrix.md)) | fault cases pass only when the fault is expected AND EL2 survives to a diagnosable end state | W08 matrix; W06 IS-series (assumed M2); W07 record (M2) | DV04; P4-V14 |
| Repeated declared iterations stable (P4-V15) | Nothing exists | Repeat rules: minimum counts, independence, per-iteration evidence ([03 §5](03-automation-contract.md)) | stability must be defined over independent, evidence-retained iterations | W08 rules; W07 minimums (M2, D7) | DV05; P4-V15 |
| Timeout / incomplete / unsupported are diagnosable non-success | Nothing exists | Verdict taxonomy with distinct determinate outcomes ([03 §4](03-automation-contract.md)) | an indistinguishable hang is the failure mode this requirement excludes | W08 taxonomy; P1-W10 precedent (M3) | DV06 |

No row above requires a decision outside this design's authority; the
scenario set shape, verdict names, manifest schema, and evidence layout are
stage-local design freedom in the plan's declared scope.

## Resolved design decisions and their authority

Summarized here; full rationale and authority citations in
[01 §4](01-scope-and-foundations.md):

1. **One automation entry point:** the P4 regression extends the P0-W09
   runner entry with P4-scoped parameters; no parallel QEMU command path is
   created, and no QEMU knowledge enters Core.
2. **Scenario matrix fixed here (SM-series), keyed to W5/W6/W7 seams:** each
   row names its VG scenario, IS expectation, P4-RR fields, and repeat rule;
   W08 never re-derives expectations.
3. **Determinate verdict taxonomy:** `PASS`, `FAIL`, `TIMEOUT`,
   `INCOMPLETE`, `UNSUPPORTED` — every run ends in exactly one; timeouts and
   missing evidence are non-success outcomes, never passes and never
   crashes.
4. **Expectations are host-authored contracts:** marker sequences (W05), IS
   verdicts (W06), and P4-RR fields (W07) are the only accepted evidence;
   QEMU console text beyond those surfaces is not a pass condition.
5. **Repetition without randomness:** fixed scenario sequence, fresh QEMU
   process per iteration, W07's minimum counts (may be raised in the
   manifest, never lowered), per-iteration evidence retained; no seeds exist
   because no input is random.
6. **Evidence lives under stage `verification/` only,** with a declared
   layout per run (logs, records, build identity); design and plan documents
   never hold evidence.
7. **Regression verdict is all-or-nothing per declared set:** any
   non-`PASS` iteration fails the regression for that set; partial sets are
   reported as `INCOMPLETE`, not as passes.
8. **No mechanism code:** W08 delivers automation tooling (host-side scripts
   and manifest data) plus documentation; any pressure to fix Guest/hypervisor
   behavior from the automation side is a defect in the owning package,
   recorded, never patched here.

## Work breakdown and loading order

1. Load [01-scope-and-foundations.md](01-scope-and-foundations.md): the
   Required/Reserved/Out-of-Scope detail, assumed contracts M1–M6 with
   failure boundaries, contested-area analysis, decisions D1–D8, open items.
2. Load [02-scenario-matrix.md](02-scenario-matrix.md) for the SM-series
   rows: per-scenario input, expected observable, pass condition, and proof
   boundary; and the repeat rules for P4-V15.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md),
   loading [03-automation-contract.md](03-automation-contract.md) for the
   manifest, entry point, verdict rules, and evidence layout.
4. Record validation in
   `../../verification/p4-w08-qemu-integration-regression-verification.md`
   and implementation facts in
   `../p4-w08-qemu-integration-regression-record.md` only when work starts;
   neither this design nor the records may claim W08 complete, and no
   scenario result may be written anywhere before a real run produced it.

## Explicitly excluded interfaces

Not designed or authorized by W08: any hypervisor or Guest code path,
module, or API; any change to the VG marker grammar, IS expectations, or
P4-RR grammar (consumed as versioned inputs; change requests route to W05/
W06/W07); the QEMU runner's internal implementation (P0-W09's); CI provider
configuration (P0 CI package); Orange Pi or any non-reference environment;
Linux Guest scenarios (P8); performance timing methodology; and any
completion or pass claim. The regression also defines no new Core-visible
interface: everything it adds lives in the automation/tooling layer.

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W09** (design: `../p4-w09-closeout-p5-handoff/README.md`) receives
  the automation entry point name and usage contract, the declared
  environment facts, the scenario set identity (SM-series version), and the
  evidence locations — as facts to record, with their run/not-run status as
  of closeout.
- **P5 regression users** ([P5-W09](../../../p5/plans/p5-w09-telemetry-safe-logging-regression.md)
  and [P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md) via the P5
  task book's preserved-P4-regression requirement, P5-V16): receive the
  preserved P4 scenario set with its extension rules — P4 scenarios are
  preserved verbatim and new P5 scenarios are added alongside, never
  redefined on top of them.
- **P4-W05/W06/W07** receive defect reports, not patches: a failing
  expectation is raised against the owning package's design.
