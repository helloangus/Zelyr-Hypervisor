# P3-W03 Physical CPU Lifecycle — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The explicit, observable authority for physical-CPU lifecycle and
online eligibility required by
[P3-W03](../../plans/p3-w03-physical-cpu-lifecycle.md).  
**Owner/change context:** P3-W03 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W03. It defines the runtime
lifecycle state machine for physical CPUs, the single owning registry, the
exactly-once admission rule for the online state, and the read-only
eligibility and online-set surfaces every other P3 package consumes. It
deliberately does **not** classify topology inputs
([P3-W01](../p3-w01-cpu-topology-inputs/README.md) owns that vocabulary),
issue start requests ([P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)
calls this design's transitions), create per-CPU runtime state
([P3-W04](../p3-w04-per-cpu-runtime/README.md)), or coordinate boot phases
([P3-W05](../p3-w05-smp-boot-synchronization/README.md)).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, scope classification, resolved decisions |
| [02-architecture-and-state.md](02-architecture-and-state.md) | the lifecycle state machine, transition ownership table, registry object model, concurrency rules |
| [03-code-contracts-cpu-state-machine.md](03-code-contracts-cpu-state-machine.md) | state vocabulary, encoding, transition-table, and failure-path contracts |
| [04-code-contracts-registry-and-admission.md](04-code-contracts-registry-and-admission.md) | registry, record, admission, eligibility, and online-set contracts |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W03 plan → this
design → Coding Guidelines. Binding constraints:

- The ADR object model names `PhysicalCpu` as a first-class runtime object
  whose state — identity, local resources, lifecycle — is per-CPU and
  distinct from any future `Vcpu` (guardrail: pCPU and vCPU are separate
  objects; static binding is scheduler policy, and vCPU states are P7).
  Nothing here may be read as a vCPU state machine; P4 receives
  availability semantics only.
- ADR-016 keeps scheduling policy out; `Online` means "admitted as usable
  by the hypervisor", never "running a vCPU" or "idle per policy".
- The task book requires the lifecycle to distinguish absent, present,
  starting, initializing, online, and failed CPUs; to prevent duplicate
  online admission and failed-CPU use; and (P3-V03) that no CPU is usable
  before local initialization, no CPU is twice online, and failed CPUs
  stay outside the online set. Each requirement is a designed invariant,
  not prose.
- The plan reserves offline/stopping/suspended states without
  implementing them; this design declares them unreachable and records
  their intended meanings.
- ADR-048 requires observable lifecycle activity; transitions are
  first-class events with CPU attribution (catalog owned by
  [P3-W11](../p3-w11-smp-observability/README.md)).

Classification:

- **Required** for W03 closure: the state vocabulary and encoding, the
  legal-transition table with per-transition owners, the registry and
  records, exactly-once online admission, the eligibility gate, the
  online-set snapshot, lifecycle events, and P3-V03 acceptance evidence.
- **Reserved** with recorded triggers: `Offline` (runtime online→offline),
  `Stopping` (graceful teardown), `Suspended` (low-power wait) — each
  trigger: an approved hotplug/power-management design; `Failed` recovery
  or re-admission — trigger: an approved recovery design; hotplug-driven
  registry mutation — trigger: an approved hotplug design that also
  revisits W01's frozen-topology contract.
- **Out of Scope:** start-request mechanics (W02), per-CPU stacks/areas and
  CPU-local access (W04), rendezvous and SMP-ready (W05), lock primitives
  (W06 — this design uses only atomics), notification targeting (W07),
  vCPU or guest lifecycle (P4+/P7), scheduler policy (P7), runtime CPU
  hotplug (stage out of scope), Orange Pi BSP specifics (P15).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Distinguish absent, present, starting, initializing, online, failed | [state machine](03-code-contracts-cpu-state-machine.md) §2 | P3-V03 (W03-DV01) |
| Reserve offline/stopping/suspended explicitly | [state machine](03-code-contracts-cpu-state-machine.md) §2.3 | W03 closure review (W03-DV01) |
| Permitted outcomes, ownership, diagnostics | [architecture](02-architecture-and-state.md) §3–§4, [transition contracts](03-code-contracts-cpu-state-machine.md) §3 | P3-V03 (W03-DV02, DV05) |
| Online-set admission rules; no duplicate online; no failed-CPU use | [admission contract](04-code-contracts-registry-and-admission.md) §3–§5 | P3-V03 (W03-DV03, DV04) |
| Integrate lifecycle consumers (per-CPU state, notifications, audit, telemetry, regression) | [handoff](06-validation-and-handoff.md) §3; [architecture](02-architecture-and-state.md) §6 | W03 closure review (W03-DV06) |
| Lifecycle evidence including failed-start handling | [workflow](05-implementation-workflow.md) steps 5–6; matrix in [validation](06-validation-and-handoff.md) | P3-V03 (W03-DV05); matrix execution is [P3-W13](../p3-w13-qemu-smp-regression/README.md) |
| State contract recorded; later transitions reserved | [workflow](05-implementation-workflow.md) step 6 | W03 closure review (W03-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P0 documentation scaffold only — no workspace, no sources,
no runtime state of any kind. Sibling P3 designs (W01/W02 upstream; W04–W15
downstream or parallel) are being prepared on this branch; this design
consumes their plan goals and references them by path and P3-Wxx ID without
assuming design content.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Explicit observable authority for lifecycle | No runtime state exists at all | The `CpuRegistry` with per-CPU records as the single authority, and a rule that no other state may answer "which CPUs are usable" | Without a single authority, online-ness would be inferred inconsistently by consumers — the exact opacity P3-V03 forbids | W03 (this design) | W03-DV01/DV02 reviews; DV03 tests |
| Lifecycle distinguishes the six required conditions | No vocabulary exists | `CpuLifecycleState` with the six reachable states plus declared-unreachable reserved states | The task book names the states; the vocabulary must be total before W02 flows in | W03; input classes from W01 | W03-DV01 |
| Prevent duplicate online admission | Nothing prevents it | CAS-based exactly-once admission (`Initializing → Online`) with a fatal diagnostic on the losing path | "No CPU is twice online" must be structural, not a review convention | W03 | W03-DV03 tests |
| Prevent failed-CPU use | Nothing prevents it | The eligibility gate consumed by W04/W05/W07 admission paths, backed by the transition table's Failed-is-terminal rule | Consumers must be structurally unable to admit a Failed or never-initialized CPU | W03 (gate); consumers (calls) | W03-DV04 tests |
| Transitions owned and diagnosable | No transitions exist | The transition-ownership table (architecture §3) plus per-transition events with CPU, from/to, cause | One owner per mutable transition is a stage guardrail; observability is ADR-048 | W03; event catalog W11 | W03-DV05 capture review |
| Lifecycle evidence including failed-start handling | No QEMU harness (W13 is a plan) | Boot diagnostics rendering registry state at SMP-ready, plus the W02 failure-path outcomes landing in the registry | P3-V03's failed-CPU exclusion must be observable in the same evidence as online membership | W03 output; W02 reports; W13 matrix | W03-DV05; matrix deferred to W13 |

W01's vocabulary split (input classes `Present/Possible/Unavailable` vs
runtime states) and W02's terminal outcomes are consumed as upstream
contracts; the failure boundaries are recorded in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §1.2. No ledger
row requires this design to fix crate names or runtime policy owned
elsewhere.

## Resolved design decisions and their authority

1. **Registry as sole authority.** One `CpuRegistry`, built once from
   `TopologyInputs` during global initialization, holds one record per
   discovered CPU (Present/Possible/Unavailable classes alike — excluded
   classes get records so their exclusion is observable, not silent).
   Every lifecycle answer ("is CPU N usable?", "which CPUs are online?")
   is derived from the registry; any cached count elsewhere is telemetry,
   never authority. Rationale: P3-V03 makes the online set an invariant
   over *the* set of CPUs; multiple authorities would make it unprovable.
2. **State encoding: one atomic word per record.** Each record's lifecycle
   state is a single packed atomic value; transitions are
   compare-and-swap operations against the legal-transition table. No
   lock is used (P3-W06 owns lock semantics; W03 needs none). Ordering:
   acquire-loads for reads, acquire-release compare-exchange for
   transitions; a release store publishes the final
   SMP-ready-relevant states (the publication gate is W05's).
3. **One owner per transition.** The legal-transition table names the
   designated actor for each edge: requester (W02 flow) for
   Present→Starting; the secondary itself for Starting→Initializing; the
   boot CPU (bring-up flow) for any→Failed on request/timeout failures;
   the secondary itself for secondary-detected failure;
   W05's rendezvous completion for Initializing→Online (coordinated by
   the boot CPU on behalf of each CPU). The boot CPU's own path uses
   Present→Initializing directly — the one edge restricted to a single
   designated CPU, because the boot CPU issues no start request against
   itself. Rationale: the guardrail "every mutable state transition has
   one owner" plus the task book's diagnosability requirement.
4. **Exactly-once admission via CAS, fatal on contention loss.**
   Initializing→Online uses a single compare-exchange; a losing
   competitor has attempted a duplicate online admission, which is an
   internal sequencing bug, not a recoverable condition — it takes the
   fatal diagnostic path (P0 panic policy) with full attribution.
   Rationale: silently tolerating duplicate admission would hide exactly
   the class of bug P3-V03 exists to expose.
5. **Failed is terminal; registry is boot-static.** No recovery, retry,
   removal, or post-boot record creation exists at P3. Rationale: the
   stage has no hotplug and the plan reserves runtime transitions;
   a static record set (identity fields immutable after build) keeps the
   authority auditable.
6. **Absence is a property of the inventory, not a state.** A CPU not
   declared by P2 has no record; `Absent` is expressed by lookup failure,
   matching W01's omission rule. Rationale: minting records for absent
   CPUs would invent identities W01 refuses to synthesize.
7. **Vocabulary handoff.** `Online` and `Failed` are W03-owned runtime
   states that complete the classification vocabulary the task book
   assigns to P3; the relation to W01's input classes
   (`Online ⊆ attempted Present`, `Failed ⊆ attempted`) is asserted at
   registry build and re-checkable in evidence. Rationale: README of W01
   decision 3; keeps one owner per vocabulary half.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, scope split, and decisions; confirm the W01/W02 upstream
   contracts exist as agreed designs.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   state machine, ownership table, registry model, and concurrency rules.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   state machine with
   [03](03-code-contracts-cpu-state-machine.md) (steps 1–2), registry and
   admission with [04](04-code-contracts-registry-and-admission.md)
   (steps 3–4).
4. Record implementation decisions in
   `../p3-w03-physical-cpu-lifecycle-record.md` and evidence in
   `../../verification/p3-w03-physical-cpu-lifecycle-verification.md` only
   when the work is performed. Validation conditions and the handoff
   checklist are in [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No start-call mechanics, per-CPU storage accessor, rendezvous primitive,
lock type, notification send, TLB transport, scheduler run-queue, or
vCPU-related state is designed or authorized by W03. The registry exposes
no mutation other than the designated transition operations, and no
consumer may mutate a record's identity fields or write a state word
directly — a "set_state" style API is a scope violation to stop at review.
W03 also does not define *which* CPUs W04 may install runtime for beyond
the eligibility gate's answers; per-CPU admission sequencing is W04's.

## Downstream handoff

- **W04** receives the eligibility gate (`Initializing`/`Online` as the
  installable states) and the online-set snapshot as the per-CPU runtime
  rollout's admission input.
- **W05** receives the Initializing→Online transition operation as the
  rendezvous completion's target and the registry scan as its
  terminal-state check for failed participants.
- **W07/W08** receive the online-set snapshot as the valid targeting
  universe (offline/failed/never-started CPUs are structurally untargetable).
- **W09** receives the invariant that exception/fatal diagnostics can
  obtain the executing CPU's lifecycle state from the registry for
  attribution.
- **W10/W11/W12/W13** receive the transition-event stream and
  online-set snapshot as audit, telemetry, stress, and regression
  assertion surfaces.
- **W14/P4** receive physical-CPU availability semantics only
  (`Online` eligibility, class vocabulary); P4 must not infer a vCPU
  state machine from this design — the vCPU lifecycle is a P4/P7 design
  object.
