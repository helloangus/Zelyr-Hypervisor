# P3-W13 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W13 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"A repeatable QEMU `virt` Host SMP regression matrix for 1, 2, 4, and 8
CPUs and repeated cold boots" requires five concrete artifacts:

1. **Rows** — R-series per-count rows with assertions sourced from the
   owning designs — [02-regression-matrix.md](02-regression-matrix.md) §2.
2. **Repetition policy** — cold-boot depth per row with rationale — same
   file §4.
3. **Scenario deployment** — the S1–S7 mapping to counts and depths —
   same file §5.
4. **Classification and capture** — result vocabulary and per-run capture
   set — [03-result-classification-and-evidence.md](03-result-classification-and-evidence.md)
   §3.
5. **Execution path** — the P0-W09 automation entry consuming rows, or an
   honest blocked status until it exists — same file §2.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| QEMU automation entry | [P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md) | A declared way to build, boot the hypervisor image under QEMU `virt` with a declared smp count, and capture output/exit status | Without it, every QEMU row is **blocked** and recorded as such; W13 authors no runner. If its inputs cannot express a per-row CPU count or capture set, the row is blocked on P0-W09 with the gap cited — never worked around with local scripts |
| CI baseline | [P0-W20](../../../p0/plans/p0-w20-ci-baseline.md) | CI consumes declared evidence-producing commands; matrix gating is P0-W20's wiring decision | A CI gap is a P0-W20 issue, recorded; W13 adds no workflow files |
| Host-side testing baseline | [P0-W08](../../../p0/plans/p0-w08-host-side-testing-baseline.md) | Host-test convention for the review-side checks | Absence reclassifies affected checks as manual reviews, recorded |
| Bring-up and rendezvous assertions | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md), [P3-W05](../p3-w05-smp-boot-synchronization/README.md) | Outcome/phase vocabularies; induced absent-CPU input; `SmpReadyState`; degraded accounting | A vocabulary change is a contract conflict resolved between designs; matrix rows cite, never redefine |
| Lifecycle and per-CPU surfaces | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md), [P3-W04](../p3-w04-per-cpu-runtime/README.md) | `OnlineSet`, isolation diagnostics, counter-block capacity | Same treatment |
| Observability surfaces | [P3-W11](../p3-w11-smp-observability/README.md) | Dump/snapshot/event surfaces; attribution rules | W11 seam gaps degrade specific row checks; cited per result |
| Scenarios | [P3-W12](../p3-w12-smp-stress-failure-tests/README.md) | S1–S7 definitions, limits, determinism rules, evidence template | Matrix rows consume scenarios at declared depths; W13 does not edit scenarios — a needed change goes back to W12's record |
| SMP-safety audit status | [P3-W10](../p3-w10-smp-safety-audit/README.md) (parallel, unread) | The audit's classification/unresolved items underlying the stressed mechanisms in rows R5/R7 | An unresolved audit item on a stressed mechanism makes the corresponding row's pass conditional on that item's resolution — recorded per row, never assumed clean |
| Inventory bound and topology | [P3-W01](../p3-w01-cpu-topology-inputs/README.md) | Declared counts 1/2/4/8; n ≤ 8 bound; intake diagnostics | A count beyond the bound is a Reserved trigger (W01 design change), not a matrix row |

### 1.3 Why no hidden essential deliverable remains

- Plan step 1 ("inspect P0 QEMU/CI contracts and W02/W05–W12 acceptance
  boundaries") is realized by §1.2 — every inspected contract is an
  assumed-consumption rule with a failure boundary.
- Plan step 2 ("define matrix rows, repetition policy, diagnostic capture,
  and result classification") is realized by §1.1 items 1–4.
- Plan step 3 ("integrate matrix evidence with stress/failure cases and
  P4 entry review") is realized by the scenario mapping (matrix §5) and
  the handoff section.
- Plan step 5 ("run or collect matrix evidence when implementation
  exists") is the deferred execution phase with its own status rules; the
  matrix contract, not results, is the design-phase deliverable.

## 2. Scope classification

### 2.1 Required

- R-series rows for each of 1/2/4/8 CPUs with per-row assertions, capture
  sets, and prove/does-not-prove statements.
- Cold-boot repetition policy with recorded depth constants.
- S1–S7 scenario deployment mapping.
- Result classification and per-run evidence destinations.
- The P0-W09 consumption rules and blocked-status rules.
- The platform-independence and QEMU-limits review obligations.
- The hardware-gap statement inherited by P15/W14.

### 2.2 Reserved (must not block a future design; not implemented now)

- CPU counts beyond 1/2/4/8; trigger: an approved W01 inventory-bound
  change.
- CI gating of matrix rows; trigger: P0-W20 wiring decision consuming
  this contract.
- Additional machine/board rows; trigger: an approved platform addition
  (ADR-045 tiers).
- Performance rows; trigger: a performance-baseline design (P7+).
- Guest/Stage-2 regression rows; trigger: P4-W08's own contract.

### 2.3 Out of Scope

- Runner/CI implementation and scripts (P0-W09/P0-W20).
- Real-hardware validation (P15).
- QEMU behavior as an architecture definition (ADR rules; recorded as
  observations only).
- Performance certification; timing thresholds.
- Guest, vCPU, Stage-2, GIC regression content (P4+).
- Editing W12 scenarios or any mechanism design to suit the matrix.
