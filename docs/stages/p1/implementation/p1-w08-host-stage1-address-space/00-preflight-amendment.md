# W08 preflight design correction

**Status:** Proposed detailed design; implementation not claimed.
**Scope:** Translation topology, MAIR, linker premises and activation.
**Version:** v0.2
**Owner/change context:** P1 preflight, 2026-09-24.
**Supersedes:** Direct L1-to-L3 topology, omitted MAIR programming, barrier
scope and assumed page-separated linker regions throughout this design.

## Baseline and foundation ledger

At a05fca6 W02 links execution at 0x40080040, has 16-byte class alignment,
and embeds stack in BSS. No tables exist.

| Required outcome | Missing foundation | Owner and acceptance |
|---|---|---|
| Valid 4 KiB page translation | intermediate L2 tables | W08; independent three-level walk |
| Explicit memory types | programmed MAIR matching AttrIndx | W08; encoding and readback review |
| Separate no-RWX classes | page-separated linker bounds | W08 with W02/W05 seam; layout review |
| Visible tables/control effects | complete activation barriers | W08; emitted sequence review |

These foundations are Required. Dynamic maps, relocation and SMP are Reserved;
allocator, Stage-2 and permanent identity-map ABI are Out of Scope.

## Correct table topology and attributes

Use 39-bit VA, 4 KiB granule, T0SZ=25, and L1 root. A table descriptor
advances one level: L1 → L2 → L3. The direct L1 → L3 text is superseded.
Fixed storage comprises five individually 4 KiB-aligned, 4096-byte pages:
one L1, two L2 (image/console), two L3 (image/console). Larger enclosing
alignment must not introduce unaccounted page offsets.

Reference image L1[1] leads to image L2[0]; console L1[0] leads to console
L2[72]. Each L2 entry points to the appropriate L3. Compute indices with
typed checked arithmetic. Assert the image inventory fits the 2 MiB window
starting 0x40000000 and console inventory its own 2 MiB window. Failure stops
before enable; block mappings remain prohibited. Arm's
[memory management guide](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/LearnTheArchitecture-MemoryManagement-101811_0100_00_en.pdf)
describes the 4 KiB indexing geometry.

W08 owns MAIR_EL2 in addition to TTBR0_EL2/TCR_EL2. Slot 0 is 0xee (Normal
inner/outer WB read allocate); slot 1 is 0xff (Normal inner/outer WB read/write
allocate); slot 2 is 0x04 (Device-nGnRE); unused slots are zero.
CodeRx/RoData/Vectors select 0, DataRw/BootStack select 1, ConsoleMmio selects 2.
The descriptor decoder verifies AttrIndx against MAIR and expected class.
AP, XN, AF and reserved bits must follow the non-VHE EL2 descriptor regime,
not unexamined EL1 constants.

## Linker and typed inventory seam

W08 extends the existing W02 linker extension point with page-separated
code/rodata/data/BSS/stack bounds. The first code mapping begins 0x40080000
despite execution starting after the 64-byte Image header. The stack is
excluded from generic data mapping and given distinct bounds while still
inside the entry-time BSS clear range. W05 supplies a dedicated vector page.
Map table storage DataRw; the shared UART window covers both early writers.
BootContext retention satisfies boot-data scope; DTB contents remain unmapped.

## Activation contract

Keep premise/build/verify/postcheck APIs and extend audited accessors to
MAIR write/read and translation-control readback. After table construction
and full verification with caches off:

1. DSB SY; TLBI ALLE2; DSB SY; ISB.
2. IC IALLU; DSB SY; ISB.
3. Write MAIR_EL2, TTBR0_EL2 and TCR_EL2; ISB; read back and compare documented
   meaningful fields against expected values.
4. Set SCTLR_EL2 M/C/I over the established baseline; immediately ISB.
5. Perform existing post-MMU checks and return to the W09 continuation.

SY is deliberately conservative for non-shareable single-CPU mappings.
No table cache clean is needed only under W01/W04's caches-off construction
premise. No mapping changes after enable or shootdown service are introduced.
Failures retain the existing terminal Stage1 route; transition exceptions use
W05/W07. No retry or partial-success declaration is permitted.

## Acceptance

W08-DV02/DV03 independently walk all three levels, decode actual attributes,
and establish inventory/table set equality in both directions. Inspect bounds,
five-page layout and the full activation sequence. Post-MMU sentinel reads and
writes must actually access memory, rather than optimize to constants.
W10 proves repeated continuation; W11 proves permission/unmapped faults and
diagnostics. Actual results belong in separate verification records.

