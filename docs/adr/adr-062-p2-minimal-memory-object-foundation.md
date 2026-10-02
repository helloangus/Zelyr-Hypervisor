# ADR-062 — Clarify the P2 minimal memory-object foundation

Chinese readers can use the [Chinese edition](adr-062-p2-minimal-memory-object-foundation.zh-CN.md).

**Date:** 2026-10-02
**State:** Proposed
**Owner decision:** Option A approved for planning by the project owner on 2026-10-02 ("可以"); formal ADR integration remains pending
**Supersedes:** None; proposes clarification of ADR-000 §4/§6 and its P2 roadmap, preserving their ownership model and stage assignment
**Scope:** P2-ACR-01; minimum architecture-neutral memory ownership and mapping-view foundation, and the P2/P4 producer boundary. No new public ABI, sharing/donation policy, DMA, Guest lifecycle or scheduler policy.
**Context:** The accepted [Chinese ADR-000](adr-000-architecture-baseline-v0.1.md) §15 P2 requires “定义 MemoryObject/MemoryRegion 的最小数据结构。” Its §4/§6 distinguish backing ownership from address-space views. [P2 task book §3](../stages/p2/task-book-v0.1.md#3-architecture-change-record) records a conflict with the P2 exclusion of P4 object-system design. The earlier P2-W11 and P4 drafts used stage-local handles; these must not silently become substitutes for the architecture's ownership model. P4-W10 scope authorization does not resolve this earlier conflict.
**Decision:** Owner-selected option A: P2 owns the minimal common MemoryObject/MemoryRegion foundation; P4 owns Guest-specific integration. Preserve the accepted stage assignment and resolve the task-book boundary explicitly through the bounded follow-up below. This paragraph is not normative until accepted.
**Consequences:** The owner has authorized the [P2-W12 plan](../stages/p2/plans/p2-w12-minimal-memory-objects.md); reconcile W11 and P4 consumers before their runtime admission. W04 is unimplemented and excluded from this submission at the owner's request; its future evidence cannot be inferred from this design. No existing completion claim is expanded.
**Alternatives considered:** B: move the common foundation to P4, explicitly amending the ADR's P2 timing and permitting a bounded temporary Host-mapping exception; this delays common ownership reconciliation and requires a separate sunset/upgrade rule. C: keep names as empty structs or indefinitely use naked frame/view tuples; rejected as a proposal because representation alone does not establish ownership or mapping lifetime.

## 1. Owner-selected bounded decision (option A)

The owner authorized separately planned **P2-W12**, providing
architecture-neutral backing ownership and address-space-view lifetime contracts.
It must implement the minimum semantics, not just introduce type names:

- MemoryObject owns exactly one allocator-derived backing allocation (or a
  bounded owned set) and an unambiguous identity/lifetime. Initial construction
  covers ordinary owned RAM only. External backing and transfer/sharing remain
  reserved; no raw physical address becomes ownership evidence.
- MemoryRegion represents a bounded view of an object's range in one address
  space. It records the required range, rights/memory-type intent and mapping
  lifetime; it does not encode Host or Guest descriptors/registers.
- A live or in-flight mapping retains its backing. Unmapping plus the owning
  translation backend's completed invalidation/quiescence precede release.
  Failed or indeterminate cleanup retains ownership; timeout is not a release
  receipt. Object/region identity cannot silently alias after reuse.
- Host Stage-1 and Guest Stage-2 backends own actual mapping authority, tables,
  ordering and execution constraints. The common objects cannot certify hardware
  completion from an ordinary boolean or forge a backend's release authority.
- W04 still owns physical allocation/free; W12 builds on its ownership handle.
  W11 maps Host views through its architecture adapter; P4-W03 owns Guest RAM
  content/loading and P4-W02/W10 own Guest mappings/context/retirement.

Exact Rust representation, APIs, borrow/token scheme, bounded storage and
accounting are subsequent detailed-design choices, not selected by this ADR.
No sharing, donation, DMA/IOMMU, COW, hotplug, VM provisioning, public capability
ABI, Guest policy or Stage-2 mechanism moves into P2.

## 2. Follow-up and dependency discipline if accepted

1. Reconcile P2 task-book §3 and add the W12 plan with traceable validation;
   do not alter the accepted baseline text to erase the conflict.
2. Make W12 depend on W04 ownership and typed-address inputs, not on W11
   hardware mapping. Establish object/view lifetime against an injected backend
   in Host tests; this is not proof of translated access.
3. Make the corresponding W11 lifetime integration depend on W12; retain W11's
   independent architecture/window/rollback review. W04 metadata mapping is
   image/boot-lifetime infrastructure and is not required to bootstrap itself
   through the new allocator-backed object API.
4. Reconcile P4-W02/W03 single-Guest contracts and W10 multi-VM extension with
   common backing/view authority before their runtime admission. Remove or
   explicitly adapt stage-local ownership substitutes in reviewed designs;
   names alone or duplicated ownership ledgers do not count as integration.
5. Preserve the existing P3 SMP admission boundary. P2 boot-CPU object evidence
   cannot establish concurrent object use or cross-CPU invalidation safety.
6. Record P2-ACR-01's decision separately from delivery: acceptance resolves the
   ownership/scope conflict; implementation and validation close the foundation.

## 3. Acceptance evidence required by the eventual package

Test/review object ownership, bounded range and permission validation, stale
identity refusal, mapping lifetime retention, construction/map failure rollback,
failed invalidation retention, unmapped release, repeated reuse and exact
allocator accounting. Test Host and Guest adapter handoffs separately at their
own stages. No Host fixture may claim actual Stage-1/Stage-2 or SMP behavior.
P2-W10 and P4/P7 input reviews consume linked, condition-specific evidence.

## 4. Decision status and history boundary

The owner selected A on 2026-10-02 ("可以") and authorized the bounded planning
follow-up. This resolves the direction question; do not ask for that choice again.
The ADR remains Proposed pending formal integration, and is not an approved
implementation design. P2-ACR-01 retains that process gate and its delivery gates.
W04-MAP, W11 architecture review and P3/P4 runtime prerequisites are independent;
work outside this conflict is not retroactively invalidated. The user's standing
instruction is to work locally without merging a PR; no PR/merge or historical
approval is fabricated. Record any later owner decision and integration state
honestly under the [ADR process](README.md#4-conflict-supersession-and-review-process).

## Change history

- 2026-10-02 — Proposed by Codex for project-owner review; decision pending;
  carrying PR: none (local remediation under the owner's no-merge instruction).

- 2026-10-02 — Project owner approved option A and planning follow-up ("可以");
  recorded by Codex. Carrying PR: none; work remains local under the standing
  no-merge instruction. Formal integration and implementation evidence pending.

Design follow-up: W12 detailed design and W11/P4 consumer bodies are now reconciled; see the [design review](../testing/documentation-audit/design-approval.md). ADR status remains Proposed; the linked record now records the owner's separate detailed-design approval, not implementation.
