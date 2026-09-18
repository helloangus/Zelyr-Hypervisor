# P3-W14 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W14 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"P4 receives an explicit, bounded Host SMP foundation contract" requires
four concrete artifacts:

1. **The contract document** at
   `docs/stages/p3/implementation/p3-w14-p4-smp-handoff-contract.md`
   with the R1–R9 reliance structure —
   [02-p4-consumer-map.md](02-p4-consumer-map.md) §2.
2. **The evidence inventory** — for every reliance statement, the
   producing package's design path, implementation-record path, and
   verification-record path with their real statuses — workflow step 1
   in [03-workflow-and-review.md](03-workflow-and-review.md).
3. **The consumer map and independence list** — same file §3–§4.
4. **The sufficiency review record** — the P4-consumer review with
   per-item results — workflow step 5.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| P3 deliverables W01–W13 | The P3 plans and their designs/records | Each plan's stated outcome, delivered with design, implementation record, and verification evidence | A package that delivers less than its plan outcome makes the corresponding reliance statement unevidenced: the contract lists it as a limitation/blocking issue; W14 never upgrades a status to make the handoff look complete |
| P2→P3 handoff contract | [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md) | The documented P2 consumer contract (CPU inventory, boot-CPU relation, PSCI/capability facts, allocation availability) and its known limitations (PCI, SMMU/IOMMU, GIC init, AP bring-up, Orange Pi; P2-ACR-01) | The P3 contract inherits P2's recorded limitations by reference; if P2-W10's delivered contract diverges, that divergence is recorded as an inherited open item with its P2 owner — W14 does not re-adjudicate P2 |
| P4 consumer requirements | [P4 task book](../../../p4/task-book-v0.1.md) §2, [P4 plan index](../../../p4/plans/README.md) | The named P3 consumption per package (allocator/synchronization, pCPU identity, notification, TLB transport, CPU-local state) | If P4's planning changes its consumption list, the map is re-issued per the contract lifecycle (entry README decision 7); W14 does not chase unpublished P4 intent |
| Stage boundaries | [P3 task book](../../task-book-v0.1.md) §2, ADR object model | Reserved splits (P4: Stage-2/VMID/TLBI semantics; P6: GIC; P7: scheduling; P8: guest SMP; P15: hardware) | A contract statement that crosses a reserved boundary is a review failure (W14-DV04) |
| Conflict labels | Plan Agent guide; skill rules | `ADR Required` / `Architecture Change Request` as the only legal markers for unresolved architecture-level conflicts | A conflict silently resolved inside the contract is a design violation; it must be labeled and listed |

### 1.3 Why no hidden essential deliverable remains

- Plan step 1 ("inspect W01–W13 plans and implementation/verification
  records that actually exist") is realized by the evidence inventory
  (workflow step 1) — an explicit artifact, not a mental check, because
  its per-item statuses drive the contract's honesty rules.
- Plan step 2 ("consolidate only evidenced guarantees, known limitations,
  and blocking issues") is realized by the R1–R9 structure with per-
  statement evidence status and the unresolved-items register.
- Plan step 3 ("integrate the handoff with the P4 planning reading order
  and stage-document boundaries") is realized by the consumer map keyed
  to P4 package IDs and the contract's placement in the documented
  reading order.
- Plan step 6 ("record the handoff package and items P4 must design
  independently") is realized by the independence list (§4 of the
  consumer-map file) with the same evidence discipline.

## 2. Scope classification

### 2.1 Required

- The contract document with: versioned status header; R1–R9 reliance
  sections (evidence-cited); limits and non-guarantees; unresolved-items
  register; independence list; consumer map; pointer table to all P3
  plans/designs/records.
- The evidence inventory with per-item status (evidenced / planned /
  blocked / absent).
- The P4-consumer sufficiency review and its record.
- Host/guest separation and ADR-compliance review.
- Traceability: every contract statement → P3-Wxx source.

### 2.2 Reserved (must not block a future design; not implemented now)

- Post-closure contract re-issues; trigger: an approved P3 change.
- P5+ consumption statements; trigger: P4-W09's closeout handoff.
- Machine-readable contract format; trigger: a tooling decision.

### 2.3 Out of Scope

- Any P4 mechanism or detailed design; Stage-2/TLBI/VMID/VM/vCPU/
  scheduling semantics.
- New P3 mechanisms or retrofitted guarantees.
- Editing upstream plans/designs/records.
- The P3 stage closure review (W15's package) and any completion claim.
