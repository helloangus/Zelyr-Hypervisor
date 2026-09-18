# P0-W15 Semantics and Review Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W15 detailed design](README.md).

## 1. Logical artifact groups and ownership

W15 is governance work, so its logical modules are authoritative artifact
groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Type-safety requirements document | `docs/development/address-identifier-type-safety.md` | ADR §13 suggestion, ADR-007/013/018/051, Coding Guidelines by pointer, this design | the sole normative home of the semantic inventories, interchange/conversion rules, review checklists, and reserved semantics; it defines no type, trait, width, or API |
| Documentation routing | one row in `docs/README.md` routing table | requirements document location | discoverability; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w15-address-identifier-type-safety-record.md` (created when work starts) | actual decisions taken | changed artifacts, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w15-address-identifier-type-safety-verification.md` (created when evidence exists) | actual commands and review output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for
the statement in its row.

## 2. Requirements document form

The document is normative, carries the status header required by
`docs/README.md`, and must contain exactly the sections of §3–§7 below. It
restates no Coding Guidelines rule by copy; the coding-side rules are linked
by pointer, and the document states that complement relationship explicitly
so neither file is read as owning the other's content.

## 3. Address-semantics inventory

Four required semantics. Per entry the document states: meaning, acronyms,
typical producers/consumers, qualitative validity concerns, and interchange
prohibitions. Entries are semantics, never types.

- **Host virtual address (HVA):** an address in the hypervisor's own
  virtual-address space. Producer/consumer: the host stage-1 subject.
  Validity concerns: mapped-ness and permissions are properties of the
  address space, not of the integer.
- **Host physical address (HPA):** an address in the machine's physical
  address space. Producers/consumers: discovery data, page allocator,
  Stage-2 mapping targets. Validity concerns: range, alignment to mapping
  granularity, ownership; the same integer never denotes two live
  allocations.
- **Guest physical address (GPA / IPA):** the guest's physical address
  space, translated by Stage-2. The project's memory and Stage-2 subjects
  use IPA; both acronyms denote this one semantic, and the document records
  that equivalence so designs do not invent a third. Producers/consumers:
  guest register state, guest memory models, Stage-2 fault reports.
  Validity concerns: guest-supplied values are untrusted until validated
  (ADR-007); mapping presence and permissions live in the guest address
  space object, not in the integer.
- **Guest virtual address (GVA):** an address in a vCPU's current
  virtual-address space, translated by the guest's own stage-1 (with stage
  configuration owned by later designs). Producers/consumers: guest
  architectural state, fault syndrome reports. Validity concerns: validity
  depends on the vCPU's translation regime; never interchangeable with GPA
  even where a guest identity-maps.

Reserved semantics (named as subjects with activation triggers, never as
types or fields): **device-visible/DMA-translated addresses** — a distinct
semantic only where a translation stage exists between a device's addresses
and HPA; activated by the IOMMU/SMMU designs. Until then, device DMA
positioning is expressed in the owning designs' own semantics.

Interchange prohibitions required of the document: HVA↔HPA, GPA/IPA↔HPA,
and GVA↔GPA pairs are distinct translations performed by distinct named
mechanisms (host stage-1, Stage-2, guest stage-1 respectively); no pair is a
cast, and a translated result is a new value in the target semantic.

## 4. Identity-semantics inventory

Five required semantics, same entry form:

- **VM identifier:** the software identity of a virtual machine object.
  Carries no privilege by itself (ADR-013/ADR-051: no VM ID implies
  authority). Validity concerns: uniqueness among live VMs; lifetime bound
  to the VM object.
- **vCPU identifier:** the identity of a vCPU within its VM. Validity
  concerns: VM-scoped uniqueness; not globally unique and never treated as
  such.
- **Physical CPU identifier:** the identity of a pCPU in the host's CPU
  inventory. Validity concerns: stable across the pCPU's online lifetime;
  distinct from any architectural register value that may encode it (the
  encoding is the owning design's).
- **Stage-2 VMID (AArch64):** the hardware TLB-tag namespace identifier the
  architecture uses to tag Stage-2 translations. Explicitly a *different*
  semantic from the VM identifier: it is a scarce hardware resource with
  allocation and reuse (invalidation) semantics, never a user-visible
  authority, and never interchangeable with the VM identifier even where an
  allocation happens to be 1:1. Representation and allocator are the Stage-2
  subject's.
- **Interrupt identifiers:** physical interrupt identity (the INTID space of
  the interrupt controller) and virtual interrupt identity (the vCPU-visible
  interrupt space) are two distinct semantics. Physical-to-virtual binding
  is a translation owned by the interrupt designs; the two identifiers are
  never the same value by assumption.

Reserved semantics with triggers: **capability handle** (generation-bearing,
non-forgeable — the P5 handle-lifecycle subject), **memory-object/region
identifiers** (P2 memory subjects), **device identifiers** (the device
framework stage), **domain identifiers** (the bootstrap/management stage).
Activation is the owning design's decision under the document's thresholds.

Interchange prohibitions required of the document: identifiers from
different inventories are never compared, assigned, or arithmetic-combined;
an identifier carries no ordering or arithmetic unless its design declares
one and states its meaning; two identifiers equal as integers but from
different semantics are simply different values.

## 5. Interchange and conversion rules (semantic red lines)

The document must state, as rules future designs are reviewed against:

1. **Typed at boundaries.** An interface whose inputs or outputs carry an
   inventoried semantic does not expose it as a bare machine integer. The
   mechanism (newtype or otherwise) is the owning design's, under the
   Coding Guidelines.
2. **Explicit named translation.** A value changes semantic only through a
   translation operation the design names, at a boundary the design
   declares. Reinterpretation, transmutation, or default conversion between
   semantics is prohibited.
3. **Translated values are new values.** A translation result carries the
   target semantic's own validity concerns; validity never carries over by
   assumption.
4. **Units and arithmetic are explicit.** Address arithmetic is checked and
   unit-explicit (bytes versus pages) per the Coding Guidelines; identifier
   arithmetic is prohibited absent a declared, meaningful ordering.
5. **Untrusted entry validation.** Guest-, firmware-, or discovery-supplied
   values enter the hypervisor only as validated values in their destination
   semantic (ADR-007/§19), never as raw integers assumed valid.
6. **No semantic leakage through storage.** Persisted or transmitted formats
   that carry inventoried values state their semantic and width explicitly;
   raw machine words are not a contract (the Coding Guidelines' ABI rule,
   applied by reference).

## 6. Review checklists

**Design-review items** (required content of the document):

- DR1: no interface exposes an inventoried semantic as a bare integer
  without a recorded, justified exception.
- DR2: every semantic change is a named translation at a declared boundary.
- DR3: validity concerns (range, alignment, uniqueness, lifetime) are stated
  where the semantic requires them.
- DR4: untrusted entry points state their validation before use.
- DR5: no rule of §5 is waived informally; exceptions are design decisions,
  recorded like any other.

**Code-review items:**

- CR1: inventoried semantics use semantic types per the Coding Guidelines
  (restated by pointer).
- CR2: no transmute or raw reinterpretation between inventoried semantics.
- CR3: no arithmetic on identifiers; address arithmetic is checked and
  unit-explicit.
- CR4: suggested technique — search changed positions for integer-typed
  address/ID parameters; the search is a technique, not a gate (any gate
  belongs to [W07](../p0-w07-development-quality-gates/README.md)/[W20](../p0-w20-ci-baseline/README.md)
  future classes).

## 7. Change thresholds

- **Routine:** clarifying an inventory entry's meaning or validity notes
  without changing interchange outcomes; adding informative examples.
- **Policy decision** (recorded issue and owner decision): adding a required
  semantic to an inventory; activating a reserved semantic ahead of its
  trigger's owning design (should not occur — the trigger's owner decides);
  changing an interchange rule.
- **ADR required:** relaxing the typed-boundary or explicit-translation rule
  to permit bare-integer interfaces as a project-wide practice — it would
  contradict the ADR §13 direction and the Coding Guidelines and follows
  `docs/adr/README.md`'s process.
