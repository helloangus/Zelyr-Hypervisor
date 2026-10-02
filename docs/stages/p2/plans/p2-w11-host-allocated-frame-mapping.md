# P2-W11 — Host allocated-frame mapping foundation

Chinese readers can use the [Chinese edition](p2-w11-host-allocated-frame-mapping.zh-CN.md).

**Status:** Authorized work package; detailed design owner-approved on 2026-10-02; implementation and validation pending
**Version:** v0.1
**Owner/change context:** P2-ACR-02 / AUD-003 owner direction, 2026-10-02
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Provide explicit, lifetime-bounded Host access to ordinary allocated frames so
P3 local areas/stacks and P4 Stage-2 tables/image-loading buffers can be safely
initialized and used. Physical allocation alone does not provide mapped access.

## Scope

Required P2-M01–M05: allocated-frame coverage and checked HPA/HVA relations;
writable, non-executable mapping attributes and alias discipline; ownership
and borrow lifetime; unmap/invalidation-before-free and transactional rollback;
boot-CPU-only validation and consumer admission under `P2-HOST-MAP`.

The detailed design must reconcile P1's existing bootstrap mappings and
page-table access with W03 protection and W04 owned-frame allocation, including
independently available table storage. W04-MAP remains responsible for allocator
metadata initialization. W05-MAP retains its heap backing/lifetime adapter;
reuse of W11 is optional and requires its own reviewed integration.

Reserved: integration with P3-owned SMP serialization and cross-CPU invalidation
after their contracts and evidence exist. P2 supplies only boot-CPU operation
and the documented preconditions for that later integration.

## Out of scope

A permanent identity-map or all-RAM access promise, arbitrary MMIO mapping,
Guest Stage-2/VM mechanisms, P2-ACR-01 memory objects, AP startup, SMP locks or
cross-CPU shootdown, new Guest/public ABI, and specific mapping algorithms,
virtual addresses, Rust APIs or module layout before detailed design.

## Work sequence

1. Reconcile P1 bootstrap coverage, W03 protected regions, W04 allocation and
   metadata access, and P3/P4 mapped-access requirements. Record missing inputs.
2. Produce a separately reviewed detailed design for frame coverage, attributes,
   aliases, table storage/bootstrap access, ownership, lifetime, rejection,
   invalidation and complete failure rollback.
3. Implement and integrate the approved boot-CPU mapping foundation with W04
   owned allocations while retaining P1 execution and protected ranges.
4. Establish reproducible host-model checks for coverage, bounds, lifetime,
   partial failure, rollback, release and reuse; hand scenarios to W08.
5. Establish target/reference-QEMU allocated-frame access and unmap/reuse
   evidence for representative stack, page-table and image-buffer uses;
   integrate the declared configuration coverage with W09.
6. Record the actual producer contract, evidence and limits for W10 and P3/P4.
   Keep `P2-HOST-MAP` blocked where required design or evidence is absent.

## Acceptance and closure

P2-V14 requires explicit coverage, attributes and lifetime guarantees; rejection
of protected/out-of-domain and invalid-lifetime access; no writable/executable
or conflicting memory-type alias; complete rollback on partial-map failure;
and unmap/invalidation before frame reuse. Host-model and reference-QEMU results
are recorded separately; stack, table and image-buffer fixtures exercise Host
access only and do not imply actual P3/P4 execution. Preserve P1 bootstrap
continuity. P2-V10/V11 provide wider regression coverage; P2-V12/V13 review the
consumer contract and planning map. No hardware or SMP safety follows from
boot-CPU/QEMU evidence, and no row is satisfied by this plan.

## Handoff

W08 receives mapping lifecycle/error scenarios, W09 receives reference-platform
integration requirements, and W10 receives the authoritative producer and
per-condition evidence. P3-W02/W04 and P4-W01/W02/W03 consume only the declared
Host views and lifetime/cleanup obligations. P3 owns any later SMP extension.
Implementation records belong under `../implementation/`; actual results under
`../verification/`. Coding requires an approved detailed design.

Owner-selected ADR-062 follow-up (2026-10-02): consume the
[W12 common ownership/view foundation](p2-w12-minimal-memory-objects.md) and
P2-V15 evidence at the corresponding lifetime/acceptance boundary. W08 owns
Host negative/lifetime scenarios, W09 owns actual integrated Host access, W10
owns condition-specific handoff, and W11 owns the translation adapter. W12
Host modeling precedes W11 and does not require its hardware; W04-MAP remains
independent. Formal ADR integration and approved detailed designs are still
required before affected coding; no object or runtime delivery is inferred.
