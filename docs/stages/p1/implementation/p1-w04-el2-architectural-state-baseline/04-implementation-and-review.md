# P1-W04 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

W04 can start only after the W02 runtime and W03 report exist in
implementable form (the baseline executes between them and guards on their
facts) and the W09 adapter contracts are available at their accepted-design
level. Before changing any file, the implementer verifies the mandatory
reading (parent README), inspects the current tree (`git ls-files`; confirm
the W02/W03/W09 state), and records the assumed-contract states from
[01-architecture-and-state.md](01-architecture-and-state.md) §8.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W03 report or query API is absent or contradicts the §8 seams — raise
  the conflict per W09's rule; do not re-read identification registers to
  work around it;
- the recorded architecture revision cannot establish a §2 bit position —
  record the blocker; do not guess masks (a wrong mask silently un-establishes
  a control);
- a control in C1–C8 proves inaccessible or behaves contrary to the
  architecture reference at EL2 — that falsifies the baseline value, not the
  architecture: record the design conflict; do not locally retune the value;
- closure appears to require vector installation, MMU enablement, GIC/timer
  virtualization, EL1 entry, or a guest policy — Out of Scope (parent
  README); stop.

## 2. Ordered implementation steps

### Step 1 — map baseline categories to capability facts

Target: implementation record
(`../p1-w04-el2-architectural-state-baseline-record.md`, created in this
step).

Work: confirm the category/fact mapping of
[01-architecture-and-state.md](01-architecture-and-state.md) §2 against the
W03 fact table as implemented; record for every category its guards, its
context facts, and the controls it covers; record the architecture revision
and the exact bit positions used for every §2 mask/constant.

Suggested observation: the architecture reference for the recorded revision;
the W03 implementation record; no repository change.

**Acceptance:** every category maps to declared facts (or explicitly to
none); every mask/constant has its recorded bit positions.  
**Failure/blocker:** a guard without a queryable fact stops the step (the
guard would be fiction); the W03/W04 coordination issue is recorded.

### Step 2 — implement the control-write layer

Target: the write module.

Work: implement the specification engine, the `ControlId` set, the two
audited `unsafe` primitives, and every §2 write exactly per
[02-code-contracts-control-writes.md](02-code-contracts-control-writes.md),
in C1→C8 order.

Suggested observation: none beyond review until the phase runs under W09;
record any tooling inspection choice made for the no-FP guarantee.

**Acceptance:** the `unsafe` inventory contains exactly the two primitives;
every write follows the RMW-with-mask or fully-specified-constant form; no
control outside the §2 set is nameable.  
**Failure/blocker:** a control requiring a form outside the two allowed
ones is a design conflict to record (parent README decision 2).

### Step 3 — implement read-back, the error route, and guards

Target: the verification module.

Work: implement the read-back step, `BaselineError`/`fail_baseline`, the
guard-consistency check, and the C1 precondition assertion per
[03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md)
§1–§3, routing through W02's panic route.

**Acceptance:** every write has exactly one read-back; every failure class
routes with the control's static name; guards skip with `SkippedAbsent` and
never guess.  
**Failure/blocker:** a route-carrying limitation (report cannot hold the
fields) is a W02/W04 coordination issue; no second route is built.

### Step 4 — implement the declaration and wire the phase body

Target: the declaration module and the establishment body.

Work: implement the declaration static and API per §4; wire
`establish_el2_baseline` as the W09 `el2_baseline_step` adapter's single
call; confirm the postcondition handoff (all categories Established or the
route taken).

**Acceptance:** the phase body adds nothing beyond assert → write → verify →
declare; consumers specified in the consumer map can name their API calls.  
**Failure/blocker:** a seam mismatch with W09 is raised per §1; silent
adaptation is prohibited.

### Step 5 — residue and scope review

Target: implementation record; the verification record.

Work: walk the firmware-residue rule register by register: every C1–C8
control has a recorded value; no control is "left as is"; no register
outside the ownership matrix is written by W04; no W04 artifact contains a
Guest/SMP/GIC/Stage-2 policy statement or a platform-name branch. Re-check
the no-FP guarantee's dependency state (parent README decision 3).

**Acceptance:** the residue review table is complete with per-control
pointers; the scope review is clean.  
**Failure/blocker:** a residue gap is a P1-V07 finding recorded with an
owner; an unremovable one stops the package.

### Step 6 — cold-boot consistency evidence and handoff

Target: verification record
(`../../verification/p1-w04-el2-architectural-state-baseline-verification.md`).

Work: perform the executable reviews of §3; record the read-back mechanism
as the at-boot consistency check; record the repeat-boot consistency proof
(identical category status and values across 100 W10 regression boots) as
deferred execution with W10 ownership; complete the handoff checklist.

**Acceptance:** the verification record distinguishes passed reviews,
deferred executions, and not-run items.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis;
completion is not claimed around it.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W04-DV01 → W04 closure | Fact-mapping review | step 1's table against the W03 implementation | every category's guards/context map to queryable facts; no re-derivation | ADR-044 conduct of the baseline; not the facts' own correctness (W03's) |
| W04-DV02 → P1-V07 | Write-specification review | inspect the implementation against [02-code-contracts-control-writes.md](02-code-contracts-control-writes.md) §2 | every control has form, recorded value, rationale, consumer; masks match recorded bit positions; C1–C8 order holds | the baseline values are explicit and owned; not that the machine honors them |
| W04-DV03 → P1-V07 | Verification and route review | exercise the read-back/guard logic against constructed cases (host-side where permitted; otherwise inspection) | every write verified by masked read-back; mismatch routes fatally with control identity; guards skip honestly; C1 asserts W02's values | at-boot consistency checking by design; not hardware behavior on real platforms |
| W04-DV04 → P1-V07 | Residue and boundary review | step 5's register walk + `unsafe` inventory read | no unowned or unrecorded control; exactly two `unsafe` primitives with filed justifications; no scope item touched | firmware-state independence as designed; not an exhaustive architecture audit |
| W04-DV05 → P1-V07 | Establishment-order review | map code to the §4 lifecycle of the architecture file | single execution, C1→C8 order, monotone statuses, no rollback | the known-state boundary is reached deterministically; not boot execution |
| W04-DV06 → P1-V07 | Cold-boot consistency (deferred execution) | W10 regression boots after W09 integrates; declaration API values compared across boots | identical category statuses and recorded values on every clean boot | cross-boot logical consistency (P1-V07's executed half); owned by W10, not by W04 |
| W04-DV07 → W04 closure | Consumability review | read the outputs as W05 (can I install vectors against this boundary?), W08 (are my premises assertable?), W09 (is the phase body exact?), W11 (is the failure class usable?), W12 (is the baseline record documentable?) | each consumer can act without inventing W04 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. The QEMU-
dependent proof (W04-DV06) is deferred to W10 by contracted wiring; until
it exists, P1-V07's executed half is unproven and no W04 artifact may
report otherwise. No validation here proves P1-V08 through P1-V21.

## 4. Error, security, and observability model

**Errors.** The baseline has exactly one failure class: `BaselineError`,
fatal, named by control, routed via the panic route with phase attribution.
Guarded optionals skip with recorded status — a data point, not an error.
There is no retry, no fallback value, no partial establishment, and no
rollback.

**Security.** The baseline *is* the stage's security posture:
deny-by-default traps for FP/SIMD and debug/performance, masked interrupts,
Stage-2 disabled, MMU off, EL1/EL0 timer and FP access denied — each value
recorded with rationale, removing firmware residue rather than inheriting
it. `SCR_EL3` is documented as outside P1 ownership (ADR-008). `unsafe` is
confined to the two register primitives plus the declaration cell, each
with filed `SAFETY` justifications. The complement guarantee — that P1's
own code emits no FP/SIMD — is a recorded dependency on the P0 target
semantics, verified by review, with toolchain inspection as the recorded
escalation if deeper evidence is demanded.

**Observability.** The baseline's observables: the declaration API (status
and values, consumed by later diagnostics), the fatal route's control-named
report, and the deferred cross-boot consistency comparison in W10's
evidence. W04 adds no prints, counters, or telemetry of its own.

## 5. Handoff checklist

Before handing W04 to a reviewer, provide:

- the exact changed-file list and module locations of every contracted item;
- the recorded architecture revision and per-control bit positions;
- W04-DV01..DV07 evidence paths and run status, including the explicit
  deferred/not-run entry (cross-boot consistency → W10);
- the audited-`unsafe` list (two primitives + declaration cell) with filed
  `SAFETY` justifications;
- the recorded baseline values as the W12 baseline record input, including
  the boundary facts (vectors unowned until W05; values minimal-not-final;
  `SCR_EL3` outside P1 ownership);
- the no-FP build guarantee's dependency state and verification choice;
- confirmation that no vector, MMU, Stage-2, GIC, timer-virtualization,
  SMP, EL1-entry, or guest-policy mechanism was introduced, and no
  platform-name branch exists;
- open items: W05/W08 supersession hooks (their designs extend or supersede
  values through recorded changes), P6's timer supersession — recorded, not
  resolved here.
