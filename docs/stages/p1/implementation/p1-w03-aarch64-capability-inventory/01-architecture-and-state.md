# P1-W03 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W03 detailed design](README.md).

## 1. Logical module map

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Fact extraction | read identification registers, decode fields into fact values | none (pure functions over reads) | EL2 execution context | per-fact values | interpretation policy, classification, any configuration |
| Classification | assign Required/Optional/Future per fact; sanity-decode values | none (pure) | extracted values | classified records | continuation decisions |
| Required-fact check | evaluate the required set; produce the named rejection | none | classified records | `CapabilityRejection` or ok | the route itself (panic route via the phase body) |
| Published report | hold the classified records for the whole stage | `CAPABILITIES` static (once-publication cell) | classified records | query API, render/emit | mutation after publication; channel transport |
| Phase body | the `capabilities` phase's single entry: extract → classify → check → publish → (later) render | orchestration only | W09 phase call | report published; rejection routed | sequencing around the phase (W09), rendering timing (W09 `console` phase wires it) |

The phase body is the mechanism entry W09's `capabilities_step` adapter
calls (W09 §8 seam table); its name and contract are in
[03-code-contracts-classification-and-report.md](03-code-contracts-classification-and-report.md)
§6.

## 2. Fact set

The P1 fact set. Each row: the fact, its source, its classification, and why
it exists. Growing the set is a design change (parent README decision 7).

| FactId | Fact | Source | Classification | Why it is in the set |
|---|---|---|---|---|
| `ExecutionLevel` | exception level at inventory time | `CurrentEL` | **Required** (`EL2` demanded) | the entire stage is an EL2 runtime; re-derives W01's checked field as defense in depth |
| `CpuAffinity` | boot CPU affinity fields (Aff3..Aff0, U, MT) | `MPIDR_EL1` | Optional | recorded identity for evidence and later per-CPU work (P3); never a continuation gate |
| `ArchProfile` | EL0/EL1/EL2/EL3 implementation widths | `ID_AA64PFR0_EL1` | Optional | supports `ExecutionLevel` evidence; records the profile the platform announces |
| `GicVersion` | CPU-side GIC system-register interface version | `ID_AA64PFR0_EL1.GIC` | Future | context for P6; P1 configures no GIC |
| `PaRange` | physical address size | `ID_AA64MMFR0_EL1.PARange` | Optional (consumed by W08 through its own route) | translation-limit knowledge the plan names |
| `AsidBits` | ASID width | `ID_AA64MMFR0_EL1.ASIDBits` | Future | context for later memory work |
| `Granule4k` | 4 KiB granule support | `ID_AA64MMFR0_EL1.TGran4` | **Required** (`supported` demanded) | the granule P1's later mapping work (W08) is designed around; absence makes P1 continuation meaningless |
| `Granule16k` | 16 KiB granule support | `ID_AA64MMFR0_EL1.TGran16` | Optional | reported limit knowledge |
| `Granule64k` | 64 KiB granule support | `ID_AA64MMFR0_EL1.TGran64` | Optional | reported limit knowledge |
| `Stage2Support` | Virtual Host extensions (Stage-2 capability) | `ID_AA64MMFR1_EL1.VH` | Future | the plan's "Stage-2 capability" fact; P4+ consumes it |
| `El2VirtualTimer` | EL2 virtual timer presence | derived: `Stage2Support` present | Optional | guards W04's optional baseline write |
| `CounterFrequency` | system counter frequency | `CNTFRQ_EL0` | **Required** (readable, non-zero demanded) | a zero/unreadable frequency means the timer platform is broken; later timer work is meaningless |
| `El2PhysicalTimer` | EL2 physical timer presence | architectural: present whenever EL2 is | Optional (constant-valued) | documents the timer baseline's write target for W04 |

Notes: the "VA limit" knowledge the plan names is the architectural 48-bit
default recorded as documentation, not a register fact (no register
distinguishes it in P1's set); adding a fact for it would be noise. `MPIDR`
and the ID registers are architecturally present on every AArch64 CPU — that
is exactly why they are not Required checks (parent README decision 2).

## 3. Observation and classification semantics

```text
Observation: Present(u64) | Absent | Unreadable
  Present  — the source was read and the fact's field decoded; u64 carries
             the raw field value (decoding helpers give semantics)
  Absent   — the field exists architecturally but reports "not implemented"
             (e.g. TGran16 == 0b1111) or the derived precondition is false
  Unreadable — the source could not be read; for P1's set this is an
             architectural impossibility for every listed source, so any
             Unreadable observation is itself an invariant violation and is
             reported, never silently mapped to Absent
```

Rules:

- O1 **Required** + anything but the demanded `Present` value ⇒ the
  required-fact check rejects with the named reason (fail-fast).
- O2 **Optional** + `Absent`/`Unreadable` ⇒ recorded and reported; never a
  panic, never a substitute failure (P1-V06: "does not become an unrelated
  panic").
- O3 **Future** facts behave like Optional for P1 continuation; their value
  is preserved for P2/P6 consumers.
- O4 `Unreadable` on any fact is reported at full weight (evidence for an
  environment behaving outside the architecture), and for a Required fact it
  rejects like any other non-conforming observation.

## 4. Report object, lifecycle, and ownership

```text
Publication lifecycle (single-writer, monotone):
  UNPUBLISHED (static default)
    -> the phase body fills a stack-local draft during the capabilities phase
    -> required check passes
    -> publish: the draft is moved into the CAPABILITIES static (once)
    -> read-only for the remainder of the stage and until the P2 handoff
  second publish attempt = invariant violation -> panic route
  read-before-publish = caller-contract violation; the query API documents
  the precondition (post-capabilities-phase), mirroring W02's BootContext
  discipline
```

Ownership: the published static is the sole authority for capability
knowledge (W09's single-owner principle applied to facts). No other
component caches, re-derives, or summarizes facts for its own use; consumers
call the query API. The once-publication cell is one of W03's audited
`unsafe` boundaries (SAFETY: single boot CPU, `DAIF` masked since W02
establishment stage 2, one boot path; readers exist only after publication).

## 5. Concurrency model

Boot CPU only; `DAIF` masked; no allocation; no locks. The extraction reads
are plain system-register reads with no side effects (all sources in the
fact set are pure identification registers — nothing in the set can
context-synchronously change). The only shared-state discipline is the
publication cell of §4 and the caller-supplied emit callback during
rendering (called once, in boot context, from the W09 `console` phase wiring).

## 6. Assumed contracts and failure boundaries

| Seam | Supplied by | Used for | Failure boundary |
|---|---|---|---|
| Live runtime context; panic route | [W02](../p1-w02-minimal-rust-el2-runtime/README.md) (accepted design) | the ability to execute and to fail fast with a named reason | if the route's report cannot carry two static strings, W02/W03 coordination issue; W03 does not build a second output path |
| `capabilities` phase placement; tracker attribution | [W09](../p1-w09-initialization-sequencing/README.md) (accepted design) | when the inventory runs and how failures are attributed | seam mismatch is raised per W09 §1, never adapted locally |
| Channel write; marker/format rules | [W06](../p1-w06-early-console-logging/README.md) (parallel design) | rendering the report lines | if the channel cannot accept the emit-callback seam, the coordination issue is recorded; rendering stays unrendered rather than re-owned |
| Baseline consumption; Stage-1 knowledge (transitive) | [P1-W04](../p1-w04-el2-architectural-state-baseline/README.md) (parallel design; the plan index's named consumer), [P1-W08](../p1-w08-host-stage1-address-space/README.md) only via W04's baseline record unless its design names a direct query dependency | the query API's consumer set | consumers failing to query (branching on names instead) is a W03-DV05 review finding against that consumer |
| NC2 variability | [P1-W11](../p1-w11-negative-fault-validation/README.md) (accepted design) | the scenario's dependency note | if no required fact is variable on the reference platform, NC2 records blocked with the finding (parent README decision 8) |

Produced for consumers: the published `CAPABILITIES` report, the query API,
the rejection vocabulary, and the render/emit seam.
