# P1-W04 implementation record

**Status:** Mechanism implemented; W09 integration and runtime evidence pending.
**Scope:** P1 boot CPU architectural baseline.
**Version:** v0.1
**Owner/change context:** P1-W04, 2026-09-25.
**Supersedes:** None.

## Implementation and dependencies

The [detailed design](p1-w04-el2-architectural-state-baseline/README.md) and
its preflight amendment govern `hypervisor/src/arch/aarch64/baseline/`.
`specs.rs` owns pure control values/masks; `mod.rs` owns register access,
guard decisions, terminal failure and the baseline declaration. W03's merged
`capabilities::query(FactId)` is consumed directly; no ID-register re-read,
board constant or platform-name selection exists here.

The phase body checks W03 publication through its query contract, validates
SPSel/DAIF, then processes C2–C8 in order. HCR is written only once in C2;
C3 relies on its verified trap posture and C7/C8 reuse its RW/VM settings.
All writes are followed immediately by ISB and masked readback. A mismatch
panics with control name, expected and observed fields; there is no retry.
Before W05 installs vectors, an architectural access fault still falls within
the documented unowned-vector limitation.

Entry is `establish_el2_baseline()`; W09 owns its eventual invocation.
W02 still terminates at the explicitly unlinked W09 seam. Item-scoped dead-code
allowances on the entry and three downstream query functions identify their
owners and must be removed as consumers land; no stub simulates phase success.

## Exact values and source authority

Register field references: Arm Cortex-A57 TRM DDI0488F,
[system-control register descriptions](https://documentation-service.arm.com/static/5e906b9fc8052b1608760c7b),
cross-checked for CPTR/MDCR against Arm Cortex-A73 TRM 100048_0002_05_en
[register descriptions](https://documentation-service.arm.com/static/5e7b6b167158f500bd5bf124).
These are architectural baseline fields, not CPU-name branches. The optional
CNTHV access follows the W03 VHE existence guard; its AArch64 system-register
encoding is S3_4_C14_C3_1. The pinned softfloat target supplies the no-FP build
premise. ISB follows the merged preflight amendment.

| Control | Form | Mask / value |
|---|---|---|
| SPSel | assertion only | 1 / 1 |
| DAIF | assertion only | 0x3c0 / 0x3c0 |
| HCR_EL2 | full constant | all bits / RW bit 31 |
| CPTR_EL2 | RMW | TFP bit 10 / set |
| CPACR_EL1 | RMW | FPEN bits 21:20 / zero |
| MDCR_EL2 | RMW | TDOSA10,TDA9,TDE8,TPM6,TPMCR5 / set |
| MDSCR_EL1 | full constant | all bits / zero |
| CNTHCTL_EL2 | RMW | EL1 gates bits 1:0 / zero |
| CNTKCTL_EL1 | RMW | legacy EL0 gates/event fields bits 9:0 / zero |
| CNTHP_CTL_EL2 | full constant | verify bits 1:0 / IMASK=1,ENABLE=0 |
| CNTHV_CTL_EL2 | full constant, optional | same; skip when W03 reports Absent |
| SCTLR_EL1 / SCTLR_EL2 | RMW | M0,C2,I12 / zero |
| VTCR_EL2 | full constant | all bits / RES1 bit 31 |
| VTTBR_EL2 | full constant | all bits / zero |

RMW preserves unowned fields and architectural reserved read values. A full
timer write is 2, while verification ignores read-only ISTATUS. The design's
“VTCR zero within RES1 constraints” is concretely 0x80000000. Stage-2 remains
disabled by HCR.VM=0; these writes introduce no Stage-2 mechanism.

## Declaration and capability decisions

`baseline_status(BaselineCategory::{C1..C8})` returns Established only after
the category's writes and guard decisions pass. `baseline_value(ControlId)`
returns the observed masked fields or None before establishment/when skipped.
`control_status(ControlId)` exposes the optional control's SkippedAbsent.
C6 is Established when mandatory timer controls pass and CNTHV is either
verified or explicitly skipped; it does not claim an absent register exists.
Unreadable optional existence is terminal, not guessed absent.

The proposed once-write cell is realized entirely with safe atomics:
one single-entry flag, eight category flags, and fifteen immutable value/status
slots. Value store precedes release status; acquire readers observe a complete
value. The single-entry flag prevents a second writer; no reset exists.
This removes the need for the proposed unsafe declaration-cell boundary.
No U-007 entry is created. There are no allocations, locks or atomic retry loops.

| Category | W03 context / guard |
|---|---|
| C1 | W02 execution assertions |
| C2 | ExecutionLevel |
| C3 | VirtualHostExtensions contextual query; no enabling VHE |
| C4 / C5 | no feature guard; architectural deny posture |
| C6 | CounterFrequency, El2PhysicalTimer; El2VirtualTimer guards CNTHV |
| C7 | ArchProfile |
| C8 | PaRange, Granule4k for downstream translation context |

## Change report and handoff

Changed code: baseline module/specifications, AArch64 module index, and
source-shared host integration tests in `crates/host-test-baseline/tests/`.
Changed docs: this record, [verification](../verification/p1-w04-el2-architectural-state-baseline-verification.md),
implementation index, design entry link and unsafe inventory.

New unsafe: U-005 (closed register reads) and U-006 (writes + ISB). Declaration
uses safe atomics. Public ABI/API and dependencies: none; all mechanism queries
are crate-local boot interfaces. No TODO/FIXME or unresolved architectural
conflict. Known boundary: SCR_EL3 is firmware-owned, vectors remain W05-owned,
MMU enabling W08-owned, timer policy beyond this posture P6-owned.

W05 consumes C1–C4 and recorded HCR/CPTR values. W08 consumes C8 and HCR/SCTLR
recorded fields and supersedes SCTLR through its own design. W09 calls the
entry once after capabilities. W10 owns 100-boot consistency evidence.
W12 must carry the temporary baseline and unowned-vector limitations.

