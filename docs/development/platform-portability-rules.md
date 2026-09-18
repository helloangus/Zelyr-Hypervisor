# Zelyr Platform Portability Rules

**Status:** Normative platform-layering governance.  
**Scope:** The layer model and dependency rules, the platform-identity
prohibition, the capability-driven selection rule, the quirk boundary,
documentation constraints, design- and code-review checklists, counterexamples,
and change thresholds. It defines no platform schema, trait, crate structure,
or code; support tiers are ADR-045's subject.  
**Version:** v0.1  
**Owner/change context:** P0-W11 platform portability guardrails;
operationalizes ADR-041/042/043/044/045/052 and the §19 platform invariants
of the [ADR baseline](../adr/adr-000-architecture-baseline-v0.1.md).  
**Supersedes:** The absence of written portability rules.

## 1. Layer model and dependency rules

| Layer | Subject matter | May depend on | Must never depend on |
|---|---|---|---|
| Core | policy-independent hypervisor core; arch-independent VM, CPU, memory, IRQ, device models | Core contracts; the normalized platform description contract | Arch, SoC, Board/BSP, Quirk, Driver, firmware, or any platform identity; raw platform descriptions (ADR-042); branching on platform identity (ADR-043, ADR-052, §19) |
| Arch | ISA backends | Core contracts; ISA/architecture definitions | SoC, Board/BSP, Quirk, or Driver code; platform identity in any form (§19) |
| SoC | chip-level controllers and glue | Arch and Driver contracts | Board/BSP or Quirk |
| Board/BSP | board wiring and boot differences | SoC, Arch, Driver, Core contracts | — (this is the only layer, besides discovery inputs, that may reference platform identity) |
| Quirk | a declaration mechanism inside the BSP/Quirk layer, not a layer above Core (§4) | its declaring BSP layer's context | — |
| Driver | reusable controllers | Arch and Core contracts | platform identity; drivers are selected by discovery/capability data |
| Discovery/Firmware inputs | DTB/ACPI/firmware data | — | consumed by the discovery layer only; converted into the normalized platform description before anything above sees them (ADR-042) |

**Direction rule:** platform knowledge flows only toward more specific
layers; specifics are injected by binary composition, and Core never selects
them. Layer-to-crate mapping is out of scope for this document.

## 2. Platform-identity prohibition

"Platform identity" means: board names, SoC/model names, vendor names,
machine-type names, and virtual-machine/hypervisor platform names (including
QEMU/virt).

- Platform identity may be referenced only inside the Board/BSP/Quirk layer
  and inside discovery inputs.
- **Core and Arch must not reference platform identity through any
  mechanism**: branches, conditional-compilation switches, string matching,
  numeric constants, or feature selection.
- No rule, document, or review may grant a Core or Arch position an informal
  exemption; exemptions do not exist below the ADR threshold in §8.

## 3. Capability-driven selection rule

- Any behavior difference between platforms must be expressed as a query
  against a declared capability or property of the normalized platform
  description.
- Every such difference names, at design review, the capability it queries
  and the layer that declares it. "The platforms differ" is not a capability.
- A new capability field is introduced only by a design that owns the
  platform-description subject; review may not add fields ad hoc.
- Core consumes the normalized description; it never reaches past it to a
  discovery source (ADR-042).

## 4. Quirk boundary

- A quirk is a declared, localized correction for an observed
  hardware/firmware deviation from architecture or specification
  expectations.
- Required per-quirk content: the deviation observed, the platform condition
  that activates it, the layer that declares it, and the correction's scope.
- A quirk may never alter architectural semantics, Core invariants, platform
  capabilities, or another layer's contracts; it may never be a channel that
  routes Core or Arch behavior by platform name; a proposed "quirk" that
  would do so is rejected as a §2 violation.

## 5. Documentation constraints

- Platform/BSP/quirk documentation must state which layer owns each described
  behavior and must carry the per-quirk content of §4.
- No tracked document may state a rule that contradicts this document's
  prohibitions; discovered contradictions are corrected in the same change or
  escalated per §8.
- Support-tier content (Reference/Tier-1/Experimental) is ADR-045's subject;
  this document only requires that platform documentation not claim Core
  support a layer does not own.

## 6. Design-review checklist

- **R1:** Does the design reference platform identity in Core or Arch, by any
  mechanism? (Must be **no**.)
- **R2:** Does every platform-conditional behavior name the capability or
  property it queries and the layer that declares it? (Must be **yes**.)
- **R3:** Does the design place each new piece of platform knowledge in the
  allowed layer per §1? (Must be **yes**.)
- **R4:** Does the design consume only the normalized platform description in
  Core positions? (Must be **yes**.)
- **R5:** Does every quirk carry the §4 content and stay inside its boundary?
  (Must be **yes**.)
- **R6:** Are any new address/identifier semantics consistent with the
  type-safety red lines of P0-W15 once that baseline has delivered? (Pointer;
  see the [address/identifier record](../stages/p0/implementation/p0-w15-address-identifier-type-safety/README.md)
  for its delivery status.)

## 7. Code-review checklist

- **C1:** No platform-identity branch, cfg, string match, constant, or
  feature selection in Core or Arch positions. (Suggested observation:
  search the changed positions for known platform names; the search is a
  technique, not a gate.)
- **C2:** Behavior differences resolve through declared capability/property
  queries, not identity checks.
- **C3:** Quirk declarations carry the §4 content.
- **C4:** New dependencies between implemented layers follow §1's directions.

## 8. Change thresholds

- **Routine** (ordinary PR review): declaring a quirk or BSP artifact inside
  these rules; documentation additions that follow §5.
- **Policy decision** (recorded issue and owner decision): adding an
  inter-layer boundary case; reclassifying a layer's allowed dependencies;
  changing a checklist item's meaning.
- **ADR required:** permitting platform-identity references in Core or Arch;
  allowing Arch→SoC/Board dependency; permitting Core to parse raw platform
  descriptions — each contradicts an accepted ADR decision and follows the
  [ADR process](../adr/README.md).

## Appendix: counterexamples (informative)

**Hypothetical examples — not repository code.**

1. **Core branch selecting a page-table path by machine name**
   (`if machine == "virt" { … }`): violates §2 / ADR-043. Compliant
   alternative: a capability query declared by the platform description
   (for example a declared stage-2-format capability), with the value
   supplied by the platform's discovery layer.
2. **Arch constant keyed to a specific SoC's controller base** (a hardcoded
   GIC distributor address chosen for one SoC): violates §1 (Arch must not
   depend on SoC specifics). Compliant alternative: the value supplied from
   the SoC/Board layer through the declared description; Arch code stays
   parameterized.
3. **A "quirk" that redirects Core scheduling behavior when a vendor name is
   detected**: violates §4 (a quirk may never route Core behavior by
   platform name). Compliant alternative: the deviation declared in the BSP
   with its activation condition and scope; Core behavior unchanged.
