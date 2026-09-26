# P2-W10 — P3/P4 handoff contract

Chinese readers can use the [Chinese edition](p2-w10-p3-p4-handoff-contract.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Publish the reviewable P2 consumer contract, evidence map, known limitations,
and completion-gate record for P3, P4, and P2 completion review.

## Scope

P2-L01–L05: platform-discovery contract, boot-memory ownership rules, P3/P4
handoffs, known limitations, and a stage-gate evidence checklist.

## Out of scope

Declaring P2 complete, writing verification evidence, resolving P2-ACR-01,
designing P3/P4 internals, or implementing any downstream runtime mechanism.

## Work sequence

1. Collect the planned outcomes, prerequisites, acceptance IDs, and known
   evidence locations from W01–W09.
2. Establish the supported P3 contract: CPU inventory, boot-CPU relation,
   PSCI/capability facts, and allocation availability without AP authorization.
3. Establish the supported P4 contract: host topology, protected-range
   exclusion, allocation/free, and ownership-extension foundation without
   Stage-2 or Guest authorization.
4. Record mandatory limitations for PCI, SMMU/IOMMU, GIC initialization, AP
   bring-up, and Orange Pi runtime support, plus unresolved P2-ACR-01.
5. Review the P2-V01–V13 evidence map, gate criteria, links, package coverage,
   and dependency acyclicity without converting planning into a done claim.
6. Hand off the contract and completion-review checklist to P3/P4 planners and
   the later P2 verification record.

## Acceptance and closure

P2-V12 requires a reviewable P3/P4 consumer contract and evidence locations.
P2-V13 requires complete one-plan-per-package mapping, objective validation
conditions, valid links, acyclic dependencies, and visible P2-ACR-01. Neither
validation is satisfied until real review evidence is recorded.

## Handoff

P3 and P4 planners may consume the stated inputs and limitations. The P2
completion reviewer must require all P2-V01–V13 evidence before any `DONE`
status, and must preserve P2-ACR-01 until authorized resolution.
