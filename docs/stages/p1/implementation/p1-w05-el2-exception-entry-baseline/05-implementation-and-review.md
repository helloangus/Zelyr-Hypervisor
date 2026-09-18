# P1-W05 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W05 detailed design](README.md).

## 1. Preconditions and failure boundary

W05 can start only after the P0 target/build baseline exists in implementable
form and the accepted W02/W04/W09 contracts are available as designs, because
every step compiles against their seams. Before changing any file, the
implementer verifies the mandatory reading (parent README), inspects the
current tree (`git ls-files`; confirm the P0 workspace/target state and the
accepted sibling designs), and records the assumed-contract states from
[01-architecture-and-state.md](01-architecture-and-state.md) §7.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W04 declaration API cannot express one of the §3 assertions, or a
  category W05 must assert is missing from the baseline — raise the
  W04/W05 coordination issue; do not read W04-owned control registers to
  work around it;
- the W07 seam (`report_fatal_exception`, readiness declaration) or the W06
  channel preference cannot be expressed as recorded in this design —
  record the deferred branch and the conflict per the W02 decision-6
  precedent; no stub report and no fake readiness;
- closure appears to require interrupt acknowledge/EOI, a GIC access, a
  return path, or timer delivery — those are Out of Scope (parent README);
  stop;
- an EC-class table entry cannot be confirmed against the recorded
  architecture revision — record the uncertainty in the implementation
  record; do not guess encodings into the table.

## 2. Ordered implementation steps

### Step 1 — confirm prerequisite seams are sufficient

Target: implementation record
(`../p1-w05-el2-exception-entry-baseline-record.md`, created in this step).

Work: verify the W04 declaration API covers the assertion set; verify W09's
`exceptions_step` adapter shape and the matrix rows W05 implements; verify
W02's writer and formatter contracts cover the pre-arm summary; record the
architecture revision and the EC encodings for the syndrome table; record
any deferred seam (W07 branch, W06 preference) with its owner.

Suggested observation: read the sibling designs; no repository change.

**Acceptance:** the record states each seam satisfied, or names the gap and
its owner.  
**Failure/blocker:** a seam gap stops the affected step (fail-closed, per
the W09 adapter discipline); no local adaptation.

### Step 2 — implement vector install and the declaration

Target: the exceptions module (physical placement per the P0 workspace
baseline; recorded).

Work: implement `install_el2_exception_entry`, the baseline assertions, the
`VBAR_EL2` write/read-back, the table region symbols, and the declaration
per [02-code-contracts-vector-install.md](02-code-contracts-vector-install.md).

Suggested observation: review against the contract; where the tree links
and the P0 artifact baseline provides inspection tooling, verify the table
symbol alignment (2 KiB) and size at link level.

**Acceptance:** the phase body asserts, installs once, verifies, and
declares; every `unsafe` block carries its `SAFETY` justification for the
P0 unsafe inventory.  
**Failure/blocker:** a W04 assertion gap routes as designed (fatal,
phase-attributed); a link/layout gap is the recorded P0 blocker.

### Step 3 — implement the entry stubs and capture

Target: the entry assembly and capture module.

Work: implement the sixteen stubs and `p1_exception_capture_entry` with the
`ExceptionFrame` exactly per
[03-code-contracts-entry-capture.md](03-code-contracts-entry-capture.md),
including the §3 prohibited-content list and the bounded stop.

Suggested observation: review of the assembly against the contract;
entry-coverage checklist (16 stubs, coordinates, no fall-through).

**Acceptance:** every (origin, category) pair has a valid entry; capture is
bounded and FP-free; the guard closes recursion at the earliest point.  
**Failure/blocker:** any prohibited content or uncovered entry fails the
step; fix the code, never the coverage rule.

### Step 4 — implement classification and routing

Target: the classification module.

Work: implement the EC-class table with validity flags, the disposition
mapping, `route_classified`, the pre-arm summary, and `VectorError` per
[04-code-contracts-classification-routing.md](04-code-contracts-classification-routing.md).
If W07's items do not exist yet, the armed branch records the deferred link
and stays uncompiled — the contracted state, not a failure (W02 decision-6
precedent).

**Acceptance:** classification is total and deterministic; routing reaches
the stop through exactly one of the two contracted paths; no second output
path.  
**Failure/blocker:** a W07 seam mismatch is raised to both owners; silent
adaptation is prohibited.

### Step 5 — boundary and ownership review

Target: implementation record; this design's review tables.

Work: run the prohibited-content walks (entry §3, boundary §5 of the
install contracts), the state-ownership register
([01-architecture-and-state.md](01-architecture-and-state.md) §3), the
state machine rules R1–R5, and the one-owner check on `VBAR_EL2` (no
second writer anywhere in the tree).

**Acceptance:** every rule passes with a pointer to code or record; any
failure is a recorded P1-V09/P1-V12 finding.  
**Failure/blocker:** an unremovable violation stops the package.

### Step 6 — acceptance evidence and closure

Target: verification record
(`../../verification/p1-w05-el2-exception-entry-baseline-verification.md`).

Work: perform the executable reviews of §3; record the deferred executions
(NC3/NC6 fault scenarios → W11; boot-integrated observation → W10) with
their owning packages, exactly as W02/W04 handle deferred boot evidence;
complete the handoff checklist.

**Acceptance:** the verification record distinguishes passed reviews,
deferred executions, and not-run items.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis;
completion is not claimed around it.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W05-DV01 → P1-V08 | Coverage review | inspect the table per [entry capture](03-code-contracts-entry-capture.md) §1 and the region symbols per [install](02-code-contracts-vector-install.md) §4 | 16 valid entries; alignment; origin/category stamping; no fall-through; region symbols present | all categories and origins have valid entry paths as designed; not that hardware delivers every category |
| W05-DV02 → P1-V08 | Origin/context model review | inspect architecture §2 and [classification](04-code-contracts-classification-routing.md) §1–§2 | legitimate-origin table matches the W02/W04-established machine state; EC table closed with validity flags | classification vocabulary is complete for P1; not syndrome-level correctness on real hardware |
| W05-DV03 → P1-V08/V09 | Capture and disposition review | inspect the frame contract and §3 mapping | fixed field set; FP-free; total deterministic mapping; summary carries syndrome/PC/fault address/phase | bounded diagnostic context exists; not its behavior under a real fault (NC3/NC6) |
| W05-DV04 → P1-V08 | Baseline integration review | inspect §1/§3 of [install](02-code-contracts-vector-install.md) against W04's API and values | assertions cover C1/C2/C3/C4 via the declaration API; VBAR write verified; no control rewritten | valid-entry behavior builds on the recorded baseline; not baseline values themselves (W04's evidence) |
| W05-DV05 → P1-V09/P1-V12 | Recursion and boundary review | walk R1–R5, the guard, the prohibited-content lists, and the ownership register | recursion terminates at the earliest point with no output; no return path; one writer per state; audited boundary exactly as named | the path cannot recurse unboundedly by design; not observed behavior under a real recursive fault |
| W05-DV06 → P1-V09 | Intentional/unexpected acceptance definition | map [W11's NC3/NC6 expectations](../p1-w11-negative-fault-validation/01-fault-scenario-matrix.md) and the §5 token classes onto this design's vocabulary | NC3's summary/report classes and NC6's classification classes are expressible from this design's vocabulary alone | the acceptance evidence is defined before execution; execution belongs to W11 |
| W05-DV07 → W05 closure | Consumability review | read the outputs as W06 (exception-context callability premise), W07 (frame + seam), W08 (region + attributes), W09 (phase body + window narrowing), W10 (token classes), W11 (NC3/NC6 targets), W12 (contract content) | each consumer can act without inventing W05 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. The
QEMU-dependent proofs (fault scenarios NC3/NC6, boot-integrated vector
behavior) are deferred to W11/W10 by contracted wiring, not omitted; until
they exist, P1-V09's executed half is unproven and no W05 artifact may
report otherwise. No validation here proves P1-V10 through P1-V21.

## 4. Error, security, and observability model

**Errors.** W05 has exactly three failure postures: install-time failures
(`VectorError` → panic route, phase-attributed `exceptions`); classified
exception events (terminal via §4 routing — full report post-arm, bounded
summary pre-arm); and recursive/unowned events (silent bounded stop).
There is no retry, no recovery, no degraded mode, no return path. Every
routed failure leaves the phase identifier recoverable (tracker read in the
summary; W07's phase field post-arm).

**Security.** The exception path adds no authorization decision (nothing
below EL2 exists in P1 to authorize). Security-relevant postures: masks are
never modified by the path; the GIC is never touched (no acknowledge, no
EOI); rendered content is machine facts and static labels only; no
fault-controlled memory is dereferenced; `unsafe` is confined to the named
audited boundary plus the entry assembly, each with `SAFETY` justifications
in the P0 unsafe inventory. The deny-by-default W04 posture is what makes
every taken exception diagnostic rather than silent — the vector path is
its enforcement arm.

**Observability.** The observable surface is: the `exceptions` phase
records via W09's tracker (markers per W09/W06 once the channel exists);
the pre-arm summary through W02's writer; the post-arm report through
W07's model; and the declaration status consumed by later phases. W05 adds
no debug prints, counters, or telemetry; structured observability remains
P0/P2 scope.

## 5. Handoff checklist

Before handing W05 to a reviewer, provide:

- the exact changed-file list and the module locations of every contracted
  item;
- the assumed-contract table as observed
  ([01-architecture-and-state.md](01-architecture-and-state.md) §7),
  including any recorded blocker or seam deviation;
- W05-DV01..DV07 evidence paths and run status, including the explicit
  deferred/not-run entries (NC3/NC6 execution → W11; integrated-path
  observation → W10);
- the implementation-selected values: architecture revision and EC
  encodings, token/prefix literals, region symbol names, frame sizing;
- confirmation that all new `unsafe` is confined to the named audited
  boundary and the entry assembly with `SAFETY` justifications filed in the
  P0 unsafe inventory process;
- confirmation that no GIC access, acknowledge/EOI, return path, mask
  change, timer delivery, second output path, or public ABI was introduced;
- open items: the deferred W07 armed branch, the W06 channel preference
  consumption, W08's region attribute confirmation — recorded, not
  resolved here.
