# P1-W09 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W09 detailed design](README.md).

## 1. Preconditions and failure boundary

W09 can start only after the W01–W08 mechanisms it sequences exist in the boot
path, because every adapter fails closed (parent README decision 7). Before
changing any file, the implementer verifies the mandatory reading (parent
README), inspects the current tree (`git ls-files`; confirm which prerequisite
designs, records, and code actually exist), and collects the W01–W08 contracts
from their plans and accepted designs.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a prerequisite package (W01–W08) has no accepted design or no implemented
  mechanism — W09 is blocked upstream; record the blocker; do not stub the
  phase;
- a prerequisite design fixes a mechanism entry, marker format, or runtime
  seam that contradicts [02-interface-contracts.md](02-interface-contracts.md)
  §8 — raise the conflict to the design owners; do not adapt silently;
- reaching closure appears to require Guest, SMP, GIC, discovery, allocator,
  or rollback machinery — that is out of scope (parent README); stop;
- a review finds a hidden dependency that cannot be removed by narrowing the
  phase's prerequisites — record it as a P1-V15 finding; it is a failure of
  the review, not an assumption to accept.

## 2. Ordered implementation steps

### Step 1 — collect and reconcile the supplying contracts

Target: implementation record (`../p1-w09-initialization-sequencing-record.md`,
created in this step).

Work: for each of W01–W08, extract from its plan and accepted design: the
mechanism entry point, its exit condition, its failure boundary, and any
diagnostic output it requires. Reconcile the phase order against
[01-init-state-machine.md](01-init-state-machine.md) §1–§2. Record the seam
table as actually observed, any deviation from §8, and the resolution of the
decision-1 ordering (or the raised coordination issue).

Suggested observation: read-only `git ls-files` and reading the accepted
designs under `../`; no repository change.

**Acceptance:** the record names, for every phase, the supplying design, the
adapter's target call, and the route owner; no phase row is "unknown".  
**Failure/blocker:** a missing or contradicting seam is a recorded blocker
(§1); the phase stays uncompiled.

### Step 2 — implement the phase type, tracker, and transition functions

Target: the boot-path module that the workspace layout fixes for early
initialization (the exact module tree is owned by the W02/W03 designs and the
P0 workspace baseline; this design owns only the items of
[02-interface-contracts.md](02-interface-contracts.md)).

Work: implement `InitPhase`, `LifecyclePosition`, `BootPhaseTracker`,
`phase_enter`, `phase_complete` exactly per their contracts, including the
encoding, the compare-exchange discipline, and the misuse route.

Suggested observation: host-side unit tests of the encoding and transition
table where the P0 host-test baseline permits; otherwise inspection against
the contracts.

**Acceptance:** every contract clause is either implemented or explicitly
recorded as deferred with its owner; no `static mut`, no allocation, no lock.  
**Failure/blocker:** a contract clause that cannot be implemented as written
(e.g., a toolchain limitation) is a design-conflict record, not a local
variant.

### Step 3 — wire the phase adapters and sequencer

Target: the adapter and `run_init_sequence` bodies.

Work: fill each adapter body with the single owning-package call identified in
step 1; wire `run_init_sequence` into the W02 runtime seam; implement
`fail_phase` per §7 and connect the W07/W02 routes as they exist.

**Acceptance:** the boot path compiles with all eight phases wired; skipping
any phase is impossible without a compile error; `entry`/`runtime` records are
written per decision 6.  
**Failure/blocker:** an adapter whose owning mechanism does not exist blocks
that step (fail-closed); no stub, default, or placeholder behavior.

### Step 4 — integrate markers and diagnostic availability

Target: marker recording, `materialize_replay`, `console_step` availability
wiring, stable-state emission point.

Work: implement the live-emission rule and replay per
[02-interface-contracts.md](02-interface-contracts.md) §§4–5; connect the W06
channel and availability signal; connect the W10 stable-marker emission point
(token per W10's design when fixed).

**Acceptance:** every phase transition is recorded; the replay enumerates
phases 1–6 at the materialization point; live markers cover all later events;
the W07 fatal path can read the phase.  
**Failure/blocker:** a marker that cannot be emitted without inventing output
before W06's channel is a design conflict; record it, do not add a parallel
output path.

### Step 5 — hidden-dependency and partial-init review

Target: the implementation record and this design's state-machine file.

Work: run the §5 rules of
[01-init-state-machine.md](01-init-state-machine.md) as a checklist over the
wired path: H1 declared prerequisites only, H2 no partial-init observation, H3
reserved boundaries, H4 route precedence, H6 phase attribution. Record each
check with its result.

**Acceptance:** every check passes with a pointer to the code or record that
satisfies it; any failure is a recorded P1-V15 finding with an owner.  
**Failure/blocker:** an unremovable hidden dependency stops the package (§1).

### Step 6 — lifecycle acceptance review and repeat-boot evidence expectations

Target: verification record
(`../../verification/p1-w09-initialization-sequencing-verification.md`).

Work: perform the W09-DV01..DV07 reviews of §3 that are executable now
(reviews), and record the repeat-boot evidence expectations (H5) as the input
W10 and W11 execute: same phase sequence, same marker token set, same terminal
state on every clean boot; any sequence deviation is a failure. Record what
was run, what was deferred to W10/W11 execution, and why.

**Acceptance:** the verification record distinguishes passed reviews, deferred
executions, and not-run items; the repeat-boot expectation is stated
measurably.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis;
completion is not claimed around it.

### Step 7 — closure and handoff

Work: confirm the handoff checklist (§5), the changed-file list, the seam
deviations (if any), and the downstream inputs for W10/W11/W12 and P2.
Completion is claimed only in the verification record, and only for what was
actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W09-DV01 → P1-V15 | Lifecycle review | read the wired path against
[01-init-state-machine.md](01-init-state-machine.md) §1–§2 | order, prerequisites, and exit conditions match the phase table; transitions are total, monotone, terminal-failure | the composed order is as designed; not that every supplying mechanism meets its own contract |
| W09-DV02 → P1-V15 (supports P1-V10) | Marker-coverage review | enumerate transitions vs marker rules in code | every transition recorded; replay emitted once at the materialization point; live rule positional, no availability flag | diagnosability of phases; not console hardware behavior (W06) |
| W09-DV03 → P1-V15 | Hidden-dependency / partial-init review | checklist H1–H3, H6 over adapter bodies and records | no phase consumes state outside its prerequisite set; no reserved-boundary reach-through | absence of hidden dependencies as designed; not future-package compliance |
| W09-DV04 → P1-V15 | Failure-routing review | matrix §4 vs route establishment per H4 | every route established by an earlier phase; misuse route bounded; no silent continuation | failure visibility by design; not the diagnostics' own content (W05/W07) |
| W09-DV05 → P1-V15 | Repeat-boot expectation definition | review the H5 statement and W10/W11 handoff wording | expectation is measurable (sequence + token set + terminal state) and consumers accept it | readiness for regression execution; not the boots themselves |
| W09-DV06 → P1-V15 | Boot-context safety review | inspect tracker/transition code against §3 concurrency clauses | single-writer discipline, wait-free reads, no unsafe beyond the audited CAS boundary, no allocation | safe P1 (single-CPU) execution; not SMP safety (P3) |
| W09-DV07 → W09 closure | Consumability review | read the design outputs as W10 (can I select a token and write verdict rules?), W11 (are fault phases and routes attributable?), W12 (is the lifecycle documentable?), P2 (is the stable environment declared?) | each consumer can act without inventing W09 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Reaching `stable` in a
QEMU boot is W10 evidence (P1-V16), not W09 closure; W09 reviews are
design-and-code reviews plus any host-side unit evidence the P0 baseline
permits. No validation here proves P1-V01–P1-V14 or P1-V16–P1-V21.

## 4. Error, security, and observability model

**Errors.** Phase failures are terminal and routed (state machine §4); misuse
of the sequencer is an invariant violation with its own bounded route; the
tracker never panics and decodes defensively. There is no retry, no recovery,
and no partial-success state anywhere in the lifecycle.

**Security.** P1 has no guest and no untrusted runtime input beyond what W01
validates at entry; W09 adds no authorization point. Security-relevant
postures it must preserve: no phase relaxes a control another phase
established (H1/H3); the unowned exception window before `exceptions.complete`
is recorded as a limitation rather than hidden; `unsafe` is confined to the
audited compare-exchange boundary and any seam the P0 unsafe governance
requires, each with a `SAFETY` justification per the Coding Guidelines.

**Observability.** The observability surface is exactly: phase labels and
events (W06 channel), the replay line, the tracker position read by the W07
fatal path, and the stable-state emission point consumed by W10. Diagnostics
identify the failed phase (H6). No additional debug prints, counters, or
telemetry are authorized by W09; structured telemetry remains P0/P2 scope.

## 5. Handoff checklist

Before handing W09 to a reviewer, provide:

- the exact changed-file list and the module locations of every §2 item;
- the seam table as observed (step 1), including any deviation from
  [02-interface-contracts.md](02-interface-contracts.md) §8 and its recorded
  resolution;
- W09-DV01..DV07 evidence paths and run status, including explicit not-run
  entries (QEMU boots, fault scenarios — W10/W11 scope);
- confirmation that no Guest/SMP/GIC/discovery/allocator mechanism, no global
  manager object, no rollback API, and no new public ABI was introduced;
- confirmation that all new `unsafe` (if any) is listed with `SAFETY`
  justifications and appears in the P0 unsafe inventory process;
- open items: W10 stable-marker token selection; W11 fault-phase attribution
  inputs; W12 initialization-contract content; the unowned pre-vector window
  limitation; any seam conflict raised per §1 — recorded, not resolved here.
