# P1-W01 Reference Boot Contract — Verification Record

**Status:** Reviews complete; executable matrix entries deferred by contract.  
**Date:** 2026-09-19 (Asia/Shanghai)  
**Environment:** development host, Linux (WSL2) x86_64,
`qemu-system-aarch64` 8.2.2, pinned toolchain `1.98.1`, branch
`p1/w01-boot-contract`.  
**Design:** [W01 detailed implementation
design](../implementation/p1-w01-reference-boot-contract/README.md) ·
[implementation record](../implementation/p1-w01-reference-boot-contract-record.md)  
**Parent validation IDs:** P1-V01 (canonical boot contract), P1-V02
(unsupported-entry rejection).

## Validation matrix results

| ID | Task-book ID | Test or review | Result | Evidence and command | Proves / does not prove |
|---|---|---|---|---|---|
| W01-DV01 | P1-V01 | P0 prerequisite inspection | **passed** | read P0-W03/W09/W16 delivered contracts and handoff-map §4.1; `git ls-files` at baseline `7688925`; states recorded in the implementation record's prerequisite section | every consumed P0 contract named with observed state; not that P0 stays immutable |
| W01-DV02 | P1-V01 | Contract content review | **passed** | implementation record "Materialized contract content" walked against [01-boot-contract.md](../implementation/p1-w01-reference-boot-contract/01-boot-contract.md) §2–§4, §6–§7: canonical path, E1–E8 with the assumption/check split, parameters/DTB/memory, transfer guarantee, recipe all present; no assumption stated as a check | the contract is reviewable and complete; not that a boot satisfies it |
| W01-DV03 | P1-V02 | Boundary semantics review | **passed** | §5 classes walked against [02-entry-validation-contracts.md](../implementation/p1-w01-reference-boot-contract/02-entry-validation-contracts.md) §1–§3: T1 before T2, R1–R5 hold by construction, single-source constant rule intact (one definition named in the record) | the boundary is explicit and non-continuing by design; not that it fires on real firmware (W01-DV06/W11) |
| W01-DV04 | P1-V01 | Layering and reserved-scope review | **passed** | §8 reconciliation recorded; one reference-platform constant (`0x09000000`), single-source, documented; search over W01 artifacts found no board/QEMU-name branch, no Guest/discovery/allocator item, no second canonical path; Reserved list explicit | ADR-041/043 compatibility of the boot path as designed; not whole-tree platform-portability compliance |
| W01-DV05 | P1-V02 | Transfer and routing review | **passed** | §4–§5 read against W09's design decision 6 and the H4/H6 rules: delegated `entry`/`runtime` records match; the §5 post-transfer refinement row is consistent (route established before the window) and stands as a recorded matrix amendment for W09-DV04 to materialize | seam compatibility as designed; not W09's implementation |
| W01-DV06 | P1-V01, P1-V02 | Executed evidence (normal boots; unsupported environment) | **not run — deferred to W10/W11** | normal-path execution belongs to W10's regression on the integrated path; NC1/NC2 execution belongs to W11. Until those records exist, P1-V01's executed half and P1-V02's executed half are unproven; no W01 artifact reports otherwise | — (deferred) |
| W01-DV07 | P1-V01/P1-V02 | Consumability review | **passed** | implementation record's handoff section read as W02 (entry-state table + verbatim tier contracts), W03 (EL precondition), W09 (tracker delegation + refinement), W10 (recipe + forbidden marker), W11 (NC1 spelling), W12 (assemblable content + limitations); each consumer can act without inventing W01 policy | handoff readiness; not downstream completion |

## Informative manual-investigation evidence (not a matrix entry)

Boot-recipe exploration on the development host (human-investigation boots;
per the P0-W09 contract these are not automation entries and were not
promoted into scripts or CI). Commands and raw-body assertions are recorded
in the implementation record. Observed on `qemu-system-aarch64` 8.2.2:

- `virt,virtualization=on` + `cortex-a57` + derived Image-mode kernel:
  entered at Non-secure EL2 (`CurrentEL = 0b10`) with `x0 = 0x44000000`
  (DTB base, non-zero) and `x1–x3 = 0` — the §2 properties hold, including
  T1/T2 pass conditions.
- The same kernel as a plain ELF via `-kernel`: EL2 entry holds but
  `x0 = 0` (`-dtb` does not change it) — the finding that selected the
  derived-image form.
- `virtualization=off` + `cortex-a57`: entry at EL1 — a probe-level
  rejection outcome consistent with NC1's environment class.

This evidence supports the recipe-field selections (W01-DV02/DV04) and the
NC1 feasibility expectation; it does not execute the W01-DV06 matrix entries
and proves nothing about the integrated W02 image's runtime behavior.

## Summary against the parent validation IDs

- **P1-V01:** contract content complete and reviewed (DV01–DV02, DV04, DV07
  passed); the executed "reference boot reaches validated Non-secure EL2"
  half is **deferred to W10** and currently unproven.
- **P1-V02:** boundary semantics, reason vocabulary, and evidence
  definitions reviewed (DV03, DV05 passed); the executed rejection proof is
  **deferred to W11 (NC1/NC2)** and currently unproven.

No entry here proves P1-V03 through P1-V21.
