# Memory and Stage-2 design approval

Chinese readers can use the [Chinese edition](design-approval.zh-CN.md).

**Status:** Informative factual approval record; the listed designs are owner-approved.
**Scope:** The completed W12/W11/P4 design revision and P7 pre-entry-abort companion.
**Version:** v0.2
**Owner/change context:** Project owner explicitly confirmed approval on 2026-10-02.
**Supersedes:** The live memory/Stage-2 remediation-review note and approval-pending
statements for the exact scope below. Other stage approvals are unchanged.

## Owner approval

**Approver:** Project owner (the user). **Date:** 2026-10-02.
**Decision:** “我确认批准” following the explanation that detailed-design approval
makes this version the basis for subsequent implementation.
**Recording:** Initially recorded in the local working tree under the then-current
no-merge instruction. The owner subsequently authorized submission and PR merge;
that integration does not change the design approval scope or supply runtime evidence.
P2-W04 implementation and completion artifacts are excluded at the owner’s request;
the package remains unimplemented and will be restarted separately.

| Approved design | Version | Scope |
|---|---|---|
| [P2-W12](../../stages/p2/implementation/p2-w12-minimal-memory-objects/README.md) | v0.1 | Common backing/view lifetime, bounded storage and identity, rights/aliases and completion authority |
| [P2-W11](../../stages/p2/implementation/p2-w11-host-allocated-frame-mapping/README.md) | v0.2 | W12 Host-region adapter, VA/table capacity, scoped bytes/pins, rollback/revoke |
| [P4-W02](../../stages/p4/implementation/p4-w02-stage2-address-space/README.md) | v0.2 | README and supporting 01–05: Stage-2 ownership, state, APIs, workflow and validation |
| [P4-W03](../../stages/p4/implementation/p4-w03-guest-memory-image/README.md) | v0.2 | README and supporting 01–05: W12 Guest adapter, fixed IPA/independent HPA, load-before-map, code visibility and release |
| [P4-W10](../../stages/p4/implementation/p4-w10-multivm-stage2-handoff/README.md) | v0.1 | Multiple spaces, per-CPU installation/execution/residency, retirement, failure and V17–V22 |
| [P7-W02 abort companion](../../stages/p7/implementation/p7-w02-scheduler-admission-lifecycle/06-pre-entry-abort.md) | v0.1 | Identity-bound pre-entry abort and its W02 lifecycle / W04 switch / W05 queue binding edits |

The approval includes corresponding Chinese editions and the P4-W04 execution-
lease/explicit-detachment binding needed by these contracts. It does not approve
all unrelated P4/P7 packages, the P3 W06/W08 package designs, P0-W01 retrospectively,
or other unresolved ADRs. ADR-062's independently recorded formal lifecycle is
unchanged; the user-selected option A remains the architecture direction.

## Remaining execution gates

Approval does not certify architecture instruction sequences, final image/stack/
table fit, absent upstream capabilities or runtime behavior. W04-MAP, W12/W11
producer implementation, P3-admitted SMP adapters, P4 Stage-2/Guest implementation,
P7 cleanup/abort integration and owning Host/target/QEMU evidence remain required.
The designs themselves retain these checks at the relevant implementation steps.
AUD-003/AUD-004 are therefore not implementation-complete or runtime-closed.

## Current record maintenance

Current findings live in the [audit summary](README.md#current-findings).
The owner requested deletion of intermediate audit records and explicitly declined
archives. Only this approval record and the current bilingual summary remain here;
there is no compressed archive, working ledger or repeated phase/handoff report.
This cleanup records status and reorganizes documents; it adds no production code,
unsafe, ABI or dependency and performs no Guest/target execution.
