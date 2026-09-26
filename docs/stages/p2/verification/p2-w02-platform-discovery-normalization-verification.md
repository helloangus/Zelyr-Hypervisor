# P2-W02 verification

**Status:** Passed for declared W02 facts; P2-V03/P2-V04 met in the bounded scope.
**Scope:** Allocation-free discovery/normalization and current boot handoff.
**Version:** v0.1
**Owner/change context:** P2-W02, 2026-09-26.
**Supersedes:** No previous implementation evidence.

The [implementation record](../implementation/p2-w02-platform-discovery-normalization-record.md)
records representation and semantic corrections. Environment, exact commands,
image hashes, 52-test result and QEMU artifacts are shared with
[W01 verification](p2-w01-boot-platform-description-intake-verification.md).
They are one executed evidence set, not two independent runs.

| Validation | Status | Evidence |
|---|---|---|
| W02-DV01 / P2-V03 | Passed | All synthesized discovery fixtures pass through the production W01 validator; no bypass parser or raw-byte access exists. The boot adapter reaches both package markers. |
| W02-DV02 | Passed | Enabled/disabled/malformed CPU records, auxiliary cpu-map skip, 1/2-cell decoding policy, zero-size CPU cells, capacity and DT-order assertions. |
| W02-DV03 | Passed | Matching boot CPU, disabled/unmatched boot CPU and empty inventory outcomes tested; actual four-CPU QEMU boot reports boot index zero. |
| W02-DV04 | Passed | One/two-cell memory, multiple banks, zero-sized banks, invalid widths, overflow and no-usable-bank diagnostics. |
| W02-DV05 | Passed | Header reservations followed by reserved-memory records, zero-address preservation, no-map flag, bad/missing reg and unsupported translation remain explicit; capacity exhaustion is fatal. |
| W02-DV06 | Passed | GICv3, GICv2, absent GIC, 64-bit/zero stride, interrupt-cell and capacity errors; timer v7 and phandle mismatch; PSCI 0.1/0.2/1.0, missing/bad conduit; status annotations and duplicate singleton counters. |
| W02-DV07 | Passed | Direct and one-hop aliased console paths, unresolved/alias-chain results, bad/capped strings, initrd equal/inverted range; resolved node path retained. |
| W02-DV08 / P2-V04 | Passed | State assertions preserve absent/unsupported/unusable/usable; PCI/SMMU/ACPI remain NotDiscovered. Capability projection uses the same states; no Boolean collapse. |
| W02-DV09 / P2-V04 | Passed | `PlatformInfo` fields private; only facts and read-only accessors exposed. Mechanical search of `hypervisor/src/platform` finds no QEMU/RK3566/Orange-Pi/board selection and no unsafe or allocator use. |
| W02-DV10 | Passed | Repeated normalization yields field-wise identical records; mutation sweep also traverses every accepted mutated input without panic. Rust padding bytes are not compared. |

The canonical QEMU case reports one CPU/one RAM bank and usable timer/PSCI;
its GICv2 description is honestly unsupported. Selecting GICv3 yields a usable
GIC fact, including in the four-CPU case. No GIC/timer/PSCI execution, AP startup,
RAM allocation or guest capability is inferred from these records.

Not run/claimed: W07 real fixture corpus/RK3566, W09's full matrix, W10 formal
P3/P4 handoff, hardware or other firmware. The bootstrap read envelope and
immutable-input premise remain W01 constraints; discovery of RAM never expands
that pre-discovery authority. Other P2 packages remain unimplemented here.
