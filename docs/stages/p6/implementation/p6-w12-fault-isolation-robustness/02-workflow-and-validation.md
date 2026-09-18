# P6-W12 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W12 detailed design](README.md).  
**Companions:** [01-robustness-case-matrix.md](01-robustness-case-matrix.md)
(case authority; containment classes §1).

## 1. Preconditions and failure boundary

Before any step, load the parent README's document set and inspect the tree
(`git ls-files`; P0 scaffold — no Host or Guest code exists yet). All
prerequisites are assumed contracts from plans and handoffs
([P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md),
[P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md)).

Stop and record a blocker (never patch a prerequisite, never add a recovery
framework) when:

- a W03/W07/W09 contract lacks the class a case exercises (spurious class,
  rejection semantics, orphan escalation) — record the deviate verdict;
  W12 exercises boundaries, it does not design them;
- exercising a case appears to require a production-path change, a new error
  type, panic machinery, a rate limiter, or a global recovery mechanism —
  out of scope; stop;
- the P4 isolation contract does not permit cross-VM probes — FI-A4 runs
  BLOCKED with the named limitation;
- storm limits cannot be declared honestly for the environment — FI-D runs
  NOT-RUN with the reason; limits are never invented after the fact.

## 2. Ordered implementation steps

### Step 1 — prerequisite reconciliation

Target: implementation record
(`../p6-w12-fault-isolation-robustness-record.md`, created in this step).

Work: read the W03 (physical IRQ classes and hooks), W07 (rejection
semantics on the vIRQ control surface), W09 (E1–E4 model, orphan
escalation), and W11 (observation asset) designs and records; verdicts per
case dependency; confirm the P5 error/rights semantics are the only
authorization boundary and that unsafe/diagnostic governance
([P0-W10](../../../p0/plans/p0-w10-unsafe-rust-governance.md),
[P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md)) is
in place for escalation paths.

**Acceptance:** every case row in
[01-robustness-case-matrix.md](01-robustness-case-matrix.md) §2 has a
confirmed, deviant, absent, or blocked verdict with evidence locations.  
**Failure/blocker:** deviate/absent verdicts route affected cases to
BLOCKED; they never justify modifying the owning package's mechanism.

### Step 2 — test-hook integration review

Target: the test-only hooks in the owning modules (consumed; gated per the
P0 feature/profile governance
([P0-W04](../../../p0/plans/p0-w04-build-profile-feature-governance.md))).

Work: verify each Injection point of the matrix exists, is test-gated, and
adds no production interface; verify induced-state hooks cannot fire in
non-test builds.

**Acceptance:** hook inventory recorded with gating evidence; no production
path modified.  
**Failure/blocker:** a missing hook is a gap routed to the owning package's
design; W12 does not add mechanism hooks itself beyond the gated test
surface its own run control needs.

### Step 3 — case execution readiness

Target: run control for FI-A–FI-D.

Work: prepare the declared inputs (crafted invalid operations, generated
malformed-input corpus within declared bounds, storm declarations), the
cross-VM probe wiring through the W11 observation asset, and the invariant
spot-checks (W09 I1–I5) used by FI-D.

**Acceptance:** every case can be executed or is explicitly BLOCKED/NOT-RUN
with the named reason, before any run.  
**Failure/blocker:** an unclassifiable case is a design gap; stop and amend.

### Step 4 — runs and evidence recording

Target: `../../verification/p6-w12-fault-isolation-robustness-verification.md`
(created when evidence exists).

Work: execute FI-A–FI-D per the matrix repetition semantics; record per-case
status (passed/failed/blocked/not-run), environment, declared storm limits,
containment class observed, artifacts, and Host correlation. Any FAIL is
recorded as evidence with diagnosis — never retried silently into a pass.

### Step 5 — containment and isolation review

Target: review findings in the implementation record.

Work: review outcomes against the containment classes of
[01-robustness-case-matrix.md](01-robustness-case-matrix.md) §1: no
GuestFault-local outcome escalated; no silent repair path observed; fatal
escalations carried full diagnostics; boundary statements (§3) present in
all draft handoff text.

**Acceptance:** each containment rule has a recorded verdict per executed
case.  
**Failure/blocker:** a containment violation is a mechanism-level defect —
record against the owning package's contract; it blocks W12 closure until
resolved or formally risk-recorded for W13's unresolved-items list.

### Step 6 — closure and handoff

Work: run the handoff checklist (§5). P6-V20–P6-V22 closure wording carries
its proof boundary: passing does not claim production DoS resistance, device
passthrough isolation, or real-hardware fault behavior.

## 3. Validation matrix

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W12-DV01 → P6-V21 | FI-A sub-cases FI-A1–FI-A3, FI-A5–FI-A7 | declared runs | structured rejection each; no Host/other-VM state change | implemented boundary rejects exercised invalid inputs; not fuzz-discovery completeness |
| W12-DV02 → P6-V21 | FI-A4 cross-VM owner probe | two-VM probe via the W11 asset | rejection in acting VM; other VM unaffected | cross-VM containment for the exercised operation; not full VM-isolation proof (P4 owns it) |
| W12-DV03 → P6-V20 | FI-B rows | declared runs via W03 hooks | classified, counted, contained; no Guest injection | safe spurious/unknown handling for exercised classes in QEMU; not real-controller misbehavior coverage |
| W12-DV04 | FI-C1/C2 escalation | induced-state runs | fatal-invariant path with full diagnostics; no silent absorption | visibility of designed impossible-state classes; not enumeration of all possible states |
| W12-DV05 | FI-C3 containment | forced-decode run | diagnostic-contain; no lifecycle mutation | containment of unrecognized patterns; not hardware bit-level coverage |
| W12-DV06 → P6-V22 | FI-D rows | declared storm runs | declared completion; invariants hold; no unexplained hang | bounded smoke robustness in QEMU; **not** production DoS resistance |
| W12-DV07 | hook gating review | inspect hook inventory and non-test builds | hooks unreachable outside test builds; no production interface added | honest injection semantics; not production-path behavior with hooks |
| W12-DV08 | containment-class review | audit outcomes vs [§1](01-robustness-case-matrix.md) classes | every observed outcome maps to its designed class | classification integrity; not completeness of the class model |

## 4. Error, security, and observability model

The error model is the containment-class table: GuestFault-local,
diagnostic-contain, fatal-invariant, consumed from
[P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md) and the
owning contracts (W09 E1–E4; W03 classes). Security position: the
authorization boundary is exclusively the P5 handle/rights/generation model
exercised through the authorized control path; ADR §19's no-degradation rule
is a review item in step 5; no case result may be used to claim DMA,
passthrough, or DoS security beyond its stated boundary. Observability:
per-case counters, escalation dumps (sized per the P1-W07 diagnostic
contract), storm run summaries, and telemetry correlation under the
P0-W12/P0-W13 contracts; raw log dumps are never the record surface. QEMU
results never prove real-hardware fault behavior.

## 5. Handoff checklist

Before handing W12 to a reviewer, provide:

- the exact changed-file list (run control, declared inputs, gated test
  hooks) and the hook-inventory gating evidence;
- step 1 prerequisite verdicts, especially any BLOCKED case (FI-A4 or
  others) with the named missing contract;
- W12-DV01–DV08 status (passed/failed/blocked/not-run) with artifact links,
  declared storm limits, and explicit FAIL rows with diagnosis;
- the boundary statements of
  [01-robustness-case-matrix.md](01-robustness-case-matrix.md) §3 verified
  present in all draft handoff text;
- confirmation that no new error type, panic machinery, recovery framework,
  rate limiter, passthrough-isolation logic, or production-path change was
  introduced;
- open items for W13 (P6-DOC-04 FI rows, unresolved risks, declared limits)
  and the P7/P8 security-review boundary statements — without resolving
  their contracts here;
- any recorded conflict or investigation with its owner.
