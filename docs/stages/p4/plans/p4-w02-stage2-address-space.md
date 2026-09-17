# P4-W02 — Stage-2 address-space correctness

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Provide a single Guest's independently managed Stage-2 capability boundary with
correct mapping, revocation, protection, query, activation, and current-path
translation consistency.

## Scope

The package covers P4-A01–A07: independent address-space lifecycle, Guest IPA
to Host physical mapping, unmap, read/write/execute permission, diagnostic
query, installation for Guest execution, and invalidation sufficient for the
current execution path. It consumes P2 memory ownership and P3's future
cross-CPU transport without choosing their internals.

## Out of scope

Page-table or VMID representation, mapper APIs/algorithms, final concurrent
shootdown policy, large-page optimization, dirty tracking, COW, migration, or
a formal multi-VM machine ABI.

## Work sequence

1. Establish the P2 memory-ownership and P3 invalidation assumptions that
   constrain Stage-2 work.
2. Produce an approved detailed design for the address-space lifecycle and its
   map, unmap, protect, query, and activation responsibilities.
3. Integrate the design with Guest-memory consumers while preserving Host/Guest
   address and ownership distinctions.
4. Define planned normal and negative acceptance scenarios for mapped,
   unmapped, and permission-changed Guest access.
5. Review the result against ADR Guest-untrusted, layering, EL1, telemetry, and
   no-permanent-single-pCPU constraints.
6. Record implementation status and evidence references, then hand the proved
   capability boundary to W04 and W06.

## Acceptance and closure

P4-V02, P4-V07, and P4-V08 require real lifecycle, isolation, and permission
evidence. Passing means the selected independent address space can apply and
diagnose the required mutations without stale current-path access. It does not
prove cross-pCPU shootdown, performance, or a future machine ABI.

## Handoff

W04 receives the Stage-2 activation boundary; W06 receives the query and
negative-access basis. Future stages receive only factual P4 capability and
limitation records, not a frozen mapper API.
