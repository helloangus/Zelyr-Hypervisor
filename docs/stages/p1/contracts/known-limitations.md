# P1 known limitations and open evidence

**Status:** Proposed current-state assembly; limitations are not closure claims.\
**Scope:** P1 assumptions, exclusions and current evidence gaps; no promised mitigation design.\
**Version:** v0.1.\
**Owner/change context:** P1-W12 consolidation of W01–W11 records, 2026-09-25.\
**Supersedes:** None.

| Limitation or open item | Owning source and consequence |
|---|---|
| Canonical loader assumptions include Non-secure EL2, AArch64 and MMU/cache-off; only EL2 and nonzero DTB pointer are checked before transfer. | [W01 entry contract](../implementation/p1-w01-reference-boot-contract/01-boot-contract.md). Other firmware/board paths are not validated. |
| Reference PL011 address, static image/table storage and one boot CPU are fixed bring-up assumptions. | [W01 selection](../implementation/p1-w01-reference-boot-contract-record.md), [W06 record](../implementation/p1-w06-early-console-logging-record.md), [W08 record](../implementation/p1-w08-mmu-activation-record.md). P1 supplies no discovered platform or SMP service. |
| Active DTB is retained but not validated, dereferenced or mapped; physical RAM and reservations are not discovered. | [W01 contract](../implementation/p1-w01-reference-boot-contract/01-boot-contract.md), [W08 mapping](host-address-space.md). P2-W01–W05 own intake, map and allocation. |
| Exceptions before `exceptions.complete` have no guaranteed hypervisor-owned vector route. | [W09 failure matrix](../implementation/p1-w09-initialization-sequencing/01-init-state-machine.md#4-failure-routing-matrix). This is a pre-vector window, not a passing negative test. |
| Identity VA=PA is temporary; no remap or permanent Host-VA ABI exists. | [W08 mapping](host-address-space.md). [W11 NC5](../verification/p1-w11-negative-fault-validation-verification.md) locally exercises one post-MMU translation fault; it is not a general address-space or hardware proof. |
| WFI idle assumes the reference firmware does not trap the instruction; real firmware policy has not been validated. | [W09 record](../implementation/p1-w09-initialization-sequencing-record.md). |
| W03 required-capability NC2 has a validation-image sample substitution, not a real CPU lacking 4 KiB support. | [W11 foundation](../implementation/p1-w11-negative-fault-validation-record.md); environment-only NC2 unavailable on recorded QEMU 8.2.2. |
| W10 R1–R6 are recorded, with NC4 replacing the historical provisional NC2 R2 control; the accepted 100/100 reference run is one image on one host. Raw evidence remains in local W10/W11 worktrees, not a durable CI artifact. | [W10 verification and NC4 addendum](../verification/p1-w10-qemu-boot-regression-verification.md), [W11 verification](../verification/p1-w11-negative-fault-validation-verification.md). These locally support P1-V16 and P1-V17 within their stated evidence bounds, not real hardware or all P1 gates. |
| W11 NC1–NC5 have paired reference-QEMU evidence and S1–S6 passed local source/linked-image review; neither is a real-board or hardware-fault validation. NC2 is a controlled sample substitution, not absent CPU hardware capability. | [W11 integrated review](../verification/p1-w11-negative-fault-validation-verification.md). NC6 alone remains unexecuted in the specified genuine unexpected-vector class. |
| NC6 real unexpected asynchronous vector injection has no accepted executed proof. | [W11 scenario matrix](../implementation/p1-w11-negative-fault-validation/01-fault-scenario-matrix.md) and [verification](../verification/p1-w11-negative-fault-validation-verification.md) record the historical P1 block. [ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md) transfers this still-open execution obligation to P6-W12/P6-V29; P1-V18 is revised, not retroactively passed. |
| P1 has no allocator/discovery/GIC/SMP/Guest, Stage-2 or VM mechanism. | [P1 task book](../task-book-v0.2.md#2-scope-classification); these are stage exclusions, not defects to patch inside W12. |

The [unsafe inventory](../../../security/unsafe-inventory.md) is the
authoritative boundary ledger. Package implementation records report new
unsafe, internal API, ABI and dependencies; package verification records state
what ran. S1–S6 source/linked-image review is recorded by W11, including independent
static acceptance of new U-016/U-017; the [stage-gate
review](stage-gate-evidence-map.md) keeps its revised P1 scope and the
transferred NC6/P6-V29 blocker visible. W12 adds no `unsafe`, Rust/public ABI
or dependency.
