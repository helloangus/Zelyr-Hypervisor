# P1-W08 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W08 detailed design](README.md).

## 1. Logical module map

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Address types and arithmetic | typed addresses, page/offset/index math, region sizing | the extended `PhysAddr` ops, the `VirtAddr` newtype | raw values from symbols/constants | checked, typed values | region policy, descriptor encoding policy |
| Descriptor and table layer | encode/decode descriptors; build and walk the L1/L3 tables | `STAGE1_TABLES` static (L1 + L3 pages, .bss) | region inventory entries | built, verified tables | the transition decision, the enable sequence |
| Region inventory | the authoritative list of mapped regions with symbols, size, class | `MAPPED_REGIONS` static table | linker symbols; W05/W06 region contracts | build inputs; review evidence | discovering regions at runtime, region content |
| Transition body | the `stage1` phase's single entry: assert → build → verify → enable → verify | orchestration only | W09 phase call (`stage1_step`) | post-MMU environment, or fatal route | consumers' continuity behavior (they report their own liveness) |
| Sysreg/asm boundary | TTBR0/TCR/SCTLR writes + read-backs; `dsb`/`isb`/`tlbi alle2`/`ic iallu` | none | typed values | register/ instruction effects | MMIO (W06 owns the console), exception registers (W05) |

## 2. Region inventory (work seq 1)

The authoritative list of everything P1 maps. Each row names its source
seam; sizes come from linker symbols or the owning package's declared
constants — never from runtime discovery.

| # | Region | Bounds source | Owning seam | Class | Why mapped |
|---|---|---|---|---|---|
| 1 | Executable code (`.text`, incl. entry asm, vector code references, all P1 runtime code) | linker text symbols (P0 extension points) | P0 target baseline (assumed); W02 entry | `CodeRx` | the executing image |
| 2 | Read-only data (`.rodata`: W03 fact tables, W04 specs, W05 EC table, W07 literals, string constants) | linker rodata symbols | P0 baseline (assumed) | `RoData` | static tables and literals |
| 3 | Writable/zero data (`.data` + `.bss`: `BOOT_CONTEXT`, `CAPABILITIES`, `EL2_BASELINE`, `VECTOR_STATE`, `CAPTURED_FRAME`, guards/flags, `TRACKER`, `CHANNEL_AVAILABLE`, `STAGE1_TABLES`) | linker data/bss symbols (`__p1_bss_start/end` among them) | P0 baseline (assumed); W02 BSS | `DataRw` | all mutable statics |
| 4 | Boot stack | `__p1_boot_stack` / `__p1_boot_stack_top` | W02 entry contracts §2 | `BootStack` | the boot CPU's only stack (separate class so stack pages are never executable and code pages are never writable) |
| 5 | Vector table | `__p1_vectors_start` / `__p1_vectors_end` | W05 install contracts §4 | `Vectors` | the live exception entry path |
| 6 | Early-console MMIO window | `CONSOLE_REGION` (base + size) | W06 mapping-requirement interface §6 | `ConsoleMmio` | the diagnostics transport post-transition |

Explicitly **not** mapped: the firmware-delivered DTB (retained pointer
only — parent README decision 8); any free RAM beyond the image (no
allocator in P1); any other device (P1 touches none); the GIC or timer
registers (no driver in P1). The plan's "boot-time data" class is
satisfied by row 3's `BOOT_CONTEXT` retention, not by a DTB mapping.

## 3. Mapping-class model (work seq 2)

Six classes; every mapped page belongs to exactly one. Semantics are
stated here; the concrete descriptor encoding (attribute fields, AP/SH
orgn/irgn values, reserved-bit handling) is recorded against the
architecture revision per
[address types](02-code-contracts-address-types-and-tables.md) §3.

| Class | Memory type | Cacheability | Executable | Writable | Regions |
|---|---|---|---|---|---|
| `CodeRx` | Normal | inner write-back, read-allocate | yes | no | 1 |
| `RoData` | Normal | inner write-back, read-allocate | no | no | 2 |
| `DataRw` | Normal | inner write-back, read/write-allocate | no | yes | 3 |
| `BootStack` | Normal | inner write-back, read/write-allocate | no | yes | 4 |
| `Vectors` | Normal | inner write-back, read-allocate | yes | no | 5 |
| `ConsoleMmio` | Device | Device-nGnRE semantics | no | yes | 6 |

Rules:

- K1 `BootStack` exists as a separate class although its attributes equal
  `DataRw`'s: the distinction is the review unit (P1-V13 names the boot
  stack explicitly; a merged class would let stack pages silently merge
  into data reasoning).
- K2 `Vectors` is executable and read-only — the table must never be
  runtime-writable (W05's ownership matrix becomes a hardware-enforced
  property).
- K3 Shareability for the Normal classes is recorded as non-shareable
  (single executing CPU, no DMA in P1); the record names P3 as the owner
  of any shareability change (Reserved trigger).
- K4 Every class's attribute set is total and unambiguous: no class
  leaves an attribute "whatever the hardware default is".

## 4. Page-table objects and ownership

```text
STAGE1_TABLES (static, .bss, 16 KiB-aligned base):
  L1:  [Entry; 512]    one level-1 table; TTBR0_EL2 target
  L3a: [Entry; 512]    level-3 pages for the image window
  L3b: [Entry; 512]    level-3 pages for the console window
```

- Ownership: W08 owns the tables and every descriptor in them; no other
  package writes them; no runtime mapping change exists in P1 (monotone
  construction during the `stage1` phase, then frozen).
- Sizing: two L3 tables cover the image window and the console window
  with headroom recorded in the sizing arithmetic; the counts are
  constants with recorded derivation — growth is a design change.
- Addressing: the tables are reached through their addresses computed by
  the typed arithmetic; the physical base of the image (and hence of the
  tables) is the canonical-path load address recorded at implementation
  from the W01 contract's memory assumptions.
- No blocks (parent README decision 2); table descriptors point to L3
  pages; L3 entries are 4 KiB page descriptors only.

## 5. Transition state machine (work seq 3)

```text
IDENTITY_PRE_MMU (W04 C8 verified state: SCTLR_EL2 M/C/I = 0)
  -> assert premises        [§3 of the transition contracts]
  -> build tables           [address types §4]
  -> verify tables          [address types §5]
  -> barriers + TLB invalidate
  -> program TTBR0_EL2, TCR_EL2 (+ isb)
  -> set SCTLR_EL2 M|C|I    (+ isb)        [the enable step]
  -> post-MMU verification  [transition §5]
POST_MMU_IDENTITY (recorded temporary assumption)
any premise/verification failure -> fail_phase(Stage1, reason)  [terminal]
any fault during the sequence    -> exception path (W05) -> armed fatal path
```

Rules: single-shot (no retry, no partial enable); the steps are ordered
and each has a recorded reason; the enable step is the only
`SCTLR_EL2` mutation in P1 and supersedes W04's C8a value through this
design (W04's ownership matrix names this supersession path explicitly).

## 6. Continuity obligations across the transition (work seq 4)

| Consumer | Obligation W08 accepts | Proof owner |
|---|---|---|
| Execution | the image window (code/rodata/data/stack) is mapped with its classes before enable | W08-DV03 verification; W10 R1 |
| Vectors | the vector region is mapped `Vectors` class; `VBAR_EL2` is unchanged by the transition (identity window) so the W05-declared base stays valid | W08-DV04 table review; NC3/NC6 (W11) |
| Console | `CONSOLE_REGION` is mapped `ConsoleMmio` class before enable; the VA is unchanged (W06 integration §3) | W08-DV04; the post-`stage1` markers (W09/W10 R1) |
| Fatal diagnostics | both transport windows' code/data are within rows 1–3; the armed fatal path needs no new mapping to report a transition failure | W08-DV04; NC5 (W11) |

W08 emits nothing itself across the transition; the first post-MMU
console access is W09's `stage1`-phase marker emission, which is by
construction after `stage1_step` returns — the designed place where an
unmapped console would surface as a diagnosable fault rather than a
silent one.

## 7. Concurrency model

Boot CPU only; `DAIF` masked; no allocation; no locks. Table
construction is straight-line code over private statics; the enable
sequence is single-shot on the boot CPU; the only architectural
synchronization is the recorded barrier discipline (address types §6).
No SMP, no DMA, no second mapper exists in P1 — the concurrency
statement is the shareability record (K3) plus the single-writer
ownership of §4.

## 8. Assumed contracts and failure boundaries

| Seam | Supplied by | Used for | Failure boundary if it delivers differently |
|---|---|---|---|
| Pre-MMU baseline values; declaration API (`baseline_value`, `baseline_status`) | [W04](../p1-w04-el2-architectural-state-baseline/README.md) (accepted design) | premise assertions (`SCTLR_EL2.M=0`, `HCR_EL2.VM=0`, C8 established); the superseded-value path | a missing or mismatching recorded value is a phase failure (`stage1`, fatal route); W08 never re-reads to "fix" it |
| Region symbols and layout extension points | P0 target baseline (planned); W02 symbols | inventory bounds | a missing extension point is the recorded P0 blocker (task book §1); no private linker fork |
| Vector region identity | [W05](../p1-w05-el2-exception-entry-baseline/README.md) (parallel design) | row 5 of the inventory | symbol/class mismatch is a W05/W08 coordination issue raised, not absorbed |
| Console region + attributes; VA continuity | [W06](../p1-w06-early-console-logging/README.md) (parallel design) | row 6; the transition continuity | an attribute/VA conflict is a W08 design change recording the W06 impact; the constant is never edited locally |
| Armed fatal path; `report_fatal_phase` | [W07](../p1-w07-fatal-crash-diagnostics/README.md) (parallel design) | the failure route of §5 | if the fatal path were not armed (illegal phase order), the premise assertion fails first and routes — the ordering is W09's guarantee |
| `stage1` phase placement; `fail_phase` arms | [W09](../p1-w09-initialization-sequencing/README.md) (accepted design) | when the transition runs; failure attribution | seam mismatch raised per W09 §1 |
| `PaRange`, `Granule4k` query API | [W03](../p1-w03-aarch64-capability-inventory/README.md) (accepted design; trigger exercised per parent README decision 3) | `TCR_EL2` configuration inputs | a missing/absent fact is a phase failure with W03's vocabulary; W08 does not re-read ID registers |
| Address type-safety red lines | P0-W15 (planned) | the typed-arithmetic discipline | upstream defect recorded per [workflow](04-implementation-and-review.md) §1; the discipline is applied regardless |

Produced for consumers: the post-MMU identity environment with its class
table, the verified page tables, the superseded `SCTLR_EL2` value, the
continuity guarantees, the temporary-assumption record, and the NC5
continuation point.
