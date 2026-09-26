# P1 Completion Report — AArch64 EL2 Minimum Bring-up

**Status:** P1 stage completion decision for the declared reference-QEMU scope.
**Scope:** P1 task book v0.2 gates and seven exit conditions; no P2/P6 or real-hardware completion claim.
**Version:** v0.1.
**Owner/change context:** P1 L7 stage review, 2026-09-26 (Asia/Shanghai), after ADR-061.
**Supersedes:** No package verification record; historical v0.1 NC6 findings remain intact.
**Basis:** [current task book](../task-book-v0.2.md), [ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md), [stage evidence map](../contracts/stage-gate-evidence-map.md), and W01–W12 implementation and verification records.

## Decision and proof boundary

**P1 is complete within its declared single-boot-CPU QEMU `virt` reference
scope.** The review below finds evidence for P1-V01–P1-V21 and all seven
exit conditions. This decision does not assert real-board operation, other
firmware/CPU configurations, a permanent identity-map ABI, an IRQ/GIC
service, or executed asynchronous IRQ/FIQ/SError delivery. In particular,
the original NC6 scenario is **not passed**: [ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md)
requires genuine asynchronous execution at P6-W12/P6-V29 after Host GIC/IRQ
readiness. P6 cannot infer that evidence from this P1 decision.

All 12 packages have an L5 implementation record and an L6 verification
record; W08 and W09 additionally have foundation records. The
[implementation index](../implementation/README.md) links their owners.
Earlier package records deliberately describe their then-current deferred
work. The merged W09–W12 records, ADR-061 governance addenda, and the fresh
integrated run below are the later evidence; no old result is rewritten as
having passed when it was first recorded.

## Fresh integrated reference run

Source was merged `main` at `ccd6fc0e09da1385afd3676c6be63bafd6650560`
(PR #59); the current code under `hypervisor/`, `scripts/`, and host tests
last changed in the merged W11 implementation. The default and NC2–NC5
images rebuilt here have the **same SHA-256 values** as W11's integrated
verification, so no code-delta inference is needed. Host: Linux WSL2
`6.18.33.2-microsoft-standard-WSL2` x86_64; QEMU AArch64 8.2.2; pinned
Rust 1.98.1; one `virt` boot CPU, 128 MiB and the documented W10 profiles.
The official `scripts/qemu-runner` entry owns every QEMU launch. The original
raw capture root was the ignored build-tree path
`.worktrees/p1completion/target/p1-completion-evidence/`. Its contents are
now preserved in the [local evidence archive](p1-local-evidence-archive.md),
with the original member paths intact. The archive is **not** a durable CI
artifact or remote backup.

| Run | Image SHA-256 | Result and retained evidence |
|---|---|---|
| Normal 100-cycle regression | `f9895fa0e365631160fa6b38edf5583b8de77b1609187995c672ad44ce79e152` | `requested=100`, `counted=100`, all `PASS`; each serial has exactly one Stable and no panic/fatal/reject marker. `r100/summary.txt`, `r100/cycle-*/{invocation,outcome,serial,emulator}.*`. |
| NC1 EL2 unavailable | same default image | 2/2 paired verdicts passed; one explicit `reason=EL` rejection per run, no normal continuation; `nc1/summary.json` and `run-{1,2}/` captures. |
| NC2 required-capability sample | `8791edf9d2040fbdea826c77d257a915e0e0d4f137f87bdcf329165b7d2ab5e7` | 2/2 passed; required 4-KiB-granule sample rejection in `capabilities.enter`; **sample substitution**, not observed absent CPU hardware. |
| NC3 synchronous undefined instruction | `78f094be85fdfb40499573815069e59a89f6176369340cf9346b78a9710129dd` | 2/2 passed; ESR/PC/phase and one bounded terminal report. |
| NC4 intentional panic | `58caf44e10cda431db48a40f945832c94780f1ec85182541ed5e2e295fe12620` | 2/2 passed; panic identity/location/phase and one bounded terminal report. |
| NC5 post-MMU translation fault | `62349bf7195e3a0db6dc05ba1f25fc1cd9d21104ff4d4f967c8587bbf1774ad0` | 2/2 passed; ESR.EC `0x25`, FAR `0x5000_0000`, `stage1.complete`, relevant frame and one bounded terminal report. |

The five `nc*/summary.json` files each say `passed=true`, contain two
concordant runs and verify image hashes against invocation records. A
separate read-only audit checked all 100 `outcome.json` and `serial.log`
files: 100 status-0 outcomes, exactly one Stable marker per cycle, and no
panic/fatal/reject marker. The repeated reference image also corroborates
W10's earlier accepted [R1–R6 and 100-cycle record](p1-w10-qemu-boot-regression-verification.md),
including NC4's final R2 panic control.

Fresh local quality checks passed: `cargo fmt --all -- --check`; host
`cargo test --workspace --exclude hypervisor` (34 tests, none failed or
ignored); host and AArch64-target Clippy with `-D warnings`; the AArch64
default and NC2–NC5 builds; and 24 Python runner/verdict tests. These do
not substitute for the QEMU evidence above. No target unit-test harness,
real board, hardware fault injection, Guest, or P6 asynchronous event ran.

## P1-V01–P1-V21 review

“Verified” below means the *task-book condition in the declared P1 scope*,
not an unbounded hardware or platform claim. Primary L6 records are linked;
the [evidence map](../contracts/stage-gate-evidence-map.md) has finer limits.

| Gate | Decision and primary evidence |
|---|---|
| P1-V01 | **Verified:** canonical image/entry contract in [W01](p1-w01-reference-boot-contract-verification.md); normal reference boot in [W10](p1-w10-qemu-boot-regression-verification.md) and fresh 100-cycle run. |
| P1-V02 | **Verified:** identical default image rejected 2/2 without EL2 in [W11 NC1](p1-w11-negative-fault-validation-verification.md) and fresh NC1 pair. |
| P1-V03 | **Verified:** stack/BSS/context/panic/identity ordering reviewed in [W02](p1-w02-minimal-rust-el2-runtime-verification.md), then integrated ordered entry-to-Stable path in [W09](p1-w09-initialization-sequencing-verification.md). |
| P1-V04 | **Verified:** hidden-register-dependency source review in [W02](p1-w02-minimal-rust-el2-runtime-verification.md), integrated 100/100 Stable run in [W10](p1-w10-qemu-boot-regression-verification.md) and fresh rerun. This does not vary firmware. |
| P1-V05 | **Verified:** bounded capability classification/host tests in [W03](p1-w03-aarch64-capability-inventory-verification.md), one integrated boot-CPU report in [W09](p1-w09-initialization-sequencing-verification.md). |
| P1-V06 | **Verified:** required/optional policy tests in [W03](p1-w03-aarch64-capability-inventory-verification.md), NC2 sample rejection 2/2 in [W11](p1-w11-negative-fault-validation-verification.md) and fresh pair. Hardware absence is not claimed. |
| P1-V07 | **Verified:** owned controls, masks, guards, readback and order reviewed in [W04](p1-w04-el2-architectural-state-baseline-verification.md), with integrated baseline boot in [W09](p1-w09-initialization-sequencing-verification.md). No per-cycle register dump is claimed. |
| P1-V08 | **Verified structurally:** all 16 entry slots and bounded classification/frame contract reviewed in [W05](p1-w05-el2-exception-entry-baseline-verification.md); executed asynchronous delivery remains P6-V29. |
| P1-V09 | **Verified:** NC3/NC5 paired synchronous ESR/location/terminal evidence in [W11](p1-w11-negative-fault-validation-verification.md) and fresh pairs. |
| P1-V10 | **Verified:** early replay/live markers in [W06](p1-w06-early-console-logging-verification.md) and [W09](p1-w09-initialization-sequencing-verification.md); fresh normal and NC2–NC5 phase-attributed captures. Pre-vector output remains limited. |
| P1-V11 | **Verified for applicable fatal classes:** bounded report fields in [W07](p1-w07-fatal-crash-diagnostics-verification.md), paired NC2–NC5 field checks in [W11](p1-w11-negative-fault-validation-verification.md) and fresh pairs. Panic has N/A syndrome rather than fabricated ESR. |
| P1-V12 | **Verified within exercised paths:** non-recursive guard review in [W07](p1-w07-fatal-crash-diagnostics-verification.md), paired single-END/no-continuation NC2–NC5 in [W11](p1-w11-negative-fault-validation-verification.md) and fresh pairs. Recursive physical-fault stress was not run. |
| P1-V13 | **Verified:** closed W08 map classes/permissions and linked placement in [W08](p1-w08-mmu-activation-verification.md); no-W+X audit in [W11 S2](p1-w11-negative-fault-validation-verification.md). Identity VA=PA remains temporary. |
| P1-V14 | **Verified:** mapped boot in [W08](p1-w08-mmu-activation-verification.md)/[W09](p1-w09-initialization-sequencing-verification.md), post-MMU NC5 ESR/FAR/phase and diagnostic route in [W11](p1-w11-negative-fault-validation-verification.md) and fresh pair. |
| P1-V15 | **Verified:** state-machine/order/failure-route source and host review plus ordered boot in [W09](p1-w09-initialization-sequencing-verification.md); fresh normal/negative captures corroborate Stable and phase attribution. |
| P1-V16 | **Verified:** R1–R6 controls, including final NC4-based R2, in [W10](p1-w10-qemu-boot-regression-verification.md) and [W11](p1-w11-negative-fault-validation-verification.md); fresh runner/verdict tests pass. |
| P1-V17 | **Verified:** [W10](p1-w10-qemu-boot-regression-verification.md) accepted 100/100 and a fresh 100/100 same-image reference run with preserved captures. |
| P1-V18 | **Verified under ADR-061:** NC3, NC4 and NC5 paired fault diagnostics in [W11](p1-w11-negative-fault-validation-verification.md) and fresh pairs. NC6 is not part of this revised P1 gate and remains open at P6-V29. |
| P1-V19 | **Verified for reviewed baseline:** [W11 S1–S6](p1-w11-negative-fault-validation-verification.md) covers input/range, no-RWX, unsafe inventory U-016/U-017, stage boundary, fault-path controls and default trigger containment. Same image hashes and unchanged code make that review applicable here. |
| P1-V20 | **Verified:** eight-contract consistency and P2 consumability in [W12](p1-w12-p1-documentation-handoff-verification.md), updated under ADR-061; this completion review checks the amended handoff and limitations. |
| P1-V21 | **Verified:** [W12](p1-w12-p1-documentation-handoff-verification.md) plan/graph/link/evidence-map review, current 12-plan and 21-ID enumeration, and this L7 no-overclaim review. |

## Seven exit conditions and required L7 questions

| Task-book §7 condition | Decision |
|---|---|
| 1. Stable Non-secure EL2 Rust entry | Met by V01/V03/V04/V15 and fresh 100/100 reference boots. |
| 2. Capabilities and EL2 baseline | Met by V05–V07; NC2 is a controlled required-fact sample, not a physically missing feature. |
| 3. Vectors and diagnostics before/after MMU | Met by V08–V12/V14/V18 within the installed-vector and exercised synchronous/fatal scope; asynchronous delivery belongs to P6. |
| 4. Mapping classes and temporary identity | Met by V13/V14/V19; no permanent identity ABI is offered. |
| 5. Automated verdict and 100 clean boots | Met by V16/V17; two recorded 100/100 sets and bounded R1–R6 controls. |
| 6. Negative/fault paths | Met by NC1–NC5 and structural vector coverage, each assigned to its revised gate; NC6 remains P6-V29. |
| 7. Contracts, reporting and handoff | Met by V19–V21 and linked contract set; P2 retains discovery/allocation ownership. |

Entry assumptions eliminated by code/review: CurrentEL/EL2 and nonzero DTB
pointer are checked before Rust transfer, required capability policy is
fail-fast, and W04 explicitly establishes its owned EL2 controls. Remaining
reference assumptions: QEMU `virt`/PL011, one CPU, fixed image/table/stack
placement, MMU/cache-off entry, opaque DTB content, firmware WFI behavior,
and one supported CPU model; see [known limitations](../contracts/known-limitations.md).
Fatal routes preserve build/CPU/EL and phase; synchronous exceptions also
preserve relevant syndrome, PC and FAR validity, while panic/phase failures
label unavailable fields rather than inventing them. The pre-vector window
has no guaranteed hypervisor-owned route. Host Stage-1's VA=PA window is a
bring-up choice, **not** an ABI. [W11 S4](p1-w11-negative-fault-validation-verification.md)
and the unchanged code since W11 confirm that Guest, SMP, GIC, platform
discovery, allocator and board-runtime mechanisms did not enter P1.

## Open items and handoff

| Open item | Owner / consumer | Effect on this decision |
|---|---|---|
| Genuine unexpected asynchronous EL2 vector (historical NC6) has no executed proof | P6-W12 / P6-V29 under ADR-061 | Not a P1-v0.2 gate; **blocks P6-V29 until real evidence**, and P1 completion cannot be cited as a pass. |
| DTB content, physical RAM/reservations, dynamic allocation | P2, via [P2 handoff](../contracts/p2-handoff.md) | Not implemented or verified by P1. |
| Real hardware, alternate firmware/CPU, hardware-fault and recursive-fault stress | Later platform/robustness validation | Not part of this reference-QEMU completion claim. |
| Raw QEMU captures are local ignored build artifacts | Local evidence custodian | [Archive](p1-local-evidence-archive.md) preserves the original captures outside removed worktrees; maintain an off-host copy if durability is required. CI required checks alone do not recreate them. |

P2's consumable package is this report, [task book v0.2](../task-book-v0.2.md),
the [eight P1 contracts](../contracts/README.md), the
[stage evidence map](../contracts/stage-gate-evidence-map.md), and the
W01–W12 implementation/verification records linked above. The package
provides a stable reference EL2 boot CPU, diagnostics and Host Stage-1
runtime—not a discovered platform, memory allocator, Guest or IRQ service.
