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
   evidence locations from W01–W09 and W11.
2. Establish the supported P3 contract: CPU inventory, boot-CPU relation,
   PSCI/capability facts, and allocation availability without AP authorization.
3. Establish the supported P4 contract: host topology, protected-range
   exclusion, allocation/free, and ownership-extension foundation without
   Stage-2 or Guest authorization.
4. Record mandatory limitations for PCI, SMMU/IOMMU, GIC initialization, AP
   bring-up, and Orange Pi runtime support, plus unresolved P2-ACR-01.
5. Review the P2-V01–V14 evidence map, gate criteria, links, package coverage,
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
completion reviewer must require all P2-V01–V14 evidence before any `DONE`
status, and must preserve P2-ACR-01 until authorized resolution.

## P2-ACR-02 planning amendment

Collect W11 alongside W01–W09. Publish P2-HOST-MAP coverage, attributes, lifetime, cleanup and applicable evidence to P3/P4. Review P2-V01–V14 and both P2-ACR-01/02; W11 planning does not satisfy the producer gate. Missing approved design, implementation or evidence blocks affected consumer admission and stage completion.

Owner-selected ADR-062 follow-up (2026-10-02): consume the
[W12 common ownership/view foundation](p2-w12-minimal-memory-objects.md) and
P2-V15 evidence at the corresponding lifetime/acceptance boundary. W08 owns
Host negative/lifetime scenarios, W09 owns actual integrated Host access, W10
owns condition-specific handoff, and W11 owns the translation adapter. W12
Host modeling precedes W11 and does not require its hardware; W04-MAP remains
independent. Formal ADR integration and approved detailed designs are still
required before affected coding; no object or runtime delivery is inferred.
