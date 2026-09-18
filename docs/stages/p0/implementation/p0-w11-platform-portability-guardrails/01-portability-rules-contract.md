# P0-W11 Portability Rules Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W11 detailed design](README.md).

## 1. Logical artifact groups and ownership

W11 is policy and documentation work, so its logical modules are authoritative
artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Portability rules document | `docs/development/platform-portability-rules.md` | ADR-041/042/043/044/045/052, §19 invariants, this design | the sole normative home of layer rules, prohibitions, capability-selection rule, quirk boundary, documentation constraints, checklists, counterexamples, and thresholds; it does not define platform schemas, traits, or code |
| Documentation routing | one row in `docs/README.md` routing table | rules document location | discoverability of the rules from the documentation index; it does not restate policy |
| Platform directory pointer | one pointer line in `docs/platform/README.md` | rules document location | discoverability from the platform-contract entry point; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w11-platform-portability-guardrails-record.md` (created when work starts) | actual decisions taken | changed artifacts, deviations; no command logs (those live in verification) |
| Verification record | `docs/stages/p0/verification/p0-w11-platform-portability-guardrails-verification.md` (created when evidence exists) | actual commands and review output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it.

## 2. Portability rules document contract

Create `docs/development/platform-portability-rules.md` as a normative
document with the status header required by `docs/README.md` (status, scope,
version `v0.1`, owner/change context, supersedes: none) and exactly the
following sections. Required content is stated per section; additional
informative detail is allowed but must not contradict a required statement.

### 2.1 Layer model and dependency rules

A table with one row per logical layer — Core, Arch, SoC, Board/BSP, Quirk,
Driver, and Discovery/Firmware inputs — and columns: subject matter, may
depend on, must never depend on. Required rules, derived from the ADR:

- **Core** (policy-independent hypervisor core and arch-independent VM, CPU,
  memory, IRQ, and device models): may depend on Core contracts and the
  normalized platform description contract. Must never depend on Arch, SoC,
  Board/BSP, Quirk, Driver, firmware, or any platform identity; must never
  parse a raw platform description (ADR-042); must never branch on platform
  identity (ADR-043, ADR-052, §19 invariant).
- **Arch** (ISA backends): may depend on Core contracts and ISA/architecture
  definitions. Must never depend on SoC, Board/BSP, Quirk, or Driver code and
  must never reference platform identity (§19 invariant).
- **SoC** (chip-level controllers and glue): may depend on Arch and Driver
  contracts. Must never depend on Board/BSP or Quirk.
- **Board/BSP** (board wiring and boot differences): may depend on SoC, Arch,
  Driver, and Core contracts. This is the only layer (besides discovery
  inputs) that may reference platform identity.
- **Quirk**: a declaration mechanism inside the BSP/Quirk layer, not a layer
  above Core (see §2.4).
- **Driver** (reusable controllers): may depend on Arch and Core contracts;
  must be selected by discovery/capability data, not by platform identity.
- **Discovery/Firmware inputs** (DTB/ACPI/firmware data): raw inputs consumed
  by the discovery layer only; converted into the normalized platform
  description before anything above sees them (ADR-042).

Direction rule: platform knowledge flows only toward more specific layers;
specifics are injected by binary composition, and Core never selects them.
Layer-to-crate mapping is out of scope for this document.

### 2.2 Platform-identity prohibition

Define "platform identity" as: board names, SoC/model names, vendor names,
machine-type names, and virtual-machine/hypervisor platform names (including
QEMU/virt). Required statements:

- Platform identity may be referenced only inside the Board/BSP/Quirk layer
  and inside discovery inputs.
- Core and Arch must not reference platform identity through any mechanism:
  branches, conditional-compilation switches, string matching, numeric
  constants, or feature selection.
- No rule, document, or review may grant a Core or Arch position an informal
  exemption; exemptions do not exist below the ADR threshold in §2.9.

### 2.3 Capability-driven selection rule

- Any behavior difference between platforms must be expressed as a query
  against a declared capability or property of the normalized platform
  description.
- Every such difference names, at design review, the capability it queries
  and the layer that declares it. "The platforms differ" is not a capability.
- A new capability field is introduced only by a design that owns the
  platform-description subject; review may not add fields ad hoc.
- Core consumes the normalized description; it never reaches past it to a
  discovery source (ADR-042).

### 2.4 Quirk boundary

- A quirk is a declared, localized correction for an observed
  hardware/firmware deviation from architecture or specification expectations.
- Required per-quirk content: the deviation observed, the platform condition
  that activates it, the layer that declares it, and the correction's scope.
- A quirk may never alter architectural semantics, Core invariants, platform
  capabilities, or another layer's contracts; it may never be a channel that
  routes Core or Arch behavior by platform name; a proposed "quirk" that
  would do so is rejected as a §2.2 violation.

### 2.5 Documentation constraints

- Platform/BSP/quirk documentation must state which layer owns each described
  behavior and must carry the per-quirk content of §2.4.
- No tracked document may state a rule that contradicts this document's
  prohibitions; discovered contradictions are corrected in the same change or
  escalated per §2.9.
- Support-tier content (Reference/Tier-1/Experimental) is ADR-045's subject
  and is not defined here; this document only requires that platform
  documentation not claim Core support a layer does not own.

### 2.6 Design-review checklist

Each item must be answerable yes/no at design review:

- R1: Does the design reference platform identity in Core or Arch, by any
  mechanism? (Must be no.)
- R2: Does every platform-conditional behavior name the capability or
  property it queries and the layer that declares it? (Must be yes.)
- R3: Does the design place each new piece of platform knowledge in the
  allowed layer per §2.1? (Must be yes.)
- R4: Does the design consume only the normalized platform description in
  Core positions? (Must be yes.)
- R5: Does every quirk carry the §2.4 content and stay inside its boundary?
  (Must be yes.)
- R6: Are any new address/identifier semantics consistent with the
  type-safety red lines of
  [P0-W15](../p0-w15-address-identifier-type-safety/README.md) when that
  baseline has delivered? (Pointer, not a restatement.)

### 2.7 Code-review checklist

- C1: No platform-identity branch, cfg, string match, constant, or feature
  selection in Core or Arch positions. Suggested observation: search the
  changed positions for known platform names; the search is a technique, not
  a gate.
- C2: Behavior differences resolve through declared capability/property
  queries, not identity checks.
- C3: Quirk declarations carry the §2.4 content.
- C4: New dependencies between implemented layers follow §2.1's directions.

### 2.8 Counterexamples (informative)

Each counterexample must be labeled "hypothetical example — not repository
code" and show violation plus compliant alternative. Required minimum set:

- A Core branch selecting a page-table path by machine name — violation of
  §2.2/ADR-043; compliant: a capability query declared by the platform
  description.
- An Arch constant keyed to a specific SoC's controller base — violation of
  §2.1; compliant: the value supplied from the SoC/Board layer through the
  declared description.
- A "quirk" that redirects Core scheduling behavior when a vendor name is
  detected — violation of §2.4; compliant: the deviation declared in BSP with
  its activation condition, Core behavior unchanged.

### 2.9 Change thresholds

- **Routine** (ordinary PR review): declaring a quirk or BSP artifact inside
  these rules; documentation additions that follow §2.5.
- **Policy decision** (recorded issue and owner decision): adding an
  inter-layer boundary case; reclassifying a layer's allowed dependencies;
  changing a checklist item's meaning.
- **ADR required:** permitting platform-identity references in Core or Arch;
  allowing Arch→SoC/Board dependency; permitting Core to parse raw platform
  descriptions — each contradicts an accepted ADR decision and follows
  `docs/adr/README.md`'s process (W06's mechanism once delivered).

## 3. Explicitly excluded artifacts

This contract authorizes no code, no trait or schema, no crate, module tree,
target, dependency, gate, or CI workflow. The checklists are review surfaces,
not implemented checks; the counterexamples are informative text. Adding any
excluded artifact under W11 is a scope conflict to be raised at review.
