# P0-W11 Platform Portability Guardrails — Implementation Record

**Status:** Implemented on branch `p0/w11-portability-guardrails`; verification
evidence in [the verification
record](../verification/p0-w11-platform-portability-guardrails-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W11 detailed implementation
design](p0-w11-platform-portability-guardrails/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/platform-portability-rules.md` (new) | Normative portability rules v0.1: seven-layer model with dependency directions, platform-identity prohibition (all mechanisms), capability-driven selection rule, quirk boundary with required content, documentation constraints, R1–R6 design checklist, C1–C4 code checklist, three labeled counterexamples, three change thresholds |
| `docs/README.md` | "Platform / BSP work" routing row extended with the rules pointer |
| `docs/platform/README.md` | One pointer line (no policy restated) |
| `docs/stages/p0/implementation/README.md` | W11 status row updated truthfully |
| This record; the verification record | Decisions and walkthrough evidence |

## Deviations from the design

None. No code, trait, schema, crate, target, dependency, gate, or CI workflow
was added; the checklists are review surfaces, the counterexamples are
labeled informative.

## Handoff notes for downstream packages

- **P1 platform-design authors:** R1–R5 are the red lines a platform design
  must cite; R6 activates when W15's baseline delivers.
- **Memory/IRQ-design authors:** §3's rule decides where a platform
  difference may be expressed; Core consumes only the normalized
  description.
- **BSP-design authors:** §4's per-quirk content is required at declaration;
  §1 is the only layer allowed to hold platform identity.
- **W15:** R6 references its red lines once delivered; no restatement here.
- **W14:** the failure taxonomy does not interact with this document
  directly; quirk activation conditions remain ordinary validated inputs.
