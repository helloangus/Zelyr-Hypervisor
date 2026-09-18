# P6-W03 Code Contracts — Identification and Classification

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W03 detailed design](README.md).  
**Convention:** checklist §3 contract template. Hardware access only via
the W02 register-access surface
([W02 03](../p6-w02-physical-gic-bring-up/03-code-contracts-register-access.md)).
Pseudocode is design logic.

## 1. `IrqClassification`

```text
Name and stability: enum IrqClassification {
  Assigned(PhysicalIntId),
  NoPending,               // spurious band: acknowledge found nothing
  ReservedBand(u16),       // implementation-reserved band value, diagnosed
  OutOfSupported(u16),     // assigned-band value outside the enabled
                           // supported range
} (P6-internal)
Purpose and caller: the sole classification vocabulary; produced by
  classify_acknowledge(), consumed by the dispatch loop of
  [04](04-code-contracts-dispatch-completion.md)
Preconditions / postconditions: constructed only from an actual IAR read;
  NoPending never carries an ID
State and ownership: value type
Errors: none (classification total over the architectural value space)
Validation: unit tests over the full encoded value space per the pinned
  revision bands (W03-DV03)
```

Band boundaries are transcribed from the pinned GIC specification revision
(Specification Investigation checkpoint, W01 step-1 record): this design
fixes that there is a no-pending band and reserved bands, that assigned
values are contiguous from 0, and that every band has exactly one
behavior; it does not hard-code numeric band edges in prose.

## 2. `PhysicalIntId` family

```text
Name and stability: enum PhysicalIntId { Sgi(SgiId), Ppi(PpiId),
  Spi(SpiId) } with SgiId(u8 valid 0..=15), PpiId(u8 valid 16..=31),
  SpiId(u16 valid 32..=supported_max) (P6-internal newtypes extending the
  P0 baseline; supported_max from the W02 readiness facts, fixed at boot)
Purpose and caller: the only representation of a physical interrupt ID in
  P6; consumers (W04 send/routing, W05 PPI registration, W08 maintenance)
  construct these, never raw numbers
Construction: fn try_new_sgi(v: u8) -> Option<SgiId>, etc.; total
  validators; Display for diagnostics; ordering by numeric ID for stats
Preconditions / postconditions: an instance always denotes an ID inside
  the architectural class range AND the platform-supported range at
  construction time
Never: From<u32> lossy conversions, naked u32/u64 parameters in any P6
  interrupt API (Coding Guidelines typed-ID rule)
Validation: range unit tests; boundary values (15/16/31/32/supported_max)
```

## 3. `classify_acknowledge()`

```text
Name and stability: fn classify_acknowledge(supported: &SupportedRange)
  -> IrqClassification (P6-internal; the only legal way to obtain an ID
  for dispatch; reads ICC_IAR1_EL1 itself so the read and the
  classification cannot diverge)
Purpose and caller: single acknowledge point of the dispatch loop
Inputs / outputs: supported-range facts (W02 readiness) → classification
Preconditions / postconditions: executing on a LocalReady pCPU in IRQ
  context with the interface enabled; the returned classification is
  complete (no re-read needed to decide)
State and ownership change: the GIC transitions the acknowledged
  interrupt to active (combined posture) as a hardware side effect —
  acknowledged means completion-obligated, which is why the dispatch loop
  (not the classifier) owns the subsequent EOI
Concurrency/allocation context: IRQ context; single volatile read; no
  allocation
Errors and failure guarantee: none; every value classifies
Security/authorization checks: hardware value treated as untrusted-adjacent
  data; no memory access derives from it before classification
Logic:
  v = read ICC_IAR1_EL1 (volatile)
  if v in no_pending_band        -> NoPending
  if v in reserved_band          -> ReservedBand(v)
  if v in assigned_band:
      if v within supported.id_space() -> Assigned(try_construct(v).unwrap_or(
           debug-invariant-violation))   # constructor must accept every
                                         # in-range value; mismatch is a
                                         # build-time invariant bug
      else -> OutOfSupported(v)
Validation: exhaustive unit classification tests per pinned revision;
  QEMU behavior via dispatch evidence (W03-DV03/DV05)
```

## 4. Named outcome behaviors (Required)

| Classification | EOI? | Registry touched? | Diagnostic |
|---|---|---|---|
| `Assigned` with consumer | yes | slot read (acquire) | none (normal path) |
| `Assigned` without consumer (consumerless) | yes | slot read only | counter + rate-limited event with ID and pcpu |
| `OutOfSupported` | yes (it was acknowledged) | never | counter + rate-limited event; ID recorded numerically, never cast to a typed ID |
| `ReservedBand` | no (architecturally not completed; verified against pinned revision) | never | counter + event |
| `NoPending` | no | never | counter only; loop exit |

The `OutOfSupported` row is the structural answer to "no invalid
index/handler": such values complete as data. The `ReservedBand`
no-EOI rule is a pinned-revision fact recorded at implementation; if the
pinned revision says a band requires completion, the table row changes
with it — the invariant that every band has one recorded behavior does not.

## 5. `SupportedRange`

```text
Name and stability: struct SupportedRange { sgis, ppis: fixed, spi_max:
  u16, id_space(): continuity check } (P6-internal; constructed once from
  W02 DistributorReadiness facts and the W01 expected identity)
Purpose and caller: bounds every classification and registry operation;
  shared read-only after construction
Preconditions: built only after W02 DistributorReady; values equal the
  confirmed read-back facts, never platform-declared wishes
Validation: consistency test against W02 readiness record
```
