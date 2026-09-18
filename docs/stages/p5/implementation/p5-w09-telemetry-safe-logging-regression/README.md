# P5-W09 Telemetry, Safe Logging, and Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** P5 observability over the W06 result categories, the safe
logging boundary, and the determinate P4/P5 regression set, required by
[P5-W09](../../plans/p5-w09-telemetry-safe-logging-regression.md).  
**Owner/change context:** P5-W09 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W09. It integrates the
result categories delivered by W06 into per-VM counters and trace events
under the established P0 diagnostics governance, defines the safe-logging
redaction boundary, and assembles the long-term regression set from the
preserved P4 cases and the W07/W08 evidence, with determinate outcomes and
first-class non-success classes. It deliberately does **not** define a final
telemetry API or backend, a production audit service, a secret-logging
policy, or any CI-policy change, and it does not restate the P0 contracts —
it consumes them by citation.

A coding agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| Counter/event contracts, safe-debug context, redaction rules | [01 — telemetry and safe-logging contract](01-telemetry-and-safe-logging-contract.md) |
| Regression set, expectations, non-success classes, ordered steps, validation matrix | [02 — regression matrix and workflow](02-regression-matrix-and-workflow.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P5
task book, and the P5-W09 plan.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → frozen contracts →
P5-W09 plan → this design → Coding Guidelines. In particular:

- **P0-W12** ([logging/diagnostic baseline](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md))
  owns log-level semantics, human-log versus structured-trace boundaries,
  release/debug visibility and trimming, and minimum crash information.
  W09 classifies its outputs into those channels; it does not redefine
  levels or visibility rules.
- **P0-W13** ([trace-event namespace baseline](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md))
  owns event-domain classification, naming, version/compatibility, and the
  new-event review rule. W09 registers P5 events in the namespace the way
  it prescribes (its classification space explicitly includes capability);
  it does not invent an off-namespace naming scheme.
- The plan covers P5-T20, T21, and T25: per-VM call/result categories,
  success/failure and denied/invalid-reference/invalid-address accounting,
  safe debug context, no default Host VA/PA/object-pointer/
  raw-sensitive-buffer disclosure, and the P4/P5 positive, negative,
  revoke, isolation, overflow, and fuzz-smoke regression.
- W06 hands over the result-category taxonomy; W07 hands over the stable
  marker set; W08 hands over the smoke configuration. All three are
  consumed as named plan-level prerequisites with failure boundaries
  ([01 §1](01-telemetry-and-safe-logging-contract.md),
  [02 §1](02-regression-matrix-and-workflow.md)).
- P5-V15 also names the ABI/security documentation route; those documents
  are authored by the implementing packages (W02–W05) and published by W10.
  W09 verifies that the route and links exist and are factual; it does not
  author or freeze them.

Classification:

- **Required:** per-VM result-category counters with safe aggregation; the
  P5 trace-event registrations under the P0-W13 namespace; the safe-debug
  context definition and redaction rules; the composed regression set
  (inherited P4 rows + P5 scenario rows + fuzz smoke) with determinate
  expectations; the non-success class taxonomy; evidence destinations.
- **Reserved:** the final telemetry API/backend, transport, and retention;
  a production audit service; secret-logging policy; performance-baseline
  telemetry (owned by W08's record); CI enforcement of the regression set
  (P0-W20 and later stage work).
- **Out of Scope:** real-hardware validation; any claim that the QEMU
  runner proves architectural or hardware correctness; changes to the P4
  regression content; telemetry for P6+ mechanisms (GIC, timers, virtual
  interrupts, scheduler).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Per-VM call/result categories; success/failure, denied/invalid-reference/invalid-address accounting (P5-T20) | [telemetry contract](01-telemetry-and-safe-logging-contract.md) §2 | P5-V15 (W09-DV02, DV03) |
| Safe debug context; no default Host VA/PA/object-pointer/raw-buffer disclosure (P5-T21) | [telemetry contract](01-telemetry-and-safe-logging-contract.md) §4 | P5-V15 (W09-DV04) |
| P4/P5 positive, negative, revoke, isolation, overflow, fuzz-smoke regression (P5-T25) | [regression matrix](02-regression-matrix-and-workflow.md) §2–§3 | P5-V16 (W09-DV05–DV07) |
| Non-success outcomes diagnosable (timeout, incomplete, leaked-sensitive, unsupported environment) | [regression matrix](02-regression-matrix-and-workflow.md) §4 | P5-V16 (W09-DV08) |
| Telemetry/regression/limitation facts handed to W10 and P6+ consumers | [workflow](02-regression-matrix-and-workflow.md) handoff checklist | W09 closure review (W09-DV09) |
| ABI/security documentation route verified as factual (P5-V15 element) | [workflow](02-regression-matrix-and-workflow.md) step 5 | W09-DV04 review item |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p5-implementation-designs`,
`git ls-files`): no telemetry code, counter, trace event, logging
implementation, or regression runner exists in the tracked tree; the P0
diagnostics/namespace plans exist but are unimplemented; `docs/stages/p4/`
has no implementation or verification records, so no P4 regression evidence
exists yet; W06–W08 designs are parallel work on this branch (forward
references by slug); `../../verification/` holds only `.gitkeep`.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Result categories are observable per VM (P5-V15) | No counters or events exist | W06's implemented category assignment plus the counter/event contracts of [01 §2](01-telemetry-and-safe-logging-contract.md) | Categories are emitted by dispatch; observability integrates them without changing them | W06 (emission); W09 (integration) | W09-DV02/DV03 |
| Diagnostics avoid default Host-pointer/Guest-buffer disclosure (P5-V15) | No logging implementation exists | The redaction rules of [01 §4](01-telemetry-and-safe-logging-contract.md), applied to every W09-visible diagnostic path | Safe logging is a property of what the diagnostic paths emit by default | W09 (rules); P0-W12 (visibility semantics) | W09-DV04 review + regression leak check |
| P4 regression preserved (P5-V16) | No P4 implementation or evidence | The P4-W08 regression set as implemented and evidenced | Preserving a regression requires the regression to exist | P4-W08/W09 (assigned prerequisite) | W09-DV05 |
| P5 regression determinate (P5-V16) | No markers/harness yet | W07's marker set and W08's smoke config as delivered prerequisites | Regression rows must have fixed expectations | W07/W08 (assigned prerequisites) | W09-DV06/DV07 |
| Non-success outcomes diagnosable | Nothing tracked | The class taxonomy of [02 §4](02-regression-matrix-and-workflow.md) wired into the runner | A regression is trustworthy only if timeouts/incompleteness fail loudly | W09 | W09-DV08 |
| Evidence lands in governed locations | `.gitkeep` only | Records under `../../verification/p5-w09-telemetry-safe-logging-regression-verification.md` | Task book separates evidence from design | W09 | verification record |

No row requires inventing a telemetry backend or transport; the reserved
boundary keeps W09's integration minimal and governed by the P0 contracts.

## Resolved design decisions and their authority

1. **Consume, do not restate, the P0 contracts.** All level, visibility,
   trimming, crash-information, event-naming, and compatibility rules come
   from P0-W12/P0-W13 as cited. If P0-W12/W13 delivered differently from
   their plans (or not at all), that is a blocked prerequisite to record —
   W09 must not bootstrap a private diagnostics convention. Authority:
   plan prerequisite structure; task book §2 governance routing.
2. **Counters are per-VM, aggregated at read.** Each dispatched call
   increments exactly one per-VM, per-category counter; aggregation to
   global views happens only at read time, avoiding a global atomic
   hotspot. Counter updates are best-effort observations: a counter failure
   must never change a Guest-visible outcome. Authority: this design within
   plan scope; Coding Guidelines counter and observability rules; task
   book observability requirement.
3. **One event family, registered under the P0-W13 namespace.** P5
   dispatch/telemetry events form one named family in the namespace's
   capability domain, versioned per the namespace rules; fields are typed
   and stable; free-form strings are never the interface. Authority:
   P0-W13; ADR-048.
4. **Redaction is default-on, with a debug-gated exception.** Default
   diagnostics (any output reachable in release/production configuration)
   exclude Host virtual/physical addresses, Host object pointers, Guest
   buffer contents, and capability internal state; a developer-debug
   context may add more only as P0-W12's debug visibility rules permit,
   and the guest-marker channel's discipline (W07) is preserved unchanged.
   Authority: task book P5-V15 wording; P0-W12; W07 marker discipline.
5. **The regression set is composed, versioned, and closed.** Rows are:
   inherited P4 rows (unchanged content), P5 scenario rows (from W07's
   inventory), and the fuzz-smoke row (W08's smoke config). A row's
   expectation changes only through the owning package's maintenance rule,
   recorded — never inside W09. Authority: plan P5-T25; W07/W08 handoffs.
6. **Non-success is first-class.** Timeout, incomplete evidence, leaked-
   sensitive-information detection, and unsupported environment are named,
   recorded outcomes; the suite cannot pass with any of them present.
   Authority: plan work-sequence item 5; this design.
7. **No CI-policy change.** W09 delivers the regression boundary and its
   runner procedure; scheduling it in CI and required-check policy remain
   with P0-W20 and later stage governance. Authority: plan out-of-scope.

## Work breakdown and loading order

1. Load [01](01-telemetry-and-safe-logging-contract.md) for the counter and
   event contracts, safe-debug context, and redaction table.
2. Load [02](02-regression-matrix-and-workflow.md) for the regression set,
   non-success classes, ordered steps, validation matrix, and handoff
   checklist.
3. Record facts in
   `../p5-w09-telemetry-safe-logging-regression-record.md` (created when
   work starts) and runs in
   `../../verification/p5-w09-telemetry-safe-logging-regression-verification.md`.
   Neither this design nor any record may claim W09 complete; completion is
   claimed only in verification material, for what was actually run.

## Explicitly excluded interfaces

No final telemetry API/backend, transport, ring buffer, or retention
format; no audit service or audit-log schema; no secret-logging policy; no
performance counters or benchmark instrumentation (W08 owns the baseline
record); no new Guest-visible surface beyond the W02 boundary and the
unchanged W07 marker channel; no CI workflow or required-check change; no
P6+ telemetry (interrupt, timer, scheduler events). Any interface implied
by prose but not contracted in
[01](01-telemetry-and-safe-logging-contract.md) must not be implemented;
raise the gap as a design conflict.

## Downstream handoff

- **W10** ([closeout](../p5-w10-closeout-p6-handoff/README.md)) receives the
  regression/evidence index, the observability-limit record (what the P5
  telemetry does and does not provide), and the redaction-rule reference
  for the factual security documentation.
- **P6+ regression consumers** (through W10; operationally
  [P6-W13](../../../p6/plans/p6-w13-telemetry-regression-handoff.md))
  receive the declared P5 regression boundary: row list, expectations,
  environment constraints, and non-success classes — to be extended, never
  weakened, by later stages. P6 designs its own interrupt/timer telemetry
  and must not fold it into the P5 event family.
