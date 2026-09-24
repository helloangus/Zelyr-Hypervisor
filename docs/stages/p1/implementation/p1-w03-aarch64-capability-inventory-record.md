# P1-W03 capability inventory implementation record

**Status:** Implemented mechanism and host validation; runtime integration pending W09.
**Scope:** W03 only; no P1 completion claim.
**Owner/change context:** P1-W03, 2026-09-24, base a05fca6.
**Design:** [W03](p1-w03-aarch64-capability-inventory/README.md), including
[reconciliation](p1-w03-aarch64-capability-inventory/05-implementation-reconciliation.md).

## Delivered boundaries

`hypervisor/src/arch/aarch64/capabilities/facts.rs` contains immutable report
types, pure decoding/classification, deterministic required verification and
bounded rendering. `mod.rs` contains the six MRS wrappers, once-cell and
`build_capability_report`, `query`, `render_report` consumer seams. The new
architecture modules are internal to the existing binary. The exact pure
file is included by the existing host member's integration harness.

No new dependency, external ABI, feature, control-register write, MMIO,
allocator, DTB parser, GIC/timer/Stage-2 mechanism or platform-name branch is
introduced. Unsafe growth is U-003 (six register reads) and U-004 (cell Sync,
write and borrow). Root independent systems soundness review accepted both
boundaries on 2026-09-24; the inventory carries the explicit verdict.

## Fact ledger

References/revision are in the reconciliation. All fields are decoded from
one snapshot; no fact depends on a platform name.

| Fact / source | Bits and decoded value | Class / purpose |
|---|---|---|
| ExecutionLevel / CurrentEL | [3:2], value 2 required | Required / EL2 continuation |
| CpuAffinity / MPIDR_EL1 | Aff3 [39:32], U [30], MT [24], Aff2..0 [23:0]; mask 0xff41ffffff | Optional / boot CPU evidence |
| ArchProfile / ID_AA64PFR0_EL1 | [15:0] packed EL3..EL0 widths | Optional / EL2 and future Stage-2 context |
| GicVersion / PFR0 | [27:24], 1 or 3; other values Absent | Future / CPU-side interface only |
| PaRange / MMFR0 | [3:0], 0–6 → 32,36,40,42,44,48,52 bits | Optional / later W08 limit |
| AsidBits / MMFR0 | [7:4], 0/2 → 8/16 | Future / later memory work |
| Granule4k / MMFR0 | [31:28], 0/1 → Present(1) | Required / P1 mapping |
| Granule16k / MMFR0 | [23:20], 1/2 → Present(1) | Optional / translation context |
| Granule64k / MMFR0 | [27:24], 0 → Present(1) | Optional / translation context |
| VirtualHostExtensions / MMFR1 | [11:8], 1 → Present(1) | Future / VHE, not a Stage-2 test |
| El2VirtualTimer | derived from VHE | Optional / W04 write guard |
| CounterFrequency / CNTFRQ_EL0 | [31:0], nonzero Hz | Required / timer platform viability |
| El2PhysicalTimer | EL2 implies presence | Optional / W04 target knowledge |

Unknown encodings conservatively mean Absent; reserved bits are masked.
VA limits remain documented architectural 48-bit P1 design knowledge, not
a new register fact. DTB discovery belongs to P2, actual Stage-2 to P4,
GIC to P6, topology to P3. The report is not a P2 public discovery type.

## Frozen output and handoff

Each `cpu.capability.fact` event is `cap <label>=<value> (<classification>)`.
Labels are the kebab-case names in `FactId::ALL` order. Values are decimal,
`Absent`, or `Unreadable`; classifications are `Required`, `Optional`, `Future`.
No newline is supplied; W06 frames the complete line. Stack buffer: 96 bytes;
13 lines; no allocation. Session identity comes from W09/W06.

Rejections are exactly:

```text
cap-reject fact=execution-level required: Non-secure EL2 execution
cap-reject fact=granule-4k required: 4 KiB granule support for P1 mapping work
cap-reject fact=counter-frequency required: non-zero system counter frequency
```

W02 panic transports the rejection. No partial report publishes. W04 consumes
the query API; its old misleading Stage2Support reference is corrected to
VirtualHostExtensions. W08 consumes translation knowledge through W04 unless
its design names a direct W03 dependency. W09 owns phase placement, rendering,
and removal of three seam dead-code allowances plus the observation re-export
allowance. Unreadable has a separate narrow allowance because production
register accessibility is a precondition, not a recoverable probe.

NC2's environment-only variant lacks real model variability on reference
QEMU; W11's 2026-09-24 design correction permits a labeled validation-image
variant. W11 owns that execution; synthetic host policy checks are not NC2
execution. W03's required set has not been changed
to manufacture a negative scenario. See [verification](../verification/p1-w03-aarch64-capability-inventory-verification.md)
for actual evidence and deferred runtime checks.
