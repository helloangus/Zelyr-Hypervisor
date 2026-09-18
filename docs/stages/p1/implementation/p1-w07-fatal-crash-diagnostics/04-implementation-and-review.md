# P1-W07 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

W07 can start only after the accepted W02/W09 contracts are available as
designs and the W05/W06 seams are settled designs, because the frame,
arming, and transport seams are consumed, not invented. Before changing
any file, the implementer verifies the mandatory reading (parent README),
inspects the current tree (`git ls-files`; confirm the P0 workspace/target
state and the accepted sibling designs), and records the assumed-contract
states from [01-architecture-and-state.md](01-architecture-and-state.md)
§8.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 extension seam cannot be exercised as recorded (e.g. the
  minimal body's guard cannot transfer cleanly) — raise the W02/W07
  coordination issue; do not create a parallel handler;
- the W05 frame lacks a field the report model requires — raise the
  W05/W07 coordination issue; W07 never captures registers itself;
- W09's `fail_phase` arms cannot call a diverging full report as this
  design assumes — raise the W09/W07 coordination issue; do not add a
  fourth entry kind;
- closure appears to require recovery, storage, symbolization, a telemetry
  transport, or a second report path — Out of Scope (parent README);
  stop.

## 2. Ordered implementation steps

### Step 1 — confirm seam sufficiency and record the required-field floor

Target: implementation record
(`../p1-w07-fatal-crash-diagnostics-record.md`, created in this step).

Work: verify W02's seam mechanics (body, guard, stop discipline, writer,
identity) against the transfer of §4; verify the W05 frame/classification
inputs cover every kind E field; verify W09's `fail_phase` and arming
placement; map the P0-W12/P0-W14 semantics onto the report posture;
record the marker literals decision point (to be fixed before the first
verdict-bearing run) and the buffer sizing arithmetic.

Suggested observation: read the sibling designs and P0 plans; no
repository change.

**Acceptance:** the record states each seam satisfied, or names the gap
and its owner.  
**Failure/blocker:** a seam gap stops the affected step (fail-closed); no
local adaptation.

### Step 2 — implement the report model and renderer

Target: the fatal-path module (physical placement per the P0 workspace
baseline; recorded).

Work: implement the field sets, availability labels, ordering, marker
prefixes, end marker, and bounded rendering per
[02-code-contracts-report-model.md](02-code-contracts-report-model.md).

Suggested observation: host-side unit evidence of the renderer (pure
functions over synthetic inputs) where the P0 host-test baseline permits;
otherwise contract inspection.

**Acceptance:** every kind's required fields render; degradation rules
hold (no fabricated values, no panics); ordering fixed; sizing recorded.  
**Failure/blocker:** a field gap stops the step; the model is never
loosened to fit available inputs.

### Step 3 — implement the three entries, the guard, and arming

Target: the fatal-path module.

Work: implement the superseding `p1_panic` body, `report_fatal_exception`,
`report_fatal_phase`, the guard, `arm_fatal_path`, and the readiness
declaration per
[03-code-contracts-fatal-path.md](03-code-contracts-fatal-path.md),
including the ownership-transfer checklist of architecture §4.

**Acceptance:** exactly one `#[panic_handler]`; guard semantics as
contracted (including the W05 layering rule); arming honest (no
fabricated readiness); every `unsafe` block carries its `SAFETY`
justification for the P0 unsafe inventory.  
**Failure/blocker:** a transfer or seam mismatch stops the step per §1;
no stub body and no dual handler.

### Step 4 — wire the transport seam and transition integration

Target: the fatal-path module plus the recorded contract points in
consumer wiring (owned by those packages).

Work: implement `emit_report_line` with the once-per-report preference;
confirm the W06 channel obligations (framing, exception-context
callability) and the W02 writer fallback; confirm with W08's contract
that both transport windows are mapping-class inputs; confirm the W09
`fail_phase` arms call `report_fatal_phase`.

**Acceptance:** the preference is evaluated once per report; both arms
work by contract pre- and post-MMU; no third transport exists.  
**Failure/blocker:** a missing consumer design records the deferred link
with its owner; no stub wiring.

### Step 5 — recursion and degradation review

Target: implementation record; this design's review tables.

Work: walk every entry path against the guard rules (architecture §6,
[fatal path](03-code-contracts-fatal-path.md) §6); walk the degradation
rules with synthetic partial-init inputs; verify no `unwrap`/`expect`/
allocation/format-panic exists anywhere on the path; verify the
call-direction invariant (architecture §2).

**Acceptance:** every rule passes with a pointer to code or record; any
failure is a recorded P1-V12 finding.  
**Failure/blocker:** an unremovable violation stops the package.

### Step 6 — acceptance evidence and closure

Target: verification record
(`../../verification/p1-w07-fatal-crash-diagnostics-verification.md`).

Work: perform the executable reviews of §3; record the deferred
executions (NC4 panic scenario; NC3/NC5/NC6 exception reports;
transition-failure observation) with their owning packages; complete the
handoff checklist.

**Acceptance:** the verification record distinguishes passed reviews,
deferred executions, and not-run items.  
**Failure/blocker:** a failed review is recorded as failed with
diagnosis; completion is not claimed around it.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W07-DV01 → W07 closure | Seam and floor review | step 1's checks against W02/W05/W09 sources and P0-W12/P0-W14 semantics | transfer mechanics exact; kind E inputs complete; `fail_phase` arms served; P0 floor mapped | readiness to build; not that the seams are implemented |
| W07-DV02 → P1-V11 | Report model review | inspect [report model](02-code-contracts-report-model.md) implementation | required fields per kind; availability labels truthful; ordering fixed; prefixes/end marker recorded; sizing arithmetic | reports carry the required context as designed; not their behavior under a real fault (NC3–NC6) |
| W07-DV03 → P1-V11, supports P1-V14 | Transition availability review | inspect §5 of [fatal path](03-code-contracts-fatal-path.md) and architecture §5 wiring | preference once-per-report; both transports contracted pre/post-MMU; arming honest | the diagnostic path is transition-independent by contract; not post-MMU liveness itself (W10/W11) |
| W07-DV04 → P1-V12 | Recursion and degradation review | step 5's walks | guard rules hold on every path; no unwrap/alloc/format-panic; second entry silent; degradation never fabricates | non-recursion and honesty by design; not observed recursive-fault behavior (W11 terminal checks) |
| W07-DV05 → P1-V11/P1-V12 | Crash/panic acceptance definition | map NC3–NC6 expectations and W10's R2 harness control onto the report model and marker classes | every scenario's expected class is expressible from this design's vocabulary alone; forbidden-class property defined | the acceptance evidence is defined before execution; execution belongs to W11/W10 |
| W07-DV06 → W07 closure | Consumability review | read the outputs as W08 (continuity obligation + kind F seam), W09 (arming + arms), W10 (classes), W11 (field list), W05 (resolved seam), W06 (renderer obligations), W12 (contract content) | each consumer can act without inventing W07 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. The
boot-dependent proofs (NC4 and the exception-report scenarios) are
deferred to W11 by contracted wiring, not omitted; until they exist,
P1-V11's and P1-V12's executed halves are unproven and no W07 artifact
may report otherwise. No validation here proves P1-V13 through P1-V21.

## 4. Error, security, and observability model

**Errors.** The fatal path is the error model's terminal tier: no retry,
no recovery, no degraded continue, no fallback work (plan out-of-scope
list). Its own failures are contained by structure: the guard (silent
second-entry stop), the transports' stall semantics, and the degradation
rules. Arming failure is the one self-reported failure, routed through
the non-recursive readiness boundary.

**Security.** The path adds no authorization decision and reads no secret
(the only reads are machine registers and static identity). It never
dereferences fault-controlled addresses, never echoes memory contents,
and never accepts input — reports are renderings of machine facts and
static labels. `unsafe` is confined to the guard/once-flag statics and
the one closed-register `CurrentEL` read, each with `SAFETY`
justifications in the P0 unsafe inventory. The guard transfer is itself a
security-relevant posture: the stage's panic path remains
termination-only and single-entry under the new owner.

**Observability.** The report is the stage's terminal observable: fixed
ordering, fixed token classes, raw addresses (symbolization Reserved).
Every failure class routes here or to its earlier-era form (W01
rejection, W05 pre-arm summary, panic route), so W10/W11 evidence can
attribute any captured failure to its phase and kind — the property
P1-V12's "not silently recurse into an unobservable crash" requires.

## 5. Handoff checklist

Before handing W07 to a reviewer, provide:

- the exact changed-file list and the module locations of every
  contracted item, including the superseded W02 body's location;
- the assumed-contract table as observed
  ([01-architecture-and-state.md](01-architecture-and-state.md) §8),
  including any recorded blocker or seam deviation;
- W07-DV01..DV06 evidence paths and run status, including the explicit
  deferred/not-run entries (NC3–NC6 execution → W11; harness-class
  observation → W10 R2);
- the implementation-selected values: marker/end-marker literals, report
  buffer capacity with sizing arithmetic, N/A label texts, readiness
  assertion list;
- confirmation that the ownership transfer left exactly one panic
  handler, that W02's minimal body is removed, and that all new `unsafe`
  is confined to the named statics and the `CurrentEL` read with
  `SAFETY` justifications filed in the P0 unsafe inventory process;
- confirmation that no recovery, storage, symbolization, telemetry, or
  second report path was introduced;
- open items: W08 window confirmation, W09 arm wiring confirmation, W06
  framing confirmation — recorded, not resolved here.
