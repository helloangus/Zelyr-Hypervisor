# P0-W15 Address/Identifier Type Safety — Verification Evidence

**Status:** Complete evidence recorded; W15 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch `p0/w15-type-safety`
(baseline: merge of PR #23).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W15-DV01 → P0-V09 | Address-inventory review | **passed** | The four required address semantics are present in entry form (meaning, acronyms, producers/consumers, validity concerns, prohibitions); the GPA/IPA equivalence is recorded as one semantic with two acronyms; the DMA-visible semantic is reserved with its IOMMU/SMMU activation trigger. |
| W15-DV02 → P0-V09 | Identity-inventory review | **passed** | The five required identity semantics are present; the VM-identifier vs Stage-2 VMID distinction is explicit (scarce hardware resource, never user-visible authority, never interchangeable); the physical/virtual interrupt split is explicit; four reserved semantics carry triggers and owners. |
| W15-DV03 → P0-V09 | Rule and checklist review | **passed** | Six red-line rules present (typed at boundaries, named translation, new-value semantics, explicit units/arithmetic, untrusted-entry validation, no storage leakage); DR1–DR5 and CR1–CR4 are answerable at their respective reviews; the Coding Guidelines are linked as the mechanism owner, not copied. |
| W15-DV04 → P0-V09 | Non-prescription walkthrough | **passed** | Four hypothetical design questions resolved by semantics alone with no type, width, trait, or module implied (evidence below). |
| W15-DV05 → P0-V09 | Discovery and link review | **passed** | Routing row reaches the requirements document in one link; Coding-Guidelines and ADR links resolve; stage-index row truthful. |
| W15-DV06 → W15 closure | Consumability review | **passed** | P1 address-space author (four address semantics + red lines), P2 allocator author (HPA validity + reserved memory identifiers), P5 handle author (reserved capability-handle trigger), P6 interrupt author (two interrupt semantics, binding as translation) — each can state the red lines for their subject without inventing types or policy. |

## Non-prescription walkthrough (W15-DV04 evidence)

| Hypothetical design question | Resolution by semantics alone | What was *not* implied |
|---|---|---|
| "A fault report carries the faulting address from Stage-2." | The semantic is GPA/IPA (Stage-2 fault reports are its consumers, §1.3); untrusted-entry rule 5 applies before use. | No fault-report struct, field name, or width was fixed. |
| "A hypercall passes a buffer location." | Guest-supplied → untrusted until validated in its destination semantic (rule 5); the buffer's semantic (GPA vs transmitted HPA) is the owning design's declared boundary. | No hypercall ABI or argument type was fixed. |
| "The scheduler needs to identify a vCPU's home pCPU." | vCPU identifier (VM-scoped) and pCPU identifier (host inventory) are distinct semantics; the binding between them is a named relation owned by the scheduler design (§2 prohibitions). | No scheduler trait or ID representation was fixed. |
| "An interrupt is forwarded from physical to virtual." | Physical INTID and virtual interrupt identity are distinct semantics; forwarding is a named translation whose result is a new value (rules 2–3). | No mapping-table type or INTID width was fixed. |

## Not run / not proved

- **No type exists**; P0 defines no newtype or API — the requirement
  constrains without reverse-specifying, per the plan's non-prescription
  boundary.
- **Future compliance:** the checklists are review surfaces; any mechanical
  gate is W07/W20 future class.
