# P1-W09 initialization sequencing verification

**Status:** Local integrated lifecycle checks passed; repetition, fault matrix and hardware remain separate work.
**Date/environment:** 2026-09-25 Asia/Shanghai; Linux x86_64 host, pinned Rust 1.98.1, QEMU AArch64 8.2.2, branch `p1/w09-initialization-sequencing` based on main `02bf587`.
**Implementation:** [W09 record](../implementation/p1-w09-initialization-sequencing-record.md).

| ID | Result | Evidence and boundary |
|---|---|---|
| W09-DV01 / P1-V15 | Passed | Source review of Entry/Runtime glue and straight-line Capabilities→Stage1 enter/body/complete pairs; three host tracker tests pass. Does not prove each mechanism's own correctness. |
| W09-DV02 / P1-V15 | Passed | Source review and one runner capture show Entry.enter…Console.enter replay in order, one live Console.complete, later live pairs and W10 Stable token. This supports marker coverage, not future fault output. |
| W09-DV03 / P1-V15 | Passed | Each adapter uses its owning package API; W03 rendering is the authorized console-phase producer. No Guest/SMP/GIC/DTB discovery/allocator call or partial-init fallback. |
| W09-DV04 / P1-V15 | Passed by code review; selected negative route observed | W03/W04/W05/console fail by early panic; W07 owns its readiness stop; Stage1 errors diverge through armed W07 with step-qualified static message; stable faults retain tracker position. One NC2 validation-image run reported `ph=capabilities.enter`; no full W11 matrix claim. |
| W09-DV05 / P1-V15 | Passed definition; 100-cycle execution deferred to W10 | Same ordered phase sequence, one Stable token, no forbidden report and same terminal state are measurable per clean boot; no 100/100 claim here. |
| W09-DV06 / P1-V15 | Passed local review | One boot writer, CAS position, wait-free readers, no allocation; U-015 WFI boundary accepted by independent review with explicit EL3 firmware limitation. Not SMP-safe by claim. |
| W09-DV07 | Passed handoff review | W10 token matches runner, W11 has phase attribution, W12/P2 receive the bounded stable environment. Downstream work is not declared complete. |

Quality gates on this branch: `cargo fmt --all -- --check`, host and target
Clippy with `-D warnings` (default and NC2 target selections),
`cargo test --workspace --exclude hypervisor` (34 test executions), default
and NC2-feature AArch64 target builds, `PYTHONDONTWRITEBYTECODE=1 python3
-m unittest discover -s scripts -p 'test_*.py'` (16 W10 runner mechanism
tests), and `git diff --check` passed. No target unit-test harness exists.
These Python tests are not the full W11 scenario suite.

The W10-owned single runner entry was used on a default image built from this
branch: `scripts/p1-image --elf target/aarch64-unknown-none-softfloat/debug/hypervisor
--output target/p1-w09-official.img`, then `scripts/p1-boot-regression
--cycles 1 --timeout 8 --image target/p1-w09-official.img --evidence
target/p1-w09-r1`. The image SHA-256 was
`759ec47cc7a9a28b1286cfaa7181d5a029aeda3ae261cf7afe1294f1874876c9`;
the runner returned `cycle=001 PASS` (exit 0). Its build-tree serial capture
shows the complete phase order, Stage1.complete and `ZELYR P1 STABLE`, with no
panic/fatal marker. This is one local R1-equivalent boot, not 100-cycle
regression evidence or a real-hardware claim. A separate pre-runner manual
5-second QEMU diagnostic showed the same sequence and expected idle timeout;
it is informative only.

For the W11 NC2 validation image, target feature build and conversion passed.
Image SHA-256 was
`d0fcbf473cecffd09ba680b12e3ecb54129fe4c165a7a7cfd586443c9d280e2c`.
One `scripts/p1-boot-regression --cycles 1 --timeout 8 --image
target/p1-w09-nc2.img --evidence target/p1-w09-nc2-r1` run returned
`FAIL-PANIC` (exit 1), the expected clean-boot-oracle classification for a
negative image. The retained serial report says
`ph=capabilities.enter` and `cap-reject fact=granule-4k`; no later phase
marker appears. W11 requires two scenario-specific runs and its own verdict,
so this observation is supporting integration evidence only.

After linking the default image, `llvm-readelf -sW` placed the W08 five-page
`STAGE1_TABLES` at `0x4009b000..0x400a0000` (20,480 bytes) inside the
page-separated DataRw region `0x40099000..0x400a1000`.
`RO_SENTINEL` at `0x40097a10` is inside RoData
`0x40095000..0x40099000`; `RW_SENTINEL` at `0x4009a2a0` is inside DataRw.
The dedicated 64 KiB stack remains `0x400a1000..0x400b1000`.
`llvm-objdump` shows a real `ldr x0, [x8]` in the volatile read helper, calls
to the AtomicU64 store/load for RW, and the closed `wfi` instruction in the
idle wrapper. The debug `enable_host_stage1` prologue reserves approximately
32 KiB plus nested frames; one QEMU success shows no observed overflow on
that path but is not a formal stack high-water proof.

P1-V15's local lifecycle review is supported. P1-V13/P1-V14 gain one executed
canonical MMU transition and linked-placement observation, but fault and
hardware coverage remain open. P1-V16/V17 ownership and the 100-cycle result
remain W10's; P1-V18/V19 remain W11's. The pre-vector unowned window and
firmware-controlled `SCR_EL3.TWI` WFI trap behavior are explicit limitations.
