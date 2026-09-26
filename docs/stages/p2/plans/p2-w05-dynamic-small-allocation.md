# P2-W05 — Dynamic small-allocation foundation

Chinese readers can use the [Chinese edition](p2-w05-dynamic-small-allocation.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Establish the P2 contract for dynamic small-object allocation with explicit
failure and release behavior backed by the safe page-allocation foundation.

## Scope

P2-F01–F04: runtime-object and variable platform-information needs, explicit
allocation failure, release capability, and repeated allocation/free stress
validation.

## Out of scope

Heap/slab/size-class selection, concrete allocation APIs, later VM/vCPU or
scheduler object designs, and performance benchmarking.

## Work sequence

1. Confirm W04's protected-page and OOM contract can supply safe backing
   capacity for P2's dynamic needs.
2. Establish the required allocation, failure-propagation, and release outcome
   without implying undefined behavior on exhaustion.
3. Describe the stress and recovery properties required for repeated lifecycle
   operations and accounting review.
4. Review the boundary against the task book's prohibition on fixed static
   arrays becoming the permanent runtime model.
5. Specify evidence that separates correctness/invariant testing from a
   performance claim.
6. Hand off the dynamic-allocation contract to inspection, regression, and
   future P3/P4 detailed-design work.

## Acceptance and closure

P2-V07 requires reproducible evidence that exhaustion, release, reuse, and
repeated allocation/free preserve stated invariants. P2-V10 provides the wider
stress regression. No performance or implementation completion is asserted.

## Handoff

W06 may report allocation statistics derived from this foundation. Downstream
stages receive an explicit dynamic-allocation capability and failure boundary.
