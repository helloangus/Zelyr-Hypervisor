# W08 activation representation and audited boundaries

**Status:** Proposed detailed-design correction; implementation and validation
are recorded separately.
**Scope:** Safe static table storage, bounded temporary model, linker-bound
inventory, and post-MMU sentinels.
**Version:** v0.1
**Owner/change context:** P1-W08 activation preflight, 2026-09-25.
**Supersedes:** The mutable-static representation and plain-write detail in
[address/table contracts](02-code-contracts-address-types-and-tables.md) §§2,
4–6 where safe Rust cannot mutably borrow a static `[Descriptor; 512]`; the
six mapping classes, five-page hardware representation, and transition order
are unchanged.

## Goal-to-baseline ledger

| Required result | Observed state | Missing foundation | Owner and evidence |
|---|---|---|---|
| Five verified hardware-readable pages | Merged W08 pure `Tables` model, but no target table storage | One aligned, writable five-page static and checked copy from the model | W08; layout, full walk and target/QEMU checks |
| Exact region inventory | Merged linker bounds, but no consumer | Closed typed region list from bound *addresses* plus W06 console window | W08; bound/class review |
| Ordered EL2 activation | No W08 register instructions | Audited MAIR/TTBR/TCR/SCTLR, TLBI, I-cache and barriers | W08; code review, QEMU, later hardware |
| Meaningful postcheck | No mapped sentinel access | Forced rodata memory read and writable data write/read | W08; QEMU and W11 fault evidence |

All four are Required for P1-V13/P1-V14. Dynamic mapping, relocation and
SMP are Reserved; Guest Stage-2, allocator and a permanent identity-map ABI
are Out of Scope.

## Corrected representation and contracts

`STAGE1_TABLES` is one 4 KiB-aligned, 20 KiB static of 2,560 `AtomicU64`
words, exactly five contiguous hardware table pages in L1/image-L2/
console-L2/image-L3/console-L3 order. The single masked boot CPU builds and
fully verifies a `Tables` value on its 64 KiB stack, then copies every word
to the static with Relaxed stores and reads every word back. The temporary
model consumes 20 KiB plus bounded call frames; W08 must inspect generated
stack use against the W02 64 KiB bound before closure. `DSB SY` after the
copy, before TLBI/programming, gives the walker visibility. Hardware never
walks the temporary value. `STARTED` is a one-way atomic guard; an error is
terminal through W09, never retried. No `static mut`, `UnsafeCell` alias or
new Rust-unsafe table mutation is authorized. The static is writable image
data and included by the linker-bound `DataRw` region.

The non-VHE EL2 one-VA-range page encoding sets AP[1] (descriptor bit 6,
RES1) for **every** mapping class, in addition to AP[2] for read-only pages.
The host model test checks the six classes; independent architecture review
caught this missing bit before activation integration.

The inventory obtains only addresses of twelve uniquely defined linker
bounds. It does not read the declared `u8` objects. Every pair is checked for
page alignment, order, overlap and the fixed 2 MiB window by the pure model;
the retained W05 vector base must match the vector bound. Console bounds come
solely from W06's fixed region contract. Only these reviewed regions are
mapped; no DTB/free-RAM page is inferred.

`enable_host_stage1() -> Result<(), Stage1Error>` is the fallible mechanism
fixed by [the failure-seam reconciliation](05-failure-seam-reconciliation.md).
It checks W04 C8/SCTLR/HCR, W05 vectors, W06 channel, W07 readiness and W03
4 KiB/PA-range facts before touching tables. On successful build/copy, it
performs the amendment's `DSB SY; TLBI ALLE2; DSB SY; ISB; IC IALLU; DSB
SY; ISB`, writes MAIR/TTBR0/TCR, executes ISB/readbacks, then sets SCTLR
M/C/I and executes ISB. Postchecks inspect SCTLR, one forced rodata load and
one writable data sentinel. Any returned error retains a step and static
detail token; W09 converts it to a terminal Stage1 report before Complete.
An architectural fault uses W05/W07 instead of returning an error.

## Unsafe boundary ledger

| ID/category | Necessity and minimum boundary | Precondition, establishment and failure | Proof boundary |
|---|---|---|---|
| U-012 `arch-register` | Rust has no safe EL2 `msr`/`mrs`/`dsb`/`isb`/`tlbi`/`ic`; closed wrappers only | W01 EL2, masked single CPU, W03/W04/W08 values and fixed order; a false premise is FC-INVARIANT terminal | target encoding/order review, QEMU continuation, later hardware |
| U-013 `memory-mgmt` | linker symbols cannot be declared as Rust-owned statics; one extern block supplies address-space inventory bounds, addresses only | W08 linker script defines every symbol, pure inventory validates bounds; false layout is FC-INVARIANT terminal | ELF symbol/section review and model validation |
| U-014 `memory-mgmt` | compiler must perform a real rodata load after MMU enable, so one `read_volatile` is required | static aligned in-image u64, verified RoData mapping; false map faults terminally through W05/W07 | code/ELF review and executed post-MMU check |

These entries require same-change inventory records and a second arch/systems
soundness reviewer before merge. No other unsafe is authorized. Target build
of an uncalled mechanism proves compilation, not actual register effects;
W09/W10/W11 must still provide execution evidence. The design has no
completion claim or architecture-level policy change.
