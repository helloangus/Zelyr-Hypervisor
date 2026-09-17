# P2-W01 — Boot platform-description intake

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Establish a validated, diagnosable boundary for the boot-supplied DTB before
any P2 consumer interprets platform facts.

## Scope

P2-A01–A04: DTB availability, physical-range and image-overlap checks,
structural bounds validation, required foundational DT encodings, and safe
handling of unknown nodes.

## Out of scope

Platform-specific discovery, normalized platform data, memory allocation, or a
choice of parser crate, API, data layout, or module structure.

## Work sequence

1. Confirm P1's documented boot-information, image-range, diagnostic, and
   unsafe-boundary inputs are available to the package.
2. Define the required validated-input outcome and its explicit diagnostic
   outcomes for absent, inaccessible, overlapping, malformed, and unsupported
   input.
3. Establish the required P2 DT structural and encoding coverage, including
   checked ranges and safely ignorable versus diagnostically relevant unknown
   nodes.
4. Align the boundary with the task-book arithmetic, portability, and
   untrusted-input constraints without selecting an implementation design.
5. Review the planned host-side evidence that distinguishes a rejected input
   from uncontrolled behavior.
6. Record the validated-intake contract for discovery, offline checking, and
   later negative regression consumers.

## Acceptance and closure

P2-V01 and P2-V02 require review/test evidence that DTB availability, range and
image-overlap outcomes are explicit, and malformed structural/encoding inputs
are bounded and diagnosable. Evidence belongs in `../verification/`; planning
alone does not satisfy either validation.

## Handoff

W02 and W07 may rely on a validated DTB intake boundary and stated diagnostics.
Parser and API design remain unimplemented.
