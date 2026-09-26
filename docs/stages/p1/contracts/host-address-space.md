# P1 Host Stage-1 address space

**Status:** Proposed assembly of the W08 mechanism boundary; reference-QEMU post-MMU NC5 evidence recorded.\
**Scope:** Fixed boot-CPU EL2 mappings and controlled transition, not virtual-memory service or Stage-2.\
**Version:** v0.1.\
**Owner/change context:** P1-W12 assembly of [W08 mapping contract](../implementation/p1-w08-host-stage1-address-space/01-architecture-and-state.md), [activation record](../implementation/p1-w08-mmu-activation-record.md), 2026-09-25.\
**Supersedes:** None.

| Class | Region | Memory | Access | Execute |
|---|---|---|---|---|
| `CodeRx` | boot code and `.text` | Normal WB | read-only | yes |
| `Vectors` | EL2 vector page | Normal WB | read-only | yes |
| `RoData` | `.rodata` | Normal WB | read-only | no |
| `DataRw` | `.data`/`.bss`, including boot context and tables | Normal WB | read/write | no |
| `BootStack` | boot CPU stack | Normal WB | read/write | no |
| `ConsoleMmio` | W06 PL011 window | Device-nGnRE | read/write | no |

The current [implementation](../implementation/p1-w08-mmu-activation-record.md)
derives page-aligned bounds from linker symbols and the W06 console region,
builds/verifies a fixed five-page 4 KiB table model, then copies/reads back a
static table. It checks W03–W07 prerequisites, invalidates EL2 translations
and instruction cache with barriers, programs MAIR/TTBR0/TCR, and enables
SCTLR M/C/I once. Postchecks read SCTLR, `.rodata`, and a writable-data
sentinel. No class is RWX; tables are not remapped after enablement. See the
[W08 transition owner](../implementation/p1-w08-host-stage1-address-space/03-code-contracts-mapping-and-transition.md)
for ordering and error semantics.

The current identity VA=PA window is a temporary bring-up implementation,
**not** a permanent identity-map ABI for P2 or a promise to future consumers.
The supplied DTB, unused RAM and other MMIO are not mapped. P2 owns validated
DTB access, physical-memory discovery and dynamic allocation; later remap
work requires its own design. The [W09 integrated boot and linked-ELF
observation](../verification/p1-w09-initialization-sequencing-verification.md)
and [W10 normal-run repetition](../verification/p1-w10-qemu-boot-regression-verification.md)
show reference-QEMU continuity after enablement. [W11's NC5 paired
run](../verification/p1-w11-negative-fault-validation-verification.md)
observed two post-MMU translation faults at unmapped VA `0x5000_0000`,
with ESR.EC `0x25`, matching FAR, `stage1.complete` attribution and terminal
reports. These are evidence for that local path, not a hardware cache/TLB
proof, arbitrary Host VA ABI, or general mapping service.
