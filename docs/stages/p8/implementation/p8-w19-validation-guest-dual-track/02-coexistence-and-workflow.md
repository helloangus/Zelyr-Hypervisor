# P8-W19 Coexistence, Workflow, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W19 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any work, the implementer verifies it has loaded the documents named
in the parent README and inspects the tracked tree. The following are
**assumed contracts**; a failure in any of them produces a recorded block,
never a local repair:

- **P8-W01 reconciliation** (`p8-w01-entry-contract-reconciliation`): the
  statement of which P4–P7 Validation Guest facts are evidenced. The
  inventory's placeholder statuses are derived from it, not re-audited.
- **Source suite designs and records** (`p4-w05-validation-guest`,
  `p5-w07-validation-guest-isolation-suite`,
  `p6-w11-validation-guest-interrupt-suite`,
  `p7-w10-validation-guest-suite`, plus P4-W09/P5-W10/P6-W13/P7-W14 closeout
  links): scenario content, observables, and existing evidence.
- **W16 envelope** (`p8-w16-automated-linux-regression`): the `VG-RET`
  family as the VG execution path.
- **W15 fixture** (`p8-w15-reproducible-linux-fixture`): Linux-fixture
  expectations; the VG asset itself remains the P4-W05 maintenance boundary.
  *Failure boundary:* if no maintenance boundary for the VG asset is
  evidenced, W19 records the block against P4-W05; W19 does not adopt the
  asset, because adopting it would extend W19's scope into VG
  implementation.
- **W02-governed approved machine contract**
  (`p8-w02-machine-contract-governance`): the reference for applicability
  judgments. *Failure boundary:* absent approved facts ⇒ applicability
  judgment is deferred and rows stay marked pending, never judged against
  candidate values.

## 2. Coexistence rules

1. **Single envelope.** Both tracks execute through the W16 harness contract.
   W19 defines no second execution path; a VG row and an LG row differ only
   in their declared inputs and owned observables.
2. **State independence.** Rows are order-independent with respect to each
   other: the harness resets between rows per the W16 contract, and no VG
   row may depend on an LG row having run (or vice versa). Any apparent
   ordering requirement is a harness defect to record, not a scenario
   property to encode.
3. **Evidence symmetry.** Both tracks produce the same class of run records
   and transcripts; verification and closure report them as two columns.
   Merging results into one "regression passed" verdict is prohibited.
4. **Change routing.** A P8 machine-contract change that would alter a VG
   row's inputs routes through the applicability rule (§4) and the source
   owner; W19 never adapts a scenario unilaterally, and a Linux-track change
   never touches a VG row.
5. **Review visibility.** Any review that reports P8 regression status must
   show VG coverage next to LG coverage (P8-V25's dual requirement); a
   report showing only the Linux track is an invalid report of P8-V25.

## 3. Fixture-maintenance expectations

- **VG asset:** owned and maintained under the P4-W05 maintenance boundary
  (assumed contract). W19's regression-perspective requirements are: the
  asset remains loadable under the approved P8 machine contract; its
  scenario markers remain stable per source design; any required rebuild
  remains reproducible under the P0 toolchain baseline. W19 records — but
  never performs — required asset changes, routing them to the owning
  design.
- **Linux fixture:** owned by W15; W19's only requirement is that Linux
  fixture evolution cannot become a reason to skip VG rows (they share no
  fixture dependency).
- **Version coupling:** neither fixture's version is coupled to the other's;
  run records record both references so a reviewer can attribute a dual-track
  result to exact fixture states.

## 4. Applicability and lost-coverage rule

When the P8 machine contract, host-side facts, or an upstream regression
affects a retained scenario, W19 assigns exactly one applicability state:

```text
unchanged       scenario runs as-is; observables inherited unchanged
adapted-input   the scenario's *input parameters* (e.g., load parameters)
                must be re-expressed for the approved P8 machine contract;
                observable semantics are untouched; the adaptation is
                recorded with the approved fact it derives from, and the
                source owner is notified
pending         the applicability judgment cannot be made yet (e.g., absent
                approved machine facts); recorded as pending with the
                missing decision named
blocked         the scenario cannot run in P8 (upstream regression, missing
                trigger, contract conflict); recorded as a dependency block
                naming the owner; remains in the inventory
```

**Lost coverage is always a block** (plan work seq 5): a scenario may never
be dropped, merged, or weakened to make the dual-track regression pass. A
`blocked` inventory row is unclosed P8-V25 scope: it is visible to
[W20](../p8-w20-documentation-closure-handoff/README.md) closure and counts
as an open exit item until the owner resolves it. If closing it would require
changing an ADR-level decision (e.g., the machine contract must break a
mechanism invariant), it is labeled `Architecture Change Request` or
`ADR Required` in the verification record.

If P8 integration exposes a mechanism gap the VG cannot observe (no scenario
covers a new Guest-visible surface), the gap is recorded as an open
investigation routed to an authorized design; W19 does not design the new VG
scenario itself (plan exclusion).

## 5. Ordered implementation workflow

### Step 1 — derive inventory statuses from W01 and source records

Target: implementation record
(`../p8-w19-validation-guest-dual-track-record.md`, created in this step).

Work: read W01's reconciliation and each source suite's record status; set
each [01](01-dual-track-scenario-matrix.md) §3 row to its §4 applicability
state with the deriving fact cited.

**Acceptance:** every inventory row has an applicability state and a cited
basis; placeholders are explicit.  
**Failure/blocker:** conflicting source statements are recorded as conflicts
and escalated; W19 does not reconcile upstream discrepancies itself.

### Step 2 — confirm envelope binding

Work: verify the W16 `VG-RET` family declares the retained entry set this
inventory defines, and that no second VG execution path exists.

**Acceptance:** one envelope, entry set consistent with the inventory.  
**Failure/blocker:** an inconsistency is a W16↔W19 design-coordination item
recorded in both records, resolved by reviewed edit of the owning design.

### Step 3 — execute the retained set

Target: verification record
(`../../verification/p8-w19-validation-guest-dual-track-verification.md`)
and W16 run records.

Work: run executable rows through the envelope under declared repetition
parameters; retain transcripts and run records; keep LG-track results in
their own column.

**Acceptance:** every executed row has evidence; blocked/pending rows remain
listed with reasons.  
**Failure/blocker:** a failing VG row is evidence routed to the owning
package (P4–P7 semantics) or to W16 (envelope defect); W19 does not patch
either.

### Step 4 — produce the dual-track statement

Work: from evidence only, state: which mechanism areas have green VG and LG
coverage, which have one side blocked or pending, and which have neither.
This statement is the P8-V25 evidence summary; it includes the
anti-substitution clause verbatim.

**Acceptance:** the statement separates tracks, names every gap, and claims
nothing for unexecuted rows.  
**Failure/blocker:** a required gap statement that cannot be written because
evidence is missing is itself recorded (the statement then says so).

### Step 5 — closure review

Work: run the validation matrix (§6); confirm the handoff checklist (§8).
Completion is claimed only in the verification record, with evidence, for
what was actually run.

## 6. Validation matrix

All rows are planned evidence; none asserts a run occurred. Record each as
**passed**, **failed**, **blocked**, or **not run** with command, input,
environment, timestamp, and reason.

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W19-DV01 → prerequisites | contract review | inspect §1 assumed contracts against tracked state and W01 | every contract named with owner and failure boundary; no adopted VG asset | retention-plan coherence; not that suites exist |
| W19-DV02 → P8-V25 | inventory completeness review | review [01](01-dual-track-scenario-matrix.md) §2–§3 against the four source suites and the plan's mechanism list | every plan-named mechanism area has inventory rows; every row cites its source authority; no restated observable | retained-set completeness; not that scenarios pass |
| W19-DV03 → P8-V25 | track-separation review | review [01](01-dual-track-scenario-matrix.md) §4 | every mechanism has distinct VG and LG probes with distinct proof statements; anti-substitution clause present | separation design; not that either track passes |
| W19-DV04 → P8-V25 | coexistence and maintenance review | review [02] §2–§3 | single envelope; state independence; evidence symmetry; fixture responsibilities stated with failure boundaries | coexistence design; not CI execution (Reserved) |
| W19-DV05 → P8-V25 | dual-track regression run | execute the retained set per Step 3 | every executable row passes with inherited observables; blocked/pending rows listed; lost-coverage register current | the Validation Guest mechanism suite remains executable and green alongside Linux, in the declared environment; not mechanism correctness beyond the suites' own scopes, not hardware behavior |
| W19-DV06 → handoff | consumability review | read the dual-track statement as W20 and as P9 | consumers can act without merging tracks or overstating coverage | handoff readiness; not that downstream work is done |

Only W19-DV05 with real runs (plus DV02/DV03's audits) can satisfy P8-V25.
QEMU/TCG results remain environment-scoped; VG passes prove the declared
mechanism behavior under QEMU and never hardware semantics.

## 7. Error, security, and observability model

- **Failure model:** W19's named failure modes are: silent inventory
  divergence from source suites, a weakened or dropped row, a merged-track
  report, and an unregistered mechanism gap. Each is a review failure with a
  recorded remedy (restore the row, re-split the report, register the gap).
- **Security position:** W19 preserves the P5 two-context authority-isolation
  evidence route and W18's citation of it. It introduces no new trust
  decision; the VG remains an untrusted-Guest probe asset, and dual-track
  tooling must treat transcripts as untrusted input (Coding Guidelines
  untrusted-input rule).
- **Observability:** evidence is the W16 run records and transcripts for
  executed rows, the source suites' own verification records for inherited
  evidence, and the W19 verification record for run/not-run status and the
  lost-coverage register.

## 8. Handoff checklist

Before handing W19 to a reviewer, provide:

- the exact artifact list (inventory, mapping, verification record) and the
  inventory revision identity;
- W19-DV01–DV06 evidence paths and run status, including explicit
  blocked/pending rows with owning packages named;
- the lost-coverage register: every block, its owner, and its resolution
  route;
- the dual-track statement with both columns and the anti-substitution
  clause;
- confirmation that no VG implementation, scenario, observable, or fixture
  change was made under W19 authority;
- confirmation that no Linux result was used to close a VG row or vice
  versa;
- open items: source-suite evidence status (per W01), approved machine
  facts for applicability judgments (task book §8 `ADR Required` via W02),
  and the P4 VG-asset maintenance boundary — without resolving any of them
  here.
