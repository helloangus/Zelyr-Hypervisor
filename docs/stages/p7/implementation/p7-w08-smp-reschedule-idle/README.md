# P7-W08 SMP Reschedule and Idle Behavior — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Per-pCPU scheduler concurrency, the reconsideration (reschedule)
request protocol, and designed idle behavior required by
[P7-W08](../../plans/p7-w08-smp-reschedule-idle.md).  
**Owner/change context:** P7-W08 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W08. It converts the bounded
work-package plan into concrete mechanics: the per-pCPU scheduling loop that
runs distinct vCPUs concurrently with isolated scheduler state; the
reconsideration request protocol (the P7-wide "cause that pCPU to re-evaluate"
transport) built over the P3 cross-CPU notification contract and the P6-evidenced
Host SGI mechanism; and idle as a designed pCPU state with defined entry,
wake sources, and exit — never a busy loop. It deliberately does **not**
design the P3 notification implementation, CPU hotplug, power management,
load balancing or work stealing, or any new scheduling policy.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| SMP scheduler model, reconsideration protocol, race and isolation rules | [01-smp-scheduler-architecture.md](01-smp-scheduler-architecture.md) |
| Implement reconsideration requests and the handler | [02-code-contracts-reschedule.md](02-code-contracts-reschedule.md) |
| Implement idle entry/wait/exit | [03-code-contracts-idle.md](03-code-contracts-idle.md) |
| Execute the ordered workflow | [04-implementation-workflow.md](04-implementation-workflow.md) |
| Plan or review validation and closure | [05-validation-and-handoff.md](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P7 task book](../../task-book-v0.1.md), the
[P7-W08 plan](../../plans/p7-w08-smp-reschedule-idle.md), and
[P7-W01](../p7-w01-entry-contract-reconciliation/README.md)'s recorded input
boundary. This document is a proposed design; it contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W08 plan → this design
→ Coding Guidelines. Binding constraints:

- ADR-015 (SMP is an early foundation) and ADR-016 (evolution to preemptive
  M:N while preserving pinned/dedicated operation) require correct concurrent
  scheduling across pCPUs; ADR-017 reserves advanced balancing and RT — this
  design implements none.
- ADR-041–045: behavior selection uses platform capabilities and the P3 online
  pCPU set, never board names; Core contains no board conditionals.
- The P3 synchronization contract
  ([P3-W06](../../../p3/plans/p3-w06-concurrency-synchronization.md)) is the
  authority for locking, ordering, and non-sleeping-context rules; this design
  states pairings and delegates barrier/memory-order spellings to it.
- The P3 cross-CPU notification contract
  ([P3-W07](../../../p3/plans/p3-w07-cross-cpu-notification.md)) and the
  P6-evidenced Host SGI mechanism
  ([P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md)) are
  consumed, not redesigned: "P7 may later design reschedule/kick policy over
  an evidenced Host mechanism" is P6-W04's explicit handoff.
- The plan's handoff note is binding: "W11 consumes the proven SMP behavior;
  no CPU-management policy is implemented here."

Classification:

- **Required:** concurrent distinct-vCPU execution with pCPU state isolation;
  the reconsideration request protocol (coalescing, self-target, offline
  targets, handler re-check loop); cross-CPU wakeup and pause notification
  effects; placement compliance for remote requests; designed idle with entry
  protocol, bounded wake sources, and non-busy wait; idle-to-work transitions;
  reschedule/idle accounting hooks; failure diagnostics for remote and idle
  paths; the P7-V17/V18 evidence plan.
- **Reserved:** CPU hotplug (P3 owns pCPU lifecycle; P7 consumes the online
  set), power-management idle states, load balancing/work stealing, NUMA,
  multi-queue policy tuning, and idle-time statistics beyond W09's hooks.
- **Out of Scope:** P3 notification implementation, timer programming, GIC
  register mechanics, runqueue topology (P7-W05), block/wakeup semantics
  (P7-W06), pause/stop semantics (P7-W07), any public/management ABI, and all
  P8+ mechanisms.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W03 and W05–W07 plus P3 notification contracts | [assumed contracts](01-smp-scheduler-architecture.md) §2 | [workflow](04-implementation-workflow.md) step 1 review |
| Concurrent distinct-vCPU execution and pCPU state isolation | [architecture](01-smp-scheduler-architecture.md) §3, isolation rules §6 | P7-V17 |
| Cross-CPU wakeup/pause notification effects; placement compliance | [reschedule contracts](02-code-contracts-reschedule.md) R-1–R-4 | P7-V18 (remote rows) |
| Idle entry/exit requirements and failure diagnostics | [idle contracts](03-code-contracts-idle.md) I-1–I-4 | P7-V18 (idle rows) |
| Review placement and lifecycle preservation under concurrent activity | [architecture](01-smp-scheduler-architecture.md) §5–§6 | [workflow](04-implementation-workflow.md) step 4 review |
| Plan SMP and idle evidence; hand to stress work | [validation and handoff](05-validation-and-handoff.md) | P7-V17–V18 evidence locations; handoff checklist |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p7-implementation-designs`): the repository is a P0 documentation
scaffold — no Cargo workspace, no Rust sources, no implemented crates;
`docs/stages/p7/implementation/` holds only the stage README. P0–P6 are
planned, not implemented; everything W08 consumes is an **assumed contract**
cited by plan path, each with a failure boundary in
[01-smp-scheduler-architecture.md](01-smp-scheduler-architecture.md) §2.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| pCPUs concurrently run distinct vCPUs (P7-V17) | Absent | Per-pCPU scheduling contexts consuming P3-W04's reserved scheduler capacity | Concurrency must be built on the P3 per-CPU foundation, not global shortcuts | [P3-W04](../../../p3/plans/p3-w04-per-cpu-runtime.md); P7-W08 | P7-V17 SMP evidence |
| pCPUs isolate scheduler state (P7-V17) | Absent | Ownership rules: per-pCPU state mutable only by its owner pCPU; shared state only via declared structures | Isolation must be structural for W11 stress to be meaningful | P7-W08 (rules); P3-W06 (sync authority) | P7-V17; P7-V24+ (W11) |
| Remote work/control events cause reconsideration (P7-V18) | Absent | Reconsideration request protocol over P3-W07 notification / P6-W04 Host SGI | W06 wakeups and W07 control requests require a "cause re-evaluation" transport; neither may build its own | [P3-W07](../../../p3/plans/p3-w07-cross-cpu-notification.md); [P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md); P7-W08 (protocol) | P7-V18 remote rows |
| Placement compliance under remote activity | Absent | C-1 eligible-pCPU predicate applied before every remote request | A remote request to an ineligible pCPU would break P7-V06 guarantees | [P7-W03](../../plans/p7-w03-placement-configuration.md); P7-W08 (application) | P7-V18 with placement matrix |
| Idle avoids busy loop and resumes on work (P7-V18) | Absent | Designed idle state: entry protocol, wake sources (folded deadlines, reconsideration, notification), wait-for-event exit | Without a designed idle state, "releases CPU" degrades to spinning | P7-W08 (this design); P6-W05 (deadline fold); arch wait seam (P1/P3 boundary) | P7-V18 idle rows |
| Idle-to-work preserves lifecycle/placement | Absent | Exit path re-enters the same scheduling decision with no policy change | Idle exit must not create a second selection path with different rules | P7-W08; L-1/L-3 (W02) | P7-V17/V18 |
| Evidence handoff to W11 | Verification directory empty (`.gitkeep` only) | Validation matrix and record paths defined here | W11 consumes only declared, locatable evidence | P7-W08 | `../../verification/p7-w08-smp-reschedule-idle-verification.md` |

No row invents an upstream mechanism. If P7-W01's reconciliation (P7-V01)
marks any assumed input missing or contradictory, the affected step is blocked
and recorded; it is not repaired in W08.

## Resolved design decisions and their authority

1. **One scheduler decision entry per pCPU.** Timer preemption (P7-W04),
   reconsideration requests, idle exit, and post-exit continuation all
   converge on a single per-pCPU decision function; there is no second
   selection path. Rationale: P7-V17 isolation and P7-V18 idle-to-work both
   require that "the pCPU re-evaluates" has exactly one meaning. Authority:
   plan scope; stage-local convergence design owned here within W02's
   admission authority (L-3).
2. **Reconsideration is a coalescing intent flag plus one signal.** Each pCPU
   owns a reconsideration intent flag; a requester sets the flag (release)
   and, if the target is remote, sends exactly one transport signal (P3-W07
   primitive; P6-W04-evidenced Host SGI). Multiple concurrent requests
   collapse into one delivery; the handler clears the flag before re-checking
   and re-runs bounded re-checks if work arrived during the handler.
   Rationale: coalescing bounds IRQ-context work and makes request storms
   safe. Authority: plan scope item 2; P3-W06 ordering rules; stage-local
   protocol owned here.
3. **Self-target fast path; offline targets are refused by the transport.** A
   request for the current pCPU skips the transport entirely; requests to
   offline pCPUs surface the P3-W07-defined offline-target outcome and are
   diagnostics-visible. P7 has no hotplug, so scheduler-side offline targets
   indicate a placement bug (C-1 violation) — an invariant, not a retry case.
   Authority: P3-W07 offline outcomes; task book Reserved list (no hotplug).
4. **Idle is a designed state, not a loop.** Idle entry: mark idle intent,
   fold the pCPU's host deadline obligations (P6-W05 surface), re-check for
   late-arriving work, then wait for an event through the architecture wait
   seam; idle exit: clear intent, run the single decision entry (decision 1).
   Rationale: P7-V18 requires non-busy idle; the two-phase entry mirrors the
   P7-W06 block protocol so a request racing idle entry cannot be lost.
   Authority: plan scope item 3; ADR-016; stage-local design freedom owned
   here.
5. **Idle wake sources are exactly three.** (a) a folded host deadline firing
   (P6-W05), (b) a reconsideration signal (decision 2), (c) a cross-CPU
   notification event delivered by the P3-W07 reception path. Anything else
   (e.g. speculative polls) is prohibited. Rationale: a closed wake-source
   list is what makes "idle never misses a wake" checkable. Authority:
   P6-W05/P3-W07 handoffs; stage-local closure of the list owned here.
6. **Remote requests are placement-verified before signal.** Wakeup
   (P7-W06 S-2/S-3) and control paths (P7-W07) name target pCPUs only from
   the C-1 eligible set; W08 refuses (with diagnostics) any request naming an
   ineligible or offline pCPU. Rationale: preserves P7-V06 placement
   guarantees under SMP activity without re-deriving policy here. Authority:
   P7-W03 handoff; plan scope.
7. **No balancing in P7.** An idle pCPU wakes only for work targeted at it or
   already locally eligible; it never pulls work from another pCPU's queue.
   Rationale: plan explicitly excludes advanced balancing; targeted-wake-only
   keeps idle behavior policy-neutral and checkable. Authority: plan out-of-
   scope list; ADR-017 reserves the rest.
8. **Wait mechanism is an architecture seam.** Core calls a wait-for-event
   primitive; the WFE-based implementation and its QEMU-vs-hardware caveats
   live in the arch layer behind the P1/P3 boundaries. QEMU idle behavior
   never becomes the Core contract. Authority: ADR-002/041–045 layering;
   plan out-of-scope list ("P3 notification implementation" and power
   management excluded).

## Work breakdown and loading order

1. Read [01-smp-scheduler-architecture.md](01-smp-scheduler-architecture.md)
   for the pCPU scheduling model, assumed contracts with failure boundaries,
   the reconsideration protocol, and the isolation rules.
2. Implement the reconsideration protocol per
   [02-code-contracts-reschedule.md](02-code-contracts-reschedule.md).
3. Implement idle per [03-code-contracts-idle.md](03-code-contracts-idle.md).
4. Follow [04-implementation-workflow.md](04-implementation-workflow.md).
5. Record planned and actual evidence per
   [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Implementation notes go to `../p7-w08-smp-reschedule-idle-record.md`
   (created when work starts); evidence to
   `../../verification/p7-w08-smp-reschedule-idle-verification.md`. Neither
   this design nor the record may claim W08 complete.

## Explicitly excluded interfaces

No notification primitive, SGI/GIC mechanism, timer programming, hotplug
control, power-management interface, balancing policy, runqueue type, public
API, ABI, or wire format is designed or authorized by W08. The P3-W07
primitive and P6-W04 mechanism are consumed through their assumed surfaces;
the arch wait seam is consumed, never implemented here. Adding any excluded
surface is a scope conflict stopped at review. Cross-package seams relied on
and handed on are inventoried in
[01-smp-scheduler-architecture.md](01-smp-scheduler-architecture.md) §2 and
[05-validation-and-handoff.md](05-validation-and-handoff.md) §3.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md), W11 consumes this package:

- **[P7-W11](../p7-w11-stress-invariants/README.md)** receives: the
  reconsideration protocol and its race classes (request-vs-idle-entry,
  handler-vs-late-work, coalescing storms) for race stress (P7-V25); the pCPU
  isolation rules for invariant/fairness stress (P7-V26); the placement-
  verified remote-request rules for placement stress (P7-V27); and the idle
  wake-source closure list for idle-to-work stress.
- **[P7-W09](../p7-w09-accounting-diagnostics/README.md)** (indirect
  consumer through the W09 prerequisites) receives the hook points for
  reconsideration sent/received, idle entry/exit with reason, and
  offline-target refusals.
- **[P7-W12](../p7-w12-qemu-regression/README.md)** (indirect, via W11)
  receives the SMP/idle behaviors its QEMU matrix (P7-V28) automates.
- **P8** (via P7-W14's handoff, not via this design) may rely only on
  evidenced P7 semantics; no pCPU-management or hotplug policy is created
  here.
