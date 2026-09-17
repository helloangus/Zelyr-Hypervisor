# P5-W07 — Validation Guest security and isolation suite

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Extend the maintained Rust Validation Guest into a determinate P5 ABI-security
test asset, including two independent security contexts for authority isolation.

## Scope

Cover P5-T15 and T18: discovery, valid HVC, valid references/authority/data,
unknown/invalid version/flags/length/address/overflow/type cases, zero/max/
random/stale/destroyed generation cases, insufficient/no/revoked/cross-VM
authority, repeated destruction, and wrong lifecycle state.

## Out of scope

Linux Guests, production Guest SDK/API, Control Domain protocol, permanent
Guest machine ABI, general multi-VM orchestration, a scheduler, or creation of
runtime evidence before the dependent mechanisms exist.

## Work sequence

1. Inspect W02–W06 scenario contracts and the P4 Validation Guest asset and
   QEMU evidence boundary.
2. Produce an approved detailed design for externally observable P5 scenarios,
   expected markers, two-context setup, and test-asset maintenance ownership.
3. Integrate all required positive, malformed, handle, authority, revocation,
   and lifecycle scenarios without inventing a general Guest protocol.
4. Define the two-context cross-VM/raw-value misuse test and its determinate
   allowed/denied expectations.
5. Review that test markers reveal neither Host pointers nor a promise of a
   future machine or management ABI.
6. Record implementation/evidence status and hand scenario results to W09–W10.

## Acceptance and closure

P5-V11 and V12 require real Validation Guest evidence that all required attack
and valid cases produce expected results and one context cannot use another's
authority. Passing proves only the declared test boundary.

## Handoff

W09 receives stable markers for telemetry and QEMU regression; W10 receives
the maintained scenario inventory and evidence links.
