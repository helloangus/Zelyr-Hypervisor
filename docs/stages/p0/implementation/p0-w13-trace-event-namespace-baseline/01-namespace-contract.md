# P0-W13 Trace Event Namespace Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W13 detailed design](README.md).

## 1. Logical artifact groups and ownership

W13 is governance work, so its logical modules are authoritative artifact
groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Trace-event namespace document | `docs/development/trace-event-namespace.md` | plan's domain list, ADR-048/§12, W12 channel boundaries by subject, this design | the sole normative home of the domain registry, grammar, declaration/compatibility rules, and the canonical-event registry; it defines no event, field, payload, or encoding |
| Documentation routing | one row in `docs/README.md` routing table | namespace document location | discoverability; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w13-trace-event-namespace-baseline-record.md` (created when work starts) | actual decisions taken | changed artifacts, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w13-trace-event-namespace-baseline-verification.md` (created when evidence exists) | actual commands and review output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for
the statement in its row.

## 2. Namespace document form

The namespace document is normative, carries the status header required by
`docs/README.md` (status, scope, version `v0.1`, owner/change context,
supersedes: none), and must contain exactly the sections of §3–§8 below. The
document's own version is its namespace-governance version: a semantic change
to any rule or domain definition bumps the minor version; a compatibility-
affecting change to the declaration or deprecation rules is a policy decision
under §8 before any bump.

## 3. Domain registry

Reserved top-level classification space; one subject definition and boundary
notes per domain. No event may be declared in P0, so every entry below is a
domain definition, not an event.

| Domain | Subject (required definition content) | Boundary notes to state |
|---|---|---|
| boot | Bring-up and startup progression from firmware handoff to a running hypervisor | ends where the subsystem's own domain begins (a boot-time GIC init is still `irq` if the fact is about IRQ state) |
| cpu | Physical CPU lifecycle and per-CPU architectural state | vCPU facts are `vcpu`; scheduling decisions are `scheduler`; pCPU bring-up mechanics are `cpu` |
| vm | Virtual-machine object lifecycle and VM-scoped state | per-vCPU state inside a VM is `vcpu`; capability operations on a VM are `capability` |
| vcpu | vCPU lifecycle, register/state transitions, entry/exit facts | scheduler policy reasons for a switch are `scheduler`; the switch itself is `vcpu` |
| scheduler | Runqueue, scheduling decisions, preemption, accounting, load | wakes caused by notifications reference the waking domain's event as cause, not as a second scheduler event |
| memory | Physical page allocation, ownership, host address space | Stage-2 translation faults and mappings are `stage2` |
| stage2 | Stage-2 address space: mappings, permissions, faults, TLB maintenance for guest translation | host stage-1 facts are `memory` |
| irq | Physical and virtual interrupt lifecycle, routing, latency | vGIC maintenance facts are `irq`; timer-driven wakes are `irq` or `vcpu` per cause |
| device | Virtual device models and passthrough device state | virtio-specific facts are `virtio` |
| virtio | Virtio transports, queues, descriptors, backends | a device-model state machine fact stays `device` unless it is about the virtio ABI |
| ipc | Endpoints, notifications, shared regions, cross-domain messages | capability grants that enable IPC are `capability` |
| capability | Capability/handle creation, delegation, attenuation, revocation, checks | management-plane operations are `management` even when authorized by capability |
| platform | Platform discovery, capabilities, BSP/quirk-relevant facts | device-specific facts belong to `device`/`irq`, not `platform` |
| management | Management ABI operations, control-plane requests, audit-relevant facts | the wire ABI itself is a future ABI contract's subject, not an event |

Rules the registry section must state: the domain list is closed for P0;
adding a domain is a policy decision; the subject definitions are the
attribution authority when two domains could claim a fact.

## 4. Canonical name grammar

- Form: `domain.subject[.qualifier…]` — lowercase, dot-separated, each token
  snake_case, singular nouns, controlled vocabulary.
- Prohibited in names: prose, verbs in imperative mood, embedded counters,
  timestamps, instance IDs, hardware or platform names (see
  [W11](../p0-w11-platform-portability-guardrails/README.md)'s identity
  containment — a name that only exists for one board is a layering smell),
  and abbreviations not already established by the registry.
- The first token must be a registered domain in its canonical spelling.
- The grammar governs canonical names only; their textual encoding on any
  wire, log line, or storage format is owned by the telemetry implementation
  design.

## 5. Declaration rule and canonical-event registry

- The document carries a canonical-event registry table, initially empty.
  Registry entries are added only by the approved design that introduces the
  event, in the same change as that design.
- Entry fields: canonical name; domain; owning stage/design reference;
  one-sentence meaning; status (`declared` or `deprecated`); replaced-by
  (when deprecated).
- A name not present in the registry is not a project event; instrumenting
  code with an undeclared name is a review failure.
- Declaring an event is a design-level act: P0 closes with zero entries, and
  no P0 artifact may contain an event name (drill examples are marked
  hypothetical and live only in verification evidence).

## 6. Compatibility and deprecation rules

- A declared name's meaning is immutable. A semantic change deprecates the
  old name (status `deprecated`, meaning sentence frozen) and declares a
  successor; both entries coexist for a recorded deprecation interval before
  any removal, and removal requires its own reviewed decision.
- Renaming without deprecation, or silently changing a meaning sentence, is
  a review failure.
- The namespace document's own version follows §2's bump rules so consumers
  can detect rule changes; this applies the ADR's versioning principle to
  the namespace as a compatibility surface and does not touch ADR-040's
  subjects (schema/machine/management-ABI versions).
- Dropping coverage of a plan-named domain, or making events non-trimmable,
  would contradict accepted ADR positions and follows the ADR path.

## 7. Boundary coordination with channels and metrics

- One semantic fact has exactly one canonical event name. Where a
  human-log statement or a metric corresponds to a traced fact, the
  producing design references the event's canonical name as the fact's
  identity instead of minting a parallel name.
- Channel semantics, levels, and visibility classes are owned by
  [W12](../p0-w12-logging-diagnostic-baseline/README.md); the namespace owns
  naming identity and compatibility only. A dispute of the form "is this a
  log or a trace?" resolves under W12; "what is this fact called?" resolves
  here.
- Payloads and encodings belong to the telemetry implementation design; the
  registry's meaning sentence constrains semantics, not representation.

## 8. Change thresholds

- **Routine:** clarifying a subject definition without changing attribution
  outcomes; adding informative grammar examples.
- **Policy decision** (recorded issue and owner decision): adding a domain;
  changing the grammar; changing declaration or deprecation mechanics;
  changing a boundary note in a way that moves attribution outcomes.
- **ADR required:** removing a plan-named domain's coverage; abandoning
  name-based compatibility (ad-hoc strings as interfaces).
