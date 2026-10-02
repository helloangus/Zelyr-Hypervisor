# P4-W03 Guest memory and image construction — detailed implementation design

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
**Scope:** P4-B01–B05 / P4-V03.
**Parent:** [W03 plan](../../plans/p4-w03-guest-memory-image.md).
**Authority:** [P4 task book](../../task-book-v0.1.md), ADR baseline and
[Coding Guidelines](../../../../development/coding-guidelines.md).
**Supersedes:** This package's pre-W12 raw-range ownership sketches; no accepted
ADR or historical stage completion evidence is changed.

**Approval:** Project owner, 2026-10-02, explicit confirmation “我确认批准”; see the [approval record](../../../../testing/documentation-audit/design-approval.md#owner-approval). Approved as the implementation design; its prerequisite, architecture/fit and runtime gates remain in force.

## Purpose and use

GuestRam is a W12 object adapter. Initialize through a W11 Host RW view,
complete instruction visibility, retire that view, then publish Guest RX/RW
regions. W12 prevents release while any region/pin remains. Temporary Guest IPA
is independent of allocator-chosen HPA; layout remains a P4 test convention.
Partial writes are failed content, never a successful all-or-nothing load.

Read [scope/foundations](01-scope-and-foundations.md),
[architecture/state](02-architecture-and-state.md),
[code contracts](03-code-contracts-guest-memory.md),
[workflow](04-implementation-workflow.md), then
[validation/handoff](05-validation-and-handoff.md).
These files contain the revised contracts themselves; no old interface is left
as coding authority behind a disclaimer.

## Current baseline and foundation ledger

P2 W04 is unimplemented and excluded from this submission; its allocation,
boot access and mapping are prerequisites for a future work-package run. Production W12, W11 and P4 mechanisms are absent.
Current Rust workspace placement is hypervisor plus host-test-baseline; logical
module names here do not imply new crates. Physical-address ownership alone
cannot supply Host references or translation completion.

| Required foundation | Producer | Consumer contract / validation |
|---|---|---|
| Backing and retained views | [W12 design](../../../p2/implementation/p2-w12-minimal-memory-objects/README.md) | Exact object/region identity, one actual W04 handle, no raw-HPA import; lifecycle negatives |
| Host byte/table access | [W11 design](../../../p2/implementation/p2-w11-host-allocated-frame-mapping/README.md) | Scoped bytes, external-use pin, full revoke before reuse; actual target mapping |
| Stage-2 and Guest ordering | W02/W03/W04 | Load-before-Guest-map; explicit entry lease and detach; target negative accesses |
| Multi-space/multi-CPU handoff | [W10 design](../p4-w10-multivm-stage2-handoff/README.md) | Separate extension with V17–V22; base evidence cannot imply availability |

## Authority, constraints, and scope classification

Owner selected ADR-062 option A: P2 owns the common memory foundation, P4 owns
Guest adapters. Formal ADR integration and producer implementation evidence are
pending. This design neither invents a competing backing owner nor degrades to
plain alloc/free when W12 is absent. Designed API names remain internal and
unstable. Base P4 evidence remains one Validation Guest/current path.

Required behavior is in the parent plan and contracts above. Huge pages, COW,
DMA/IOMMU, VMID reuse and a P8 machine ABI remain excluded. The bounded W10
extension separately owns multiple spaces and cross-CPU retirement. Host Stage-1
implementation belongs to P2, Guest control policy to its owning later stages.

## Work breakdown and loading order

Implement pure checked models first, then admitted upstream adapters, then Arch
publication/completion, then target integration. The workflow names acceptance
and failure at each step. No raw-pointer workaround bypasses a missing producer.

## Downstream handoff

W04 receives loaded input and exact installation/entry/exit contracts; W05 receives
versioned base layout/BootInfo; W06 receives committed query and fault context;
W07/W08 receive phase/outcome traces and validation cases; W09 records only
implemented capabilities and limits. P7 requires W10's separate evidence bundle.
Owner design approval is recorded above; architecture execution checks,
production code and runtime results remain separate.
