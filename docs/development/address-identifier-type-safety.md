# Zelyr Address & Identifier Type-Safety Requirements

**Status:** Normative semantic-requirements governance.  
**Scope:** The authoritative address-semantics and identifier-semantics
inventories, interchange and conversion red lines, and design/code review
checklists. It defines **no type, trait, width, module, or API** — the
coding-side mechanism is owned by the [Coding
Guidelines](coding-guidelines.md) (linked, not restated); the two documents
are complements, and neither owns the other's content.  
**Version:** v0.1  
**Owner/change context:** P0-W15 address/identifier type-safety requirement;
operationalizes the ADR §13 direction and ADR-007/013/018/051.  
**Supersedes:** The absence of a written semantic inventory.

## 1. Address-semantics inventory

Entries are **semantics, never types**. Per entry: meaning; acronyms;
typical producers/consumers; qualitative validity concerns; interchange
prohibitions.

### 1.1 Host virtual address (HVA)

An address in the hypervisor's own virtual-address space.
Producer/consumer: the host stage-1 subject. Validity concerns: mapped-ness
and permissions are properties of the address space, not of the integer.

### 1.2 Host physical address (HPA)

An address in the machine's physical address space.
Producers/consumers: discovery data, page allocator, Stage-2 mapping
targets. Validity concerns: range, alignment to mapping granularity,
ownership; the same integer never denotes two live allocations.

### 1.3 Guest physical address (GPA / IPA)

The guest's physical address space, translated by Stage-2. The project's
memory and Stage-2 subjects use IPA; **both acronyms denote this one
semantic**, and this record fixes that equivalence so designs do not invent
a third. Producers/consumers: guest register state, guest memory models,
Stage-2 fault reports. Validity concerns: guest-supplied values are
untrusted until validated (ADR-007); mapping presence and permissions live
in the guest address-space object, not in the integer.

### 1.4 Guest virtual address (GVA)

An address in a vCPU's current virtual-address space, translated by the
guest's own stage-1 (stage configuration owned by later designs).
Producers/consumers: guest architectural state, fault syndrome reports.
Validity concerns: validity depends on the vCPU's translation regime; never
interchangeable with GPA even where a guest identity-maps.

### 1.5 Reserved semantics

**Device-visible/DMA-translated addresses** — a distinct semantic only where
a translation stage exists between a device's addresses and HPA; activated
by the IOMMU/SMMU designs. Until then, device DMA positioning is expressed
in the owning designs' own semantics.

### 1.6 Address interchange prohibitions

HVA↔HPA, GPA/IPA↔HPA, and GVA↔GPA pairs are distinct translations
performed by distinct named mechanisms (host stage-1, Stage-2, guest stage-1
respectively); **no pair is a cast**, and a translated result is a new value
in the target semantic.

## 2. Identity-semantics inventory

Same entry form as §1.

### 2.1 VM identifier

The software identity of a virtual machine object. **Carries no privilege by
itself** (ADR-013/ADR-051: no VM ID implies authority). Validity concerns:
uniqueness among live VMs; lifetime bound to the VM object.

### 2.2 vCPU identifier

The identity of a vCPU within its VM. Validity concerns: VM-scoped
uniqueness; not globally unique and never treated as such.

### 2.3 Physical CPU identifier

The identity of a pCPU in the host's CPU inventory. Validity concerns:
stable across the pCPU's online lifetime; distinct from any architectural
register value that may encode it (the encoding is the owning design's).

### 2.4 Stage-2 VMID (AArch64)

The hardware TLB-tag namespace identifier the architecture uses to tag
Stage-2 translations. **Explicitly a different semantic from the VM
identifier**: it is a scarce hardware resource with allocation and reuse
(invalidation) semantics, never a user-visible authority, and never
interchangeable with the VM identifier even where an allocation happens to
be 1:1. Representation and allocator are the Stage-2 subject's.

### 2.5 Interrupt identifiers

Physical interrupt identity (the INTID space of the interrupt controller)
and virtual interrupt identity (the vCPU-visible interrupt space) are **two
distinct semantics**. Physical-to-virtual binding is a translation owned by
the interrupt designs; the two identifiers are never the same value by
assumption.

### 2.6 Reserved semantics

**Capability handle** (generation-bearing, non-forgeable — the P5
handle-lifecycle subject), **memory-object/region identifiers** (P2 memory
subjects), **device identifiers** (the device framework stage), and
**domain identifiers** (the bootstrap/management stage). Activation is the
owning design's decision under §5's thresholds.

### 2.7 Identity interchange prohibitions

Identifiers from different inventories are never compared, assigned, or
arithmetic-combined; an identifier carries no ordering or arithmetic unless
its design declares one and states its meaning; two identifiers equal as
integers but from different semantics are simply different values.

## 3. Interchange and conversion rules (semantic red lines)

1. **Typed at boundaries.** An interface whose inputs or outputs carry an
   inventoried semantic does not expose it as a bare machine integer. The
   mechanism (newtype or otherwise) is the owning design's, under the Coding
   Guidelines.
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

## 4. Review checklists

**Design-review items:**

- **DR1:** no interface exposes an inventoried semantic as a bare integer
  without a recorded, justified exception.
- **DR2:** every semantic change is a named translation at a declared
  boundary.
- **DR3:** validity concerns (range, alignment, uniqueness, lifetime) are
  stated where the semantic requires them.
- **DR4:** untrusted entry points state their validation before use.
- **DR5:** no §3 rule is waived informally; exceptions are design decisions,
  recorded like any other.

**Code-review items:**

- **CR1:** inventoried semantics use semantic types per the Coding
  Guidelines (restated by pointer).
- **CR2:** no `transmute` or raw reinterpretation between inventoried
  semantics.
- **CR3:** no arithmetic on identifiers; address arithmetic is checked and
  unit-explicit.
- **CR4:** suggested technique — search changed positions for integer-typed
  address/ID parameters; the search is a technique, not a gate (any gate
  belongs to the quality-gates/CI future classes).

## 5. Change thresholds

- **Routine:** clarifying an inventory entry's meaning or validity notes
  without changing interchange outcomes; adding informative examples.
- **Policy decision** (recorded issue and owner decision): adding a required
  semantic to an inventory; activating a reserved semantic ahead of its
  trigger's owning design (should not occur — the trigger's owner decides);
  changing an interchange rule.
- **ADR required:** relaxing the typed-boundary or explicit-translation rule
  to permit bare-integer interfaces as a project-wide practice — it would
  contradict the ADR §13 direction and the Coding Guidelines and follows the
  [ADR process](../adr/README.md).
