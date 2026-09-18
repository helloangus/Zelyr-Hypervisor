# P4-W09 Closeout Artifact Contracts and P5 Consumer Mapping

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W09 detailed design](README.md).

## 1. Logical artifact groups and ownership

W09 is a review package: its logical modules are authoritative record
sections, not Rust modules. All sections live in one implementation record
so a P5 reader resolves one file; the verification record holds only the
review's own evidence.

| Record section (in `../p4-w09-closeout-p5-handoff-record.md`, created when the review runs) | Authoritative content | Primary sources (W01–W08 records) | Non-responsibility |
|---|---|---|---|
| Boot-contract record (P4-L01) | the implemented route from build to Guest EL1: image route, embedding, boot-info block facts, entry convention, console page, scenario-id channel — each as implemented, with evidence links | W03, W05, W04 implementation records | it does not define a boot ABI (P8) or restate design intent as fact |
| Stage-2 capability matrix (P4-L02) | per-capability rows: lifecycle, map, unmap, protect, query, activation, current-path invalidation, permission enforcement, isolation negatives — each `supported (evidence)` or `not delivered (blocking row)` | W02, W06 records | it does not grade quality or promise future capabilities |
| Known-limitations record (P4-L03) | every recorded limitation with downstream owner: interrupt masking, EL1-state non-persistence, current-path-only invalidation, single-Guest/vCPU scope, no hardware walk, ledger-dump-only diagnostics, no telemetry transport, declared-environment scope, planned-scenario deferrals | W02–W08 records; W05 deferral section if any | it does not resolve limitations or schedule work |
| Unsafe-inventory delta | the P4 unsafe surface as implemented (W04 stubs/helpers, W02 write path, W03 write view — as actually delivered), each with SAFETY-note location | W0x records + the P0-W10 inventory when delivered | it does not audit the unsafe itself (P0 governance owns the rules) |
| ADR-deviation and conflict register | every `ADR Required` / `Architecture Change Request` / `Blocked prerequisite` item still open, incl. **P2-ACR-01** and the A9 scenario-provenance finding, plus any Specification Investigation items | W01 gap list; W0x records | it never resolves a labeled item |
| Evidence index (P4-V01–V16) | one row per matrix ID: evidence location or explicit absence, with run status | all `docs/stages/p4/verification/` records | it does not judge quality beyond the criterion wording |
| Not-delivered scope statement | the explicit list of P4-planned scope that has no evidence at closeout | the evidence index | it is not an apology or a schedule |
| P5 handoff statement | the consumer mapping of §3 with evidence status per deliverable | all of the above | it confers no new authority and defines no P5 semantics |

## 2. Truthfulness rules (what keeps P4 facts from becoming contracts)

1. **Citation rule:** every factual statement cites at least one W01–W08
   implementation or verification record path (and, where a number or name
   is stated, that record is its source). Uncited statements are moved to
   the not-delivered section at review.
2. **Evidence-status rule:** capability rows use exactly two states —
   `supported (citing verification evidence)` or
   `not delivered (citing the blocking W01 row)`; no intermediate wording
   exists, so partial credit cannot hide in prose.
3. **Non-ABI labeling rule:** every mention of a temporary convention
   (IPA layout values, boot-info format, console page, scenario table,
   marker grammar, P4 exit classes, P4-RR grammar, probe channel, repeat
   minimums) carries the label "P4 test contract — not a machine or guest
   ABI" (W01 A4; task book §1 Reserved). The boot-contract record opens
   with this statement so excerpts cannot strip it.
4. **Guest-fault vs Hypervisor-invariant rule:** the closeout restates the
   containment boundary as a fact only where W06's containment evidence
   exists; the distinction is never claimed as proven beyond the exercised
   classes (P4-V09 scope).
5. **Completion-language rule:** pass/fail/blocked/not-run statements appear
   only in `docs/stages/p4/verification/` records, per row; the closeout
   record references them and never re-states them as its own claims.
6. **Deferral rule:** a planned scenario (VG-008/VG-009/VG-011) without
   evidence appears in the limitations record with the AArch64-specific
   reason, the downstream owner, and the exit-criterion effect — the task
   book §6 requirements — or the closeout is incomplete.
7. **Deviation rule:** anything found during closeout that contradicts an
   ADR or an upstream contract is added to the conflict register with both
   source citations and routed to the owning stage; it is never absorbed by
   rewording.

## 3. P5 handoff — consumer-by-ID mapping

The [P5 task book](../../../p5/task-book-v0.1.md) is the named consumer
authority (its §2 entry-conditions table and §3 package map); this mapping
names what each P5 package may consume from P4 and with what evidence
status. It is recorded in the handoff statement with per-deliverable status
filled at closeout.

| P5 package (plan path under `docs/stages/p5/plans/`) | Consumes from P4 | Deliverable and owner |
|---|---|---|
| [P5-W01](../../../p5/plans/p5-w01-entry-contract-reconciliation.md) | the whole P4 entry reconciliation: evidence index, conflict register, not-delivered scope | W09 closeout record (all sections) |
| [P5-W02](../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md) | the evidenced EL2→EL1→EL2 boundary; W04's exit-class vocabulary and routing facts (HVC disabled/undefined intent, SMC classify-and-stop); W06's Guest-vs-Hypervisor fault distinction as the containment precedent | W04 boot/exit facts; W06 capability rows |
| [P5-W03](../../../p5/plans/p5-w03-guest-data-safety.md) | Stage-2 query/mapping facts, bounded Guest RAM facts, faulting-IPA diagnosis facts — the basis for checked Guest-data access | W02 matrix rows; W03 boot-contract facts; W06 diagnostic facts |
| [P5-W04](../../../p5/plans/p5-w04-handle-lifecycle-type-safety.md) | the P4 object-lifecycle precedents: vCPU run states and stop path, address-space lifecycle, GuestRam release sequencing — as facts P5's handle lifecycle supersedes, not as APIs | W04/W02/W03 limitation and capability rows |
| [P5-W05](../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md) | the single-Guest scope facts P5 must not over-generalize; the no-identity-shortcut posture as inherited from the ADR via P4's records | W09 scope/limitation rows; ADR citations |
| [P5-W06](../../../p5/plans/p5-w06-dispatch-permission-containment.md) | W06 containment evidence and the GuestFault-vs-invariant escalation split — the behavioral precedent P5's dispatch containment extends | W06 capability and limitation rows |
| [P5-W07](../../../p5/plans/p5-w07-validation-guest-isolation-suite.md) | the maintained Validation Guest asset: scenario table version, marker protocol, boot-info validation conventions, and the isolation-suite extension surface | W05 maintenance facts; W08 entry point |
| [P5-W08](../../../p5/plans/p5-w08-host-fuzz-stress-smp-baseline.md) | the observability baseline (event categories, counters, run record) and the QEMU regression entry point as the stress/fuzz regression base; the P3-inherited SMP facts as recorded | W07 telemetry facts; W08 automation facts |
| [P5-W09](../../../p5/plans/p5-w09-telemetry-safe-logging-regression.md) | the P4 event/counter inventory and P4-RR grammar as implemented facts to preserve or supersede under the P0 namespace rules | W07/W06/W02/W03/W04 event inventories |
| [P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md) | the closeout method itself (artifact groups, truthfulness rules) as the precedent P5 closeout follows | this design's contracts |

Mapping rules: a P5 package consumes only rows whose evidence status is
`supported`; for `not delivered` rows the mapping names the blocking W01
row, and the P5 package records the dependency per its own entry rules. The
mapping grants no design authority: P5 semantics are P5's (P5 task book §2
"implementation-stage deliverables").

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | One closeout record with the fixed §1 section set | one citable file for P5-W01's reconciliation; section separation keeps facts, conflicts, and handoff distinct | task book §3 delivery hierarchy; plan step 2 |
| D2 | Facts/plans split enforced by citation | the plan's core risk is "mistaking planned work or temporary test conventions for completed architecture"; a mechanical citation rule is checkable | P4-W09 plan goal; checklist §4 |
| D3 | Two-state capability rows | prevents partial-credit wording; matches the task book's evidence-or-absence style (P4-V01) | task book §6/§7 |
| D4 | Non-ABI labels at every mention | excerpting must not strip the temporary-facts disclaimer | W01 A4; task book §1 Reserved/§8 |
| D5 | P5 handoff as a consumer mapping | the task book names P5 the primary downstream; per-package mapping prevents both over- and under-claiming | task book §7; [P5 task book](../../../p5/task-book-v0.1.md) §2 |
| D6 | Completion language confined to verification records | keeps the design/implementation/verification separation honest | task book §3; W01 A8 |
| D7 | W09 adds no code, no evidence, no unsafe | completion-review package; cannot manufacture evidence | P4-W09 plan scope |

## 5. Explicitly excluded interfaces

There are no Rust structs, enums, traits, functions, modules, crates, ABIs,
wire formats, scripts, or CI configurations in this design, and none are
authorized. W09 writes only: its closeout record, its verification record,
and (if review finds gaps) updates to its own design here — never upstream
documents, never P5 documents, never verification evidence for other
packages. If executing the review appears to require any of those, the
requirement is itself a gap to record per [02](02-review-workflow.md) §1.
