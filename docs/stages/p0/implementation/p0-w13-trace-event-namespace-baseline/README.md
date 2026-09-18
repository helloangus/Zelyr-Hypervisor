# P0-W13 Trace Event Namespace Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The trace-event domain classification space, canonical naming
rules, namespace/version and compatibility governance, canonical-event
registry, and new-event review rules required by
[P0-W13](../../plans/p0-w13-trace-event-namespace-baseline.md).  
**Owner/change context:** P0-W13 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W13. It converts the bounded
work-package plan into one normative trace-event-namespace document (rules,
the reserved domain registry, and an empty canonical-event registry), minimal
discovery wiring, and an attribution drill rehearsed on clearly hypothetical
future P1 event categories. It deliberately does **not** define any event
field, payload, ring buffer, transport, or instrumentation (plan
out-of-scope), does **not** declare a single actual event (P0 has no telemetry
implementation, and adding one is a future design's decision), does **not**
define channel semantics, levels, or visibility ([P0-W12](../p0-w12-logging-diagnostic-baseline/README.md)
owns those), and does **not** choose a wire or storage encoding for names
(owned by the telemetry implementation design that first serializes them).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), P0 task
book, and P0-W13 plan. This document is the proposed detailed design for those
changes; it is not a completion record and contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W13 plan → this
design → Coding Guidelines. In particular:

- ADR-048 (structured tracing as a first-class capability with a named
  minimum coverage), ADR principle 10 (telemetry reserved from early
  stages), and §12 (events compile-time trimmable and runtime filterable)
  fix why a namespace is governed before any event exists: event names are a
  compatibility surface, and retrofitting naming onto shipped events is the
  failure this package prevents (避免临时字符串成为接口).
- The task-book outcome for W13 is: trace-event namespace and compatibility
  governance are **defined without a telemetry implementation** (P0-V09), and
  new events can be consistently classified.
- The plan names the classification areas the domain space must cover: boot,
  cpu, vm, vcpu, scheduler, memory, stage2, irq, device, virtio, ipc,
  capability, platform, and management. These fourteen domains are plan
  scope, not this design's invention.
- Prerequisite status: [W05](../p0-w05-documentation-baseline/README.md)
  (document conventions) and [W12](../p0-w12-logging-diagnostic-baseline/README.md)
  (channel boundaries) are proposed designs, not deliveries. W12's design
  hands W13 the channel boundaries and the one-fact-one-canonical-name
  principle by subject; this design consumes them by that reference. The
  standard sibling fallback applies: implement against the current tree's
  existing mandates; delivered prerequisite rules take precedence; substantive
  conflicts are raised, not absorbed (workflow §1).
- Consumers: P1+ telemetry designs (per the P1/P4+ plan indexes' telemetry
  and observability packages). No P0 package consumes event names, because
  none may exist yet.

Classification: the namespace document (fourteen-domain registry, naming
grammar, attribution and new-event review rules, compatibility and
deprecation rules, empty canonical-event registry) and its discovery wiring
are **Required** for W13 closure. The first actual event declarations (P1+
owning designs), name encoding on any wire/storage format, and registry-
validation tooling (a future [W07](../p0-w07-development-quality-gates/README.md)/[W20](../p0-w20-ci-baseline/README.md)
candidate) are **Reserved** with triggers. Event fields, payloads, ring
buffers, transports, instrumentation, metrics/log content, and all code are
**Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Domain classification space covering the fourteen named areas (work item 1) | [Namespace contract](01-namespace-contract.md) §3 | P0-V09 (W13-DV01) |
| Naming, attribution, compatibility, and deprecation rules; no ad-hoc strings as interfaces (work item 2) | [Namespace contract](01-namespace-contract.md) §4–§6 | P0-V09 (W13-DV02) |
| Log/metrics/trace boundary coordination against undeclared duplicate names (work item 3) | [Namespace contract](01-namespace-contract.md) §7 | P0-V09 (W13-DV03) |
| Namespace attribution rehearsal on future P1 categories, without adding events (work item 4) | [workflow](02-implementation-and-review.md) step 4 | P0-V09 (W13-DV04) |
| Document discoverability and coherence | [workflow](02-implementation-and-review.md) step 3 | P0-V09 (W13-DV05) |
| Downstream consumability by P1+ telemetry designs | [workflow](02-implementation-and-review.md) handoff checklist | W13 closure review (W13-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at `4e631ee`): no tracked document defines a
trace-event name, domain, namespace rule, or compatibility rule. The ADR
names telemetry coverage areas but no naming governance; no telemetry
implementation exists, so no event name exists to migrate. W01 is completed;
W05 and W12 (prerequisites) are proposed designs, W12 untracked parallel work
referenced by slug. Each ledger row below states the missing foundation the
plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Consistent, evolvable event naming exists before telemetry | Nothing governs naming; ADR-048 names subjects only | Domain registry covering the plan's fourteen areas, each with a subject definition | Consistent attribution requires stable domain semantics decided before events ship | W13 (this design) within the plan's domain list | W13-DV01 |
| Names cannot silently become ad-hoc string interfaces | No rule exists | Naming grammar plus a canonical-event registry and declaration rule | "避免临时字符串成为接口" requires names to be declared, recorded, and change-controlled | W13 | W13-DV02 |
| Version/compatibility handling for names | ADR-040's versioning principle is stated for schema/machine/ABI; nothing applies it to event names | Namespace version field and compatibility/deprecation rules for declared names | An evolvable namespace needs declared change mechanics, not folklore | W13 within ADR-040's spirit; ADR-040 untouched | W13-DV02 |
| Log/metrics/trace do not mint duplicate names for one fact | [W12](../p0-w12-logging-diagnostic-baseline/README.md)'s design states the principle; no naming-side rule exists | One-fact-one-canonical-name rule plus the reference rule for logs/metrics | Undeclared duplicate names would give one semantic fact several identities | W13 rule; W12 channel boundaries | W13-DV03 |
| New events can be classified consistently | Not exercised anywhere | Attribution drill on hypothetical P1 categories (labeled, no registry entries) | Acceptance demands demonstrated consistent classification | W13 | W13-DV04 |
| Discoverability (P0-V09) | No routing row | `docs/README.md` routing row and stage index row | Entry points must lead to the namespace document | W13 | W13-DV05 |
| P1+ telemetry can consume | Nothing to consume | Handoff statements to P1+ telemetry designs | Consumers must know how to declare names and what is not theirs to decide | W13 delivers; consumers declare | W13-DV06 |

No row requires defining a field, payload, buffer, transport, or any event,
so no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Normative home:** `docs/development/trace-event-namespace.md` is the
   sole normative home of the domain registry, grammar, declaration rules,
   compatibility rules, and the canonical-event registry, following the
   development-policy pattern (W02/W04/W05/W07/W11/W12).
   [W05](../p0-w05-documentation-baseline/README.md) may re-home it; semantic
   ownership stays with the document.
2. **Fourteen reserved domains.** The plan's list — boot, cpu, vm, vcpu,
   scheduler, memory, stage2, irq, device, virtio, ipc, capability, platform,
   management — is adopted as the reserved top-level classification space,
   each with a one-to-three-sentence subject definition and explicit boundary
   notes for the pairs that will attract disputes (vcpu vs scheduler, memory
   vs stage2, device vs virtio, capability vs management). Adding a domain is
   a policy decision; removing coverage of a plan-named domain would contradict
   ADR-048's coverage intent and is ADR-level.
3. **Canonical name grammar.** An event's canonical name is lowercase
   dot-separated tokens: `domain.subject[.qualifier…]`, each token a
   snake_case singular noun from controlled vocabulary; no prose, no embedded
   counts, timestamps, IDs, or hardware names. The grammar governs canonical
   names only; their encoding on any wire or storage format is owned by the
   telemetry implementation design.
4. **Canonical-event registry as the interface record.** The namespace
   document carries a registry table that starts empty (zero entries — no
   event may be declared in P0). An entry is added only by the approved
   design that introduces the event, with fields: canonical name, domain,
   owning stage/design reference, one-sentence meaning, and status
   (declared/deprecated, replaced-by). The registry is documentary data about
   future events, not a runtime structure.
5. **Compatibility rules.** A declared name's meaning is immutable: changing
   semantics requires deprecating the name and declaring a successor; removal
   requires a recorded deprecation interval and review. The namespace
   governance itself carries a version (`vX.Y` per W05's format rules) whose
   bump rules are stated, so rule changes are detectable. This applies
   ADR-040's versioning principle to the namespace as a compatibility
   surface without touching ADR-040's own subjects.
6. **One-fact-one-name rule.** One semantic fact has exactly one canonical
   event name. A human-log statement or metric that corresponds to a traced
   fact references that canonical name rather than minting a parallel
   identity. W12 owns channel semantics; this rule governs naming identity
   across channels.
7. **New-event review rule.** A P1+ design declaring events must show, per
   event: canonical name conformance, domain attribution with the boundary
   note applied, meaning sentence, and (when replacing) the deprecation link.
   Review rejects prose-like names, wrong-attribution events, and undeclared
   renames.
8. **Drill discipline.** The attribution rehearsal (workflow step 4) uses
   clearly hypothetical future P1 event categories, is recorded in the
   verification record, creates no registry entry, and is never readable as
   a pending event declaration.

## Work breakdown and loading order

1. Read [the namespace contract](01-namespace-contract.md) for the artifact
   groups, the domain registry content, the grammar, the declaration and
   compatibility rules, and the boundary rules with W12.
2. Apply the changes in the order stated in
   [the implementation workflow](02-implementation-and-review.md): verify
   prerequisite surfaces, author the namespace document, wire discovery, run
   the attribution drill, close.
3. Store actual review commands, output, environment, and result in
   `../../verification/p0-w13-trace-event-namespace-baseline-verification.md`,
   and record changed artifacts and any deviation in
   `../p0-w13-trace-event-namespace-baseline-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W13 complete.

## Design-level state and lifecycle

W13 adds no runtime state, registry service, lock, allocation, or code path.
The authoritative state is one tracked namespace document plus its discovery
links. Their documentary lifecycle:

```text
no event naming governance; zero events
  -> trace-event-namespace.md committed (domains, grammar, rules, empty registry)
  -> docs/README.md routing row + stage index row committed
  -> attribution drill evidenced on hypothetical P1 categories
  -> first P1 telemetry design declares the first events into the registry
     in the same change as its implementation design
  -> names evolve only through the declared deprecation/compatibility rules
  -> future registry-validation tooling (if any) is promoted through
     W07's future class; namespace rule changes bump the document version
```

The namespace document owns every naming statement. An event name in a later
design that contradicts the grammar, misattributes a domain, or renames a
declared name outside the compatibility rules is a review failure, not a
local choice.

## Explicitly excluded interfaces

No event, field, payload, ring buffer, transport, encoding, macro, function,
type, trait, module, crate, public API, ABI, or CI workflow is designed or
authorized by W13. The registry is a documentary table; the grammar is a rule
about names, not a data structure. Declaring an event, defining a payload
shape, or choosing an encoding under W13 is a scope conflict against the
future telemetry designs and must be stopped at review.

## Downstream handoff

- **P1+ telemetry designs** (the P1 stage plan index's early-logging and
  diagnostics packages, P4's Stage-2 fault trace subject, P3's SMP
  observability subject, and later stages' telemetry work) receive the
  declaration path: every proposed event arrives with its canonical name,
  domain attribution, meaning sentence, and deprecation link where
  applicable; instrumentation, payloads, and encoding remain their design
  freedom within these rules.
- **W12** is honored in reverse: the channel boundaries and the
  one-fact-one-name principle are consumed from W12's baseline by subject;
  W13 changes neither.
- **W07/W20** receive the registry-consistency predicate a future check could
  verify (every used name declared and non-deprecated); classification and
  wiring stay with them.
- **W05** may re-home the namespace document under its documentation
  taxonomy; **W22** maps this deliverable into the P1 handoff package by its
  own plan scope.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
