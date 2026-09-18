# P0-W15 Address/Identifier Type Safety — Implementation Record

**Status:** Implemented on branch `p0/w15-type-safety`; verification evidence
in [the verification
record](../verification/p0-w15-address-identifier-type-safety-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W15 detailed implementation
design](p0-w15-address-identifier-type-safety/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/address-identifier-type-safety.md` (new) | Normative type-safety requirements v0.1: four address semantics (HVA, HPA, GPA/IPA with the equivalence recorded, GVA) plus the DMA-reserved semantic with trigger; five identity semantics (VM, vCPU, pCPU, Stage-2 VMID with the explicit VM-ID distinction, physical/virtual interrupt identifiers) plus four reserved semantics with triggers; six red-line rules; DR1–DR5/CR1–CR4 checklists; thresholds; Coding-Guidelines complement statement |
| `docs/README.md` | One routing row for address/identifier semantics in new interfaces |
| `docs/stages/p0/implementation/README.md` | W15 status row updated truthfully |
| This record; the verification record | Decisions and walkthrough evidence |

## Deviations from the design

None. No type, trait, width, module, API, or code was defined or added; the
Coding Guidelines are linked, not restated.

## Handoff notes for downstream packages

- **P1 address-space authors:** HVA/HPA/GPA(IPA)/GVA semantics and red lines
  1–6 apply; translations must be named operations at declared boundaries.
- **P2 allocator authors:** HPA validity concerns (range, alignment,
  ownership) and the reserved memory-object identifiers are their subjects.
- **P5 handle authors:** the capability-handle reserved semantic
  (generation-bearing, non-forgeable) activates with their design.
- **P6 interrupt authors:** physical vs virtual interrupt identifiers are
  distinct semantics; binding is a named translation.
- **W11:** checklist item R6 now resolves to this delivered baseline.
