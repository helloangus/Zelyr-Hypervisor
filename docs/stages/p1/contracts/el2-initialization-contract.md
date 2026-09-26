# P1 EL2 initialization contract

**Status:** Proposed W09 assembly; reference repetition and NC2–NC5 fault-phase evidence recorded, NC6 execution deferred to P6-V29 under ADR-061.\
**Scope:** Boot-CPU phase order, markers and terminal failure routing; no general service lifecycle.\
**Version:** v0.1.\
**Owner/change context:** P1-W12 assembly of [W09's owning state machine](../implementation/p1-w09-initialization-sequencing/01-init-state-machine.md) and [record](../implementation/p1-w09-initialization-sequencing-record.md), 2026-09-25.\
**Supersedes:** None.

The sole legal sequence is `entry → runtime → capabilities → el2-baseline →
exceptions → console → fatal-path → stage1 → stable`. Each phase records
`enter` then `complete` exactly once; the next phase requires the preceding
completion. `stable` is terminal for normal initialization, emits the
[W10 token](reference-qemu-environment.md) and enters controlled idle. There
is no rollback, skip, retry or secondary-CPU traversal.

The [phase table and predecessor contracts](../implementation/p1-w09-initialization-sequencing/01-init-state-machine.md#1-phase-model)
identify the W01–W08 supplier of each completion. The tracker retains
pre-console events. W06 availability causes exactly one ordered replay from
`entry.enter` through `console.enter`; subsequent events are emitted live.
Thus absence of an early serial marker does not imply that its phase was
skipped. [W09 verification](../verification/p1-w09-initialization-sequencing-verification.md)
records one observed full replay and Stable emission. [W10's 100-cycle
run](../verification/p1-w10-qemu-boot-regression-verification.md) independently
records the required start tokens and Stable in every cycle, but does not
revalidate every intermediate phase transition in every cycle.

Failure leaves the tracker at the failed phase and terminates boot. `entry`
uses W01's pre-transfer rejection. Before the W07 fatal path is armed,
`runtime` through `console` use the W02/P0 bounded panic route, except W05's
own guarded exception boundary. `fatal-path` uses W07's own readiness stop;
`stage1` uses the armed W07 `fail_phase` route with W08's step-qualified
reason. A post-`stable` exception uses W05/W07 and remains attributed to
`stable`. The [owning matrix](../implementation/p1-w09-initialization-sequencing/01-init-state-machine.md#4-failure-routing-matrix)
specifies the individual routes.

Before `exceptions.complete`, an unexpected exception is not guaranteed a
hypervisor-owned vector route. This unowned pre-vector window is a declared
[limitation](known-limitations.md), not an implicit prerequisite. The
[W11 NC5 paired evidence](../verification/p1-w11-negative-fault-validation-verification.md)
locally observes a terminal post-MMU translation fault with
`stage1.complete`, ESR and FAR. That establishes this reference-QEMU path,
not arbitrary post-MMU fault coverage or real-hardware behavior; NC6 remains
unexecuted separately and is required at P6-V29 under
[ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md).
