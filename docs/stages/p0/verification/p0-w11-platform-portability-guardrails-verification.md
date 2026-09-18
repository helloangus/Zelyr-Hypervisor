# P0-W11 Platform Portability Guardrails — Verification Evidence

**Status:** Complete evidence recorded; W11 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch
`p0/w11-portability-guardrails` (baseline: merge of PR #21).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W11-DV01 → P0-V12 | Rules review | **passed** | §1's layer table covers all seven rows (Core, Arch, SoC, Board/BSP, Quirk, Driver, Discovery/Firmware inputs) with subject matter, may-depend-on, and must-never-depend-on; the direction rule is stated; §Appendix carries the three required counterexamples, each labeled "Hypothetical examples — not repository code" with violation + compliant alternative. |
| W11-DV02 → P0-V12 | Prohibition review | **passed** | §2 defines platform identity and prohibits it in Core and Arch through *all* mechanisms (branches, cfg, string match, constants, feature selection), permitting it only in Board/BSP/Quirk and discovery inputs; §4 closes the quirk loophole (no routing Core/Arch behavior by name); consistent with ADR-043, ADR-052, and the §19 invariants — no rule permits the prohibited forms. |
| W11-DV03 → P0-V12 | Selection-rule review | **passed** | §3 requires every platform difference to name its queried capability and declaring layer, reserves field introduction to the owning design, and bars Core from reaching past the normalized description; the walkthrough (below) resolves its difference through the rule; no schema/trait is invented. |
| W11-DV04 → P0-V12 | Platform-difference walkthrough | **passed** | Hypothetical UART difference traced (evidence below) to one classification with no code written and no invented schema. |
| W11-DV05 → P0-V09/P0-V12 | Discovery and link review | **passed** | Reachable in one link from all three surfaces: `docs/README.md` routing row (Platform/BSP work), `docs/platform/README.md` pointer, and the development directory's role; links resolve; stage-index row truthful. |
| W11-DV06 → P0-V12/P0-V15 | Consumability review | **passed** | P1 platform-design author (R1–R5 citable), memory-design author (§3 decides where a difference lives), IRQ-design author (same rule; Core consumes the normalized description), BSP-design author (§4 content + §1 identity home) — each can state the red lines without inventing policy. |

## Platform-difference walkthrough (W11-DV03/DV04 evidence)

Hypothetical: two platforms differ in their debug-UART register stride
(byte vs halfword).

1. **Classify the difference** (§3): it is a property of the platform's
   discovery data, not of the driver's identity — the difference must be
   expressed as a declared property of the normalized platform description.
2. **Name the capability and declaring layer**: a UART register-stride
   property declared by the platform-description subject (owning design);
   the driver queries it; "the platforms differ" is not accepted.
3. **Placement** (§1): the property value originates in discovery inputs →
   Board/BSP layer; the driver (Driver layer) consumes the declared
   description; Core is not involved; no identity string appears anywhere.
4. **Checklists applied**: R1 no (no identity in Core/Arch), R2 yes
   (capability named), R3 yes (knowledge in allowed layer), R4 yes (Core
   unaffected), C2 yes (query not identity), C4 yes (directions respected).
5. **No invented schema**: the property's introduction is recorded for the
   platform-description owning design; this walkthrough wrote no field, no
   trait, and no code.

## Not run / not proved

- **No code exists to obey the rules**; P0-V12 is satisfied by reviewable
  rules per its definition ("no rule permits board/QEMU dependencies in Core
  or Board dependencies in Arch").
- **Mechanical enforcement:** C1's name search is a technique, not a gate;
  any future gate is W07/W20 scope.
- **W15 red lines:** R6 is a pointer; it activates on W15 delivery.
