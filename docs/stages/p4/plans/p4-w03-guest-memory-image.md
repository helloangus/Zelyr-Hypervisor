# P4-W03 — Guest memory and image construction

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Construct a bounded, deterministically initialized Validation Guest memory
input and loadable image path that cannot overwrite forbidden Host memory.

## Scope

This package covers P4-B01–B05: Guest RAM from P2-allocatable memory, temporary
Validation Guest IPA-layout facts, one repeatable image route selected during
detailed design, range/entry/overflow/empty-image validation, and deterministic
initial memory state.

## Out of scope

Selecting a final P8 machine layout, defining image-loader API/module design in
this plan, Linux/firmware boot, Guest DTB, dynamic memory policy, shared
memory, ballooning, or snapshot formats.

## Work sequence

1. Inspect the reconciled P2 allocation, reserved-range, and ownership
   contracts together with W02's address-space requirements.
2. Produce an approved detailed design for bounded Guest-memory construction
   and a repeatable Validation Guest image path.
3. Integrate the planned memory and image inputs with Stage-2 mapping and the
   vCPU entry consumer without exposing forbidden Host ranges.
4. Define normal and negative image/range scenarios, including overflow,
   invalid entry, prohibited overlap, and deterministic reinitialization.
5. Review the boundary against Guest-untrusted input handling and the rule that
   temporary P4 layout facts are not a machine ABI.
6. Record implementation/evidence status and hand a factual boot-input
   contract to W04, W05, and W09.

## Acceptance and closure

P4-V03 requires evidence that only allocatable Host pages back Guest RAM and
that invalid image inputs cannot overwrite protected memory. Passing also
requires deterministic initialization across declared repeat scenarios. It does
not prove Linux boot or a versioned virtual platform.

## Handoff

W04 receives validated Guest entry, stack, and image inputs; W05 receives the
maintained test image route. W09 later records only facts actually implemented
and verified.
