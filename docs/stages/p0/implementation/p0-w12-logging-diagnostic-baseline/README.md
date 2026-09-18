# P0-W12 Logging & Diagnostic Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The diagnostic channel semantics, stable log levels, release/debug
visibility and trimming rules, minimum fatal (panic/crash) information,
build/version identity association, and P1+ consumer constraints required by
[P0-W12](../../plans/p0-w12-logging-diagnostic-baseline.md).  
**Owner/change context:** P0-W12 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W12. It converts the bounded
work-package plan into one normative diagnostics-baseline document plus
minimal discovery wiring, cross-reviews with the trace-namespace and
version-metadata contracts, and a constraint walkthrough proving the baseline
binds P1 without prescribing telemetry implementation. It deliberately does
**not** implement any telemetry transport, ring buffer, or logging subsystem
(plan out-of-scope), does **not** define trace-event names, namespace, or
compatibility rules ([P0-W13](../p0-w13-trace-event-namespace-baseline/README.md)
owns those), does **not** define identity field schemas or formats
([P0-W16](../p0-w16-version-build-metadata-baseline/README.md) owns those),
does **not** define the failure taxonomy ([P0-W14](../p0-w14-panic-failure-classification/README.md)
owns it; W12 consumes it), and does **not** authorize any macro, type, crate,
or code surface.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting files needed for its assigned step: the
channel-and-level contract, the crash/identity/consumer-constraint contract,
and the implementation workflow. Before editing it must also follow the Coding
Guidelines preflight, including the repository `AGENTS.md`, documentation
index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
P0 task book, and P0-W12 plan. This document is the proposed detailed design
for those changes; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W12 plan → this design
→ Coding Guidelines. In particular:

- ADR principle 10 (observability is not an afterthought; structured
  telemetry is reserved from early stages) and ADR-048 (structured
  tracing/metrics are first-class, with a named minimum coverage) fix why this
  baseline exists; §12 fixes its shape: trace events must be compile-time
  trimmable and runtime filterable, production/benchmark builds can disable
  high-overhead events, and panic policy separates fatal hypervisor invariant
  failures from guest-caused faults (the taxonomy itself is W14's).
- The task-book outcome for W12 is: logging, tracing, metrics, panic/crash,
  release, and version-metadata semantics are **governed** (P0-V09, P0-V14).
  Governance means semantics and boundaries that a P1 design can be held to —
  not a logging implementation.
- Prerequisite status: [W04](../p0-w04-build-profile-feature-governance/README.md)
  (profile/feature semantics), [W05](../p0-w05-documentation-baseline/README.md)
  (document conventions), and [W06](../p0-w06-adr-governance/README.md)
  (escalation mechanism) are proposed designs, not deliveries. W04's design
  already hands W12 the boundary that release-visibility decisions reference
  its profile semantics; this design consumes that boundary by subject. The
  fallback rule is the standard sibling assumption: implement against the
  current tree's existing mandates; delivered prerequisite rules take
  precedence; substantive conflicts are raised, not absorbed (workflow §1).
- Consumer alignment: W13 consumes the channel boundaries fixed here; W16's
  design already declares a two-way cross-review obligation with W12 over the
  per-record identity requirement; P1 consume the semantics for early-console
  logging and crash diagnostics (P1-W06, P1-W07 per the P1 plan index).

Classification: the diagnostics-baseline document (channel taxonomy, level
set, visibility and trimming classes, minimum fatal information, identity
association rule, P1+ constraint list, boundary rules with W13/W14/W16) and
its discovery wiring are **Required** for W12 closure. The concrete logging
API/macro surface, transports (P1 early console first), runtime filter
mechanism, ring buffers, metrics aggregation, and the first actual trace
events are **Reserved** to the designs that introduce them. Any logging or
telemetry implementation, transport, buffer, wire format, CI wiring, crate,
or module is **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Stable log levels and per-channel purposes with non-substitution rules (work item 1) | [Channel and level contract](01-diagnostic-channels-and-levels.md) §2–§3 | P0-V09 (W12-DV01) |
| Release/debug visibility, trimming, and minimum panic/crash information principles (work item 2) | [Channel and level contract](01-diagnostic-channels-and-levels.md) §4, [crash and identity contract](02-crash-identity-and-consumer-constraints.md) §1 | P0-V09/P0-V14 (W12-DV02, DV03) |
| Per-diagnostic build/version identity requirement aligned with W16 (work item 3) | [crash and identity contract](02-crash-identity-and-consumer-constraints.md) §2 | P0-V14 (W12-DV04) |
| Baseline constrains P1 without deciding telemetry implementation (work item 4) | [crash and identity contract](02-crash-identity-and-consumer-constraints.md) §3, [workflow](03-implementation-and-review.md) step 4 | P0-V09 (W12-DV05) |
| Document discoverability and coherence | [workflow](03-implementation-and-review.md) step 3 | P0-V09 (W12-DV06) |
| Downstream consumability by W13, W16, and P1 bring-up/telemetry work | [workflow](03-implementation-and-review.md) handoff checklist | W12 closure review (W12-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at `4e631ee`): no tracked document defines a
log level, a diagnostic channel, a visibility class, a fatal-information
minimum, or an identity-association rule. The ADR states the observability
principles; the Coding Guidelines state that telemetry is a designed
interface, but neither is an applicable diagnostic contract. No Rust source
exists, so no logging call exists to govern. W01 is completed; W04–W06
(prerequisites) and W13/W16 (consumers) are proposed designs, several of them
untracked parallel work referenced by slug only. Each ledger row below states
the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Stable log levels exist | No level set anywhere; nothing for P1 to share | Level set with fixed semantics, audience, and retention defaults | P1 bring-up and later runtime must share one vocabulary or channels degrade into debug prints (ADR principle 10's warning) | W12 (this design) within the plan's 建立稳定日志等级 mandate | W12-DV01 |
| Each diagnostic channel has a purpose, and channels cannot substitute for each other | No channel definitions | Channel taxonomy with purpose, required content semantics, and per-channel non-substitution rules | Channel conflation is the failure mode the plan names (避免把它们互相替代) | W12 | W12-DV01 |
| Release/debug visibility and trimming rules (work item 2) | ADR §12 states trimmable/filterable; no operational classes exist | Three visibility classes mapped to levels/channels, with trimming expressed through W04-classified build selections, never ad-hoc switches | Production/benchmark disablement (§12) must be a governed property, not per-call folklore | W12 classes; W04 profile semantics | W12-DV02 |
| Minimum panic/crash information principle (work item 2) | Absent; P1-W07 would otherwise invent it | Minimum fatal-information sections for panic messages and crash dumps | "最低 panic/crash 信息原则" is an explicit plan deliverable, needed before P1's crash design | W12; failure classes from W14 by subject | W12-DV03 |
| Per-diagnostic identity association aligned with W16 (work item 3) | W16's proposed design declares the minimum identity set and a cross-review with W12; no W12-side requirement exists | Association rule consuming W16's minimum set by reference, plus the two-way cross-review obligation | Diagnostics that cannot be tied to a build cannot be traced to source (P0-V14's purpose) | W12 requires the property; W16 owns the fields | W12-DV04 |
| Baseline constrains P1 without deciding telemetry implementation (work item 4) | Not exercised | Must/must-not constraint lists for P1+ designs plus a walkthrough | Acceptance demands demonstrated constraint-without-prescription | W12 | W12-DV05 |
| Discoverability (P0-V09) | No routing row | `docs/README.md` routing row and stage implementation index row | Entry points must lead to the baseline | W12 | W12-DV06 |
| W13/W16/P1 can consume | No contract to consume | Handoff statements per consumer | Consumers must know what is governed here versus in their own designs | W12 delivers; consumers own their contracts | W12-DV07 |

No row requires choosing a transport, buffer, format, dependency, or code
surface, so no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Normative home:** `docs/development/diagnostics-baseline.md` is the sole
   normative home of diagnostic channel semantics, level semantics,
   visibility/trimming classes, fatal-information minimums, identity
   association, and consumer constraints, following the established
   development-policy pattern (W02/W04/W05/W07/W11).
   [W05](../p0-w05-documentation-baseline/README.md) may re-home it; semantic
   ownership stays with the document.
2. **Four channels.** Human-readable log, structured trace, metrics, and
   fatal/crash diagnostics are the only P0-governed channels. Each has a
   purpose, an audience, required content semantics, and an explicit
   non-substitution list (contract 01 §2). Fewer channels cannot express the
   ADR-048 coverage; more channels are a policy decision under the document's
   thresholds.
3. **Level set: five levels.** Trace, Debug, Info, Warn, Error, with fixed
   semantics (contract 01 §3). The plan authorizes 建立稳定日志等级; the
   *semantics and set* are the baseline. The future API surface (macro or
   item names) is named by the consuming stage's design within its own
   namespace rules — this design fixes no code identifier.
4. **Three visibility classes.** Always-retained (Error, and the fatal
   channel's minimum), removable-by-build-selection (Warn, Info), and
   compiled-out-by-default (Debug, Trace, and high-overhead trace events).
   Trimming selections must be classified under
   [W04](../p0-w04-build-profile-feature-governance/README.md) semantics —
   never ad-hoc conditional-compilation names — and runtime filterability is
   a binding requirement on the future logging subsystem design, not an
   implementation here (contract 01 §4).
5. **Minimum fatal information.** Every panic message and crash dump must
   permit attributing: the failure classification (per
   [W14](../p0-w14-panic-failure-classification/README.md)'s taxonomy, by
   subject), the site, and the build identity (W16's minimum set, by
   reference); plus site-appropriate state named by the owning design. Guest-
   derived data must never be presented as trusted context (contract 02 §1).
6. **Identity association is a property requirement.** Every diagnostic
   record must be associable with W16's minimum identity set: fatal output
   carries it inline; other channels must be associable via the artifact or
   session that produced them. Association mechanics are owned by the
   producing design (W16 defers them); W12 owns the property and the
   cross-review obligation (contract 02 §2).
7. **P1+ constraints are explicit must/must-not lists.** A future logging
   design must define its concrete surface within these level/visibility
   semantics, bind the P1 early console as its first transport, implement
   runtime filtering for trimmable classes, and respect channel boundaries;
   it must not create parallel logging paths after adoption, exit fatally
   outside [W14](../p0-w14-panic-failure-classification/README.md)'s rules,
   encode machine-parsed fields in the human log, or route around trimming
   (contract 02 §3). The transitional pre-subsystem early-console path is
   bounded by a stated rule so P1-W06 has governance from day one.
8. **Boundary with W13.** W12 owns channel semantics and visibility; W13 owns
   event names, namespace, and compatibility. W12 states the one-fact-one-
   canonical-name principle; W13 operationalizes it (contract 02 §4).
9. **Boundary with W14.** The fatal channel is reserved for exits W14's
   taxonomy classifies as hypervisor-invariant fatal; guest-caused and
   recoverable classes are VM-scoped diagnostics, never fatal-channel
   content. W14's classification is consumed by subject; if its delivered
   taxonomy differs materially, the cross-review in workflow step 5 reconciles
   or raises the conflict.

## Work breakdown and loading order

1. Read [the channel and level contract](01-diagnostic-channels-and-levels.md)
   for the artifact groups, channel taxonomy, level semantics, and
   visibility/trimming classes.
2. Read [the crash and identity contract](02-crash-identity-and-consumer-constraints.md)
   before writing anything about panic/crash content, identity association, or
   consumer constraints.
3. Apply the changes in the order stated in
   [the implementation workflow](03-implementation-and-review.md): verify
   prerequisite surfaces, author the baseline document, wire discovery, run
   the constraint walkthrough, run the W13/W16 cross-reviews, close.
4. Store actual review commands, output, environment, and result in
   `../../verification/p0-w12-logging-diagnostic-baseline-verification.md`,
   and record changed artifacts and any deviation in
   `../p0-w12-logging-diagnostic-baseline-record.md` only when implementation
   begins. Neither this design nor a written record may claim W12 complete.

## Design-level state and lifecycle

W12 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked baseline document plus its discovery
links. Their documentary lifecycle:

```text
no diagnostic semantics anywhere
  -> diagnostics-baseline.md committed (channels, levels, visibility,
     fatal minimums, identity association, consumer constraints, boundaries)
  -> docs/README.md routing row + stage index row committed
  -> constraint walkthrough evidenced (hypothetical P1 scenarios bound)
  -> W13/W16 cross-reviews recorded (or blocked-with-surface)
  -> P1 logging/crash designs cite the baseline; the first logging design
     names the API surface within these semantics
  -> baseline changes only through the document's thresholds
```

The baseline document owns every diagnostic-semantic statement. A future
logging subsystem whose semantics contradict it is a review failure to
reconcile in the consuming design, not a local choice; where reconciliation
would change an ADR-level property (for example dropping the trim/filter
requirement of §12), the conflict is labelled `ADR Required`.

## Explicitly excluded interfaces

No macro, function, type, trait, module, crate, transport, ring buffer,
format, encoding, CI workflow, or public API is designed or authorized by
W12. The level names in contract 01 §3 are semantic level names, not code
identifiers; the identity fields in contract 02 §2 are W16's by reference,
not restated as schema. Trace-event names belong to W13's registry. Adding
any excluded item under W12 is a scope conflict to be raised at review.

## Downstream handoff

- **W13** receives the channel boundaries and the one-fact-one-name principle
  its namespace coordinates within; W13 owns all naming and compatibility
  rules and must not re-decide channel semantics.
- **W16** receives the per-record association requirement its minimum
  identity set satisfies; the two-way cross-review is W16's declared closure
  condition and W12's consumption point. W12 must not have defined any
  identity field.
- **P1** (early-console logging and crash diagnostics per the P1 plan index)
  receives the level semantics, visibility classes, fatal-information
  minimum, the transitional early-console rule, and the constraint lists its
  designs must satisfy and cite.
- **W04** is honored in reverse: W12's trimming rules reference W04's
  classified build selections; W12 adds no switch of its own.
- **W14** receives the fatal-channel reservation: its invariant class is the
  only producer of fatal-channel content; other classes have VM-scoped or
  metric treatments named in contract 02 §4.
- **W10** is unaffected except that its SAFETY failure-class references (per
  W14) and any diagnostic side effects of unsafe boundaries must respect the
  channel boundaries fixed here.
- **W05** may re-home the baseline document; **W20** may later consume
  mechanical check candidates (none defined here); **W22** maps this
  deliverable into the P1 handoff package by its own plan scope.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
