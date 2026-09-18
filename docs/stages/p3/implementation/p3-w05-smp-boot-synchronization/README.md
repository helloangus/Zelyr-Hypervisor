# P3-W05 SMP Boot Synchronization — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The defined, verifiable boot-time ordering from one-time global
initialization through secondary readiness to SMP-ready, required by
[P3-W05](../../plans/p3-w05-smp-boot-synchronization.md).  
**Owner/change context:** P3-W05 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W05. It defines the boot
phase model (one authority for "what may run when"), the one-shot
rendezvous protocol between the boot CPU and the secondaries, the exact
memory-ordering rules that make global initialization visible before any
secondary consumes it, the degraded-outcome semantics when some CPUs fail,
and the testable SMP-ready condition consumed by
[P3-W12](../p3-w12-smp-stress-failure-tests/README.md) and
[P3-W13](../p3-w13-qemu-smp-regression/README.md). It deliberately does
**not** provide a general barrier library or reusable synchronization
primitive (explicit plan out-of-scope; shared-state lock semantics are
[P3-W06](../p3-w06-concurrency-synchronization/README.md)), does not
coordinate runtime hotplug, and does not implement lifecycle transitions
(it calls [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)'s
`admit_online`).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, scope classification, resolved decisions |
| [02-architecture-and-state.md](02-architecture-and-state.md) | boot phase model, rendezvous protocol, ordering rules, failure semantics |
| [03-code-contracts-boot-rendezvous.md](03-code-contracts-boot-rendezvous.md) | phase gate, ready gate, coordinator, and SMP-ready query contracts |
| [04-implementation-workflow.md](04-implementation-workflow.md) | ordered implementation steps |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W05 plan → this
design → Coding Guidelines. Binding constraints:

- The task book requires coordinating "one-time global initialization,
  per-CPU initialization, secondary readiness, and a testable SMP-ready
  condition"; P3-V05 requires once-only global initialization, per-CPU
  local initialization, and SMP-ready only after the declared readiness
  condition. Each is a designed invariant here.
- The ADR invariant that any cross-CPU operation must define its
  synchronization and invalidation semantics applies to this design's own
  publication steps; the ordering rules in
  [02 §5](02-architecture-and-state.md) are therefore normative, not
  advisory.
- The plan's out-of-scope list ("a general barrier library", "a concrete
  primitive implementation") is interpreted as: the rendezvous is a
  single-purpose, one-shot boot protocol owned by this design — not a
  reusable barrier type, and not a synchronization framework. Its export
  surface is deliberately minimal.
- Boot synchronization *protocols* (PSCI start/release shape) are
  stage-local design freedom with recorded rationale; W05 inherits
  [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)'s PSCI selection
  and adds the release ordering around it.

Classification:

- **Required** for W05 closure: the boot phase gate, the per-CPU ready
  gate, the coordinator wait with failed-participant accounting, the
  SMP-ready declaration and query, the degraded-outcome record, and
  P3-V05 acceptance evidence.
- **Reserved** with recorded triggers: timing measurement via the
  architectural counter for rendezvous-latency telemetry (informative
  diagnostic only until [P3-W11](../p3-w11-smp-observability/README.md)
  and the P6 timer baseline own timing); a strict-fail boot profile
  (halt on any secondary failure) — trigger: a boot-policy decision with
  the P1-W09-style boot-integration owner; WFE/SEV parking — trigger:
  P3-W07's event primitive; reuse of the rendezvous machinery after boot
  (e.g., stop-the-world) — trigger: a later-stage design (P16 owns
  stop-the-world semantics).
- **Out of Scope:** lock primitives and shared-data access rules beyond
  the boot publication ordering (W06); CPU lifecycle transitions (W03);
  per-CPU runtime contents (W04); notification/IPI (W07); runtime
  hotplug coordination (stage out of scope); scheduler blocking (P7);
  guest-visible synchronization (P8+).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Global-init, local-init, ready, SMP-ready ordering defined | [phase model](02-architecture-and-state.md) §2, [contracts](03-code-contracts-boot-rendezvous.md) §2 | P3-V05 (W05-DV01) |
| Prevent secondary use of incomplete global resources | [ordering rules](02-architecture-and-state.md) §5, [gate contract](03-code-contracts-boot-rendezvous.md) §2 | P3-V05 (W05-DV02) |
| Require local initialization for every CPU | [ready gate](03-code-contracts-boot-rendezvous.md) §3 | P3-V05 (W05-DV03) |
| Once-only authority (global init, SMP-ready) | [coordinator contract](03-code-contracts-boot-rendezvous.md) §4 | P3-V05 (W05-DV01) |
| Failure observability (degraded results) | [degraded semantics](02-architecture-and-state.md) §6, [outcome contract](03-code-contracts-boot-rendezvous.md) §5 | P3-V05 (W05-DV04) |
| Integrate readiness contract with synchronization and test consumers | [handoff](05-validation-and-handoff.md) §3; [workflow](04-implementation-workflow.md) step 5 | W05 closure review (W05-DV05) |
| Repeated rendezvous evidence incl. timing/failure observations | [workflow](04-implementation-workflow.md) step 6; matrix in [validation](05-validation-and-handoff.md) | P3-V05 (W05-DV06); matrix execution is [P3-W13](../p3-w13-qemu-smp-regression/README.md) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P0 documentation scaffold only — no workspace, no sources,
no boot sequence. Sibling P3 designs are being prepared in parallel on
this branch; W05 consumes W02/W03/W04 as upstream contracts and serves
W06/W10/W12–W15 downstream, all referenced by path and P3-Wxx ID without
assuming design content.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Boot-time ordering is defined and verifiable | No boot sequence exists to order | The `BootPhase` authority: one atomic phase word with designated actors and once-only transitions | "Once-only global initialization" needs a structural guard, not a convention | W05 (this design) | W05-DV01 tests |
| Secondaries never consume incomplete global resources | Nothing prevents it | The publication gate: global init completes and release-publishes before any CPU_ON is issued; secondaries acquire-check before touching shared state | A started secondary that reads half-built registries is the classic SMP boot corruption; P3-V05 forbids it structurally | W05 gate; W02 dispatch obeys | W05-DV02 |
| Every CPU performs local initialization before readiness | Nothing requires it | The per-CPU ready gate: release-signal per CPU after W04 install; coordinator counts only signaled CPUs | "SMP-ready" must mean something per CPU, or it is a boot flag with no content | W05; W04 signal point | W05-DV03 |
| SMP-ready is declared once, after a declared condition | No condition exists | The coordinator's terminal-state check over W03's registry plus the ready count, and the once-only SMP-ready publication | The condition must handle *failed* participants or boot hangs on any failure | W05; W03 registry scan | W05-DV01/DV04 |
| Failure observability for the rendezvous | No degraded model exists | `SmpReadyState::{Pending, Ready, Degraded}` with the failed set recorded and diagnosed | P3-V05's failure-observability wording; also the P3-V02 degradation flow needs a consumer | W05; continuation policy surfaced to boot-integration owner | W05-DV04 |
| Repeated rendezvous evidence, timing observations | No QEMU harness (W13 is a plan) | Boot diagnostics emitting phase transitions and the SMP-ready result; an optional counter-based latency observation recorded as informative | Repeatability is observable only if phase transitions are emitted every boot | W05 output; W13 execution | W05-DV06 |

The upstream contracts and their failure boundaries are tabulated in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §1.2. No ledger
row requires fixing crate names, timing sources, or boot policy owned
elsewhere.

## Resolved design decisions and their authority

1. **Single phase word as the boot-ordering authority.** `BootPhase ∈
   {Bootstrap, GlobalInitPublished, SmpReady}` — two once-only
   transitions, each performed by its designated actor (boot CPU for
   both). Rationale: the ordering requirements are few and strictly
   ordered; a single monotonic word with CAS transitions is auditable,
   needs no lock, and gives every rule in this design a single anchor.
2. **Structural ordering, then belt-and-braces acquire checks.** Secondaries
   *cannot* run before global initialization because CPU_ON is dispatched
   only after `GlobalInitPublished` (W02 asserts the gate before its first
   call). The secondary additionally acquire-reads the phase before its
   first shared-state touch, so a firmware anomaly that starts a CPU
   early fails closed with a diagnostic instead of corrupting state.
   Rationale: defense in depth at near-zero cost; the primary guarantee
   is structural.
3. **Publication by release-store, consumption by acquire-load; explicit
   fence at SMP-ready.** All cross-CPU visibility in the boot protocol is
   expressed with acquire/release atomics; the SMP-ready declaration
   performs a full-barrier (`dmb ish`) store-release so the multi-variable
   boot state (registry, areas, tables, outcomes) is coherent to all
   observers. Rationale: the ADR cross-CPU invariant demands defined
   semantics; acquire/release plus one explicit publish fence is the
   minimal sufficient discipline, and no SeqCst is needed anywhere in the
   protocol (stated so reviewers can hold the line).
4. **Rendezvous counts readiness; the registry counts fate.** The ready
   gate counts CPUs that completed local initialization; failed CPUs are
   accounted through W03's registry states (Starting/Failed), not through
   the ready counter. Rationale: one fact, one owner — readiness belongs
   to W05, lifecycle fate to W03; mixing them would create two authorities
   for "what happened to CPU N".
5. **Coordinator wait is bounded by the attempted-set terminal condition,
   not by time.** The wait completes when every attempted CPU is either
   ready or terminal-failed; W02's bounded poll already guarantees
   terminals exist. No timer is introduced at boot. Rationale: no timer
   exists in the P1–P3 baseline (P6 owns timers); the terminal condition
   reuses W02's guarantee instead of inventing a second timeout.
6. **Degraded is a first-class SMP-ready outcome.** `Degraded{online,
   failed}` is declared when attempted CPUs include failures; the
   reference-boot default is continue-with-diagnostics so failures stay
   repeatable and diagnosable (matching P3-V02), while the halt-on-degradation
   policy is explicitly surfaced to the boot-integration owner (P1-W09
   style) as a Reserved profile. Rationale: the plan removes continuation
   policy from W02 and gives W05 "failure observability"; this design
   makes degradation observable and bounded without choosing a policy it
   does not own.
7. **The rendezvous is one-shot and non-reusable by construction.** Its
   types take the boot's expected set at construction, expose no reset,
   and document that a second rendezvous requires a new object and a new
   design decision. Rationale: the plan excludes a general barrier
   library; making reuse awkward is the design honoring that boundary.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, scope split, and decisions.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   phase model, protocol, ordering rules, and degraded semantics.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md)
   with [03-code-contracts-boot-rendezvous.md](03-code-contracts-boot-rendezvous.md)
   (steps 1–4).
4. Record implementation decisions in
   `../p3-w05-smp-boot-synchronization-record.md` and evidence in
   `../../verification/p3-w05-smp-boot-synchronization-verification.md`
   only when the work is performed. Validation conditions and the handoff
   checklist are in [05-validation-and-handoff.md](05-validation-and-handoff.md).

## Explicitly excluded interfaces

No lock type, no general barrier or event primitive, no runtime
stop-the-world, no hotplug coordination, no timer dependency, no lifecycle
transition implementation, and no notification/IPI mechanism is designed
or authorized by W05. The exported surface is exactly: the phase gate
(publish/check), the per-CPU ready signal, the coordinator wait,
`declare_smp_ready`, and the `smp_ready_state` query. Anything resembling
`Barrier::wait_and_reset`, a condition variable, or a broadcast channel is
a scope violation to stop at review. W05 also does not own the *content*
of global initialization — it owns when it is complete and published.

## Downstream handoff

- **W06** receives the settled memory-ordering baseline for boot-published
  shared state (what is guaranteed visible when, from the phase model) as
  the starting point for its lock/atomic policy, and the boundary that
  W05 added no lock.
- **W10** receives the once-only and per-CPU authority rules as audit
  criteria for P0–P2 shared-state classification (the "boot-only" class in
  the P3-V10 taxonomy maps to this design's phase gate).
- **W12** receives `smp_ready_state` and the phase-transition events as
  the stress harness's boot assertion surface, and the degraded-outcome
  record as a failure-injection checkpoint.
- **W13** receives the rendezvous as a required assertion in every
  regression boot (phase sequence, SMP-ready result, degraded accounting)
  and the recorded timing-observation definition (informative until W11/P6).
- **W15** receives the readiness conditions and downstream limits for the
  stage documentation package.
- **P4** consumes boot ordering only through
  [P3-W14](../p3-w14-p4-smp-handoff/README.md); P4 designs any additional
  synchronization it needs against W06's policy, not against this
  rendezvous.
