# P1-W11 negative/fault validation verification record

**Status:** NC1–NC5 paired execution and S1–S6 review passed locally after W10 integration; NC6 blocks P1-V18 and stage completion. Earlier sections are dated snapshots.
**Date:** 2026-09-26 (Asia/Shanghai; earlier snapshots dated below).
**Scope:** [W11 implementation record](../implementation/p1-w11-negative-fault-validation-record.md)
on the W10-integrated `5319995` baseline; earlier `4fd5700`/`249e1b3`
sections are dated historical snapshots.
**Version:** v0.1.
**Owner/change context:** P1-W11 validation-image foundation.
**Supersedes:** None.

## Executed foundation checks

- `cargo fmt --all` — passed.
- `cargo test --workspace --exclude hypervisor` — passed: 34 test executions,
  including the two new NC2 source-shared assertions. Five W03 source tests
  also execute a second time in the NC2 integration-test crate; these are not
  five additional distinct product cases (29 distinct test cases).
- `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings`
  — passed.
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor` — passed
  for the default selection.
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor --features p1-w11-nc2`
  — passed for the NC2 selection.
- `cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor --features p1-w11-nc2 -- -D warnings`
  — passed.
- `python3 scripts/test_p1_runner.py` — passed: 16 W10 runner foundation
  tests. This does not execute any NC scenario or validate a W11 profile.
- `git diff --check` — passed.

These checks prove source compilation and pure substitution/policy behavior;
they do not prove that W09 invokes W03, that the selected image faults, or
that the default linked ELF excludes every future trigger. The W03 entry is
currently dormant, so link-time garbage collection can obscure selected-code
presence in the final ELF until W09 integration.

## Scenario execution status (W11-DV03/DV04)

| Scenario | Run 1 | Run 2 | Current reason |
|---|---|---|---|
| NC1 unsupported execution | Not run | Not run | W11 scenario verdict/profile not integrated; W10 boot-smoke requires Stable |
| NC2 missing required fact | Not run | Not run | W09 phase call and W11 scenario verdict/profile not integrated; source-shared policy test is not execution |
| NC3 synchronous fault | Not run | Not run | W09 insertion hook and W11 scenario verdict/profile not integrated |
| NC4 panic | Not run | Not run | W09 insertion hook and W11 scenario verdict/profile not integrated |
| NC5 post-MMU fault | Not run | Not run | W09 MMU-on continuation and W11 scenario verdict/profile not integrated |
| NC6 unexpected vector | Not run | Not run | W09 Stable hook and W11 scenario verdict/profile not integrated |

No QEMU fault capture, paired-run concordance, terminal-marker observation,
or hardware fault evidence was collected. NC2 environment-only coverage is
unavailable on reference QEMU 8.2.2; the compile-time variant must be
identified as injected policy evidence when it runs. W10 R2 has not been
anchored to NC4.

## Scope/security review status (W11-DV05)

The S1–S6 review must be performed against the final integrated tree and
record its exact commit/image identity, evidence pointers, outcomes, and
owners for findings. It is **not run** on this foundation: S1 input/range,
S2 no-RWX, S3 unsafe inventory, S4 stage boundary, S5 no fault-path
relaxation, S6 linked default-image containment. The source-level default-off
feature selection and no-new-`unsafe` diff are partial S6/S3 facts, not a
passing S1–S6 review or P1-V19 evidence.

P1-V18 remains open because none of NC1–NC6 has paired real executions.
P1-V19 remains open because S1–S6 were not completed. W11-DV01 matrix
coverage is documented in the detailed design; W11-DV02 full binary
containment and W11-DV06 downstream handoff remain pending.

## Later W09 integration observation (2026-09-25)

The scenario table above is the standalone NC2 foundation's PR-time state.
The [W09 verification](p1-w09-initialization-sequencing-verification.md)
records one selected NC2 image run through the clean-boot runner: it returned
`FAIL-PANIC`, while the serial report identified `capabilities.enter` and
`cap-reject fact=granule-4k`. This is supporting evidence that W09 invokes
the unchanged W03 required-fact route; the clean-boot runner's failure is not
a W11 scenario-specific pass. A second NC2 run, paired comparison and the
NC1/NC3–NC6 fault scenarios remain unperformed; P1-V18/P1-V19 stay open.

## Integrated W11 execution, branch `p1/w11-fault-execution` (2026-09-25)

The sections above are the historical NC2-foundation snapshot. This section
records the later unmerged W11 branch over main commit `249e1b3`, with a
working-tree diff; it must be rerun after W10's runner changes and before
closure. Host: Linux x86_64, QEMU AArch64 8.2.2, pinned Rust toolchain.
The W11 verdict layer called the one W10 `scripts/qemu-runner` entry twice
per NC2–NC5 image (8 bounded QEMU runs total), each with timeout 8 seconds.
Runner status 4 means its *clean-boot* oracle detected the intentional
target failure; W11's independent exact-field checks returned status 0 for
each pair. Four runner artifacts per run and W11 `summary.json` are retained
under the build-tree paths below; they are local, not committed artifacts.
The final v4 verdict additionally checked W07's build identity, CPU/EL,
PC/SPSR and all 31 GPRs plus SP for exception reports, panic location and
handler-entry SP/LR for panic reports, the syndrome/FAR validity fields, and
the local image bytes' SHA-256 against both runner invocation records, and
rejected any P1 marker after the single report-end marker. V4 was rerun on
2026-09-26 after that stricter terminal check; the original v3 captures remain
local exploratory history.

| Scenario | Image SHA-256 | Run 1 / Run 2 | Exact observed class and route | Local evidence root |
|---|---|---|---|---|
| NC2 injected required fact | `8791edf9d2040fbdea826c77d257a915e0e0d4f137f87bdcf329165b7d2ab5e7` | passed / passed | `kind=P`, `ph=capabilities.enter`, `cap-reject fact=granule-4k`, one END, no Stable | `target/p1-w11-nc2-paired-v4/` |
| NC3 undefined instruction | `78f094be85fdfb40499573815069e59a89f6176369340cf9346b78a9710129dd` | passed / passed | `kind=E`, sync/fatal-syndrome, ESR `0x02000000` (EC 0), W05 `cls=unknown`, FAR unavailable, `ph=fatal-path.complete`, one END | `target/p1-w11-nc3-paired-v4/` |
| NC4 intentional panic | `58caf44e10cda431db48a40f945832c94780f1ec85182541ed5e2e295fe12620` | passed / passed | `kind=P`, exact static message, `ph=fatal-path.complete`, one END | `target/p1-w11-nc4-paired-v4/` |
| NC5 post-MMU translation | `62349bf7195e3a0db6dc05ba1f25fc1cd9d21104ff4d4f967c8587bbf1774ad0` | passed / passed | `kind=E`, sync/fatal-syndrome, ESR `0x96000006` (EC 0x25), `cls=data-abort-translation`, FAR `0x50000000`, `ph=stage1.complete`, one END | `target/p1-w11-nc5-paired-v4/` |

Each pair used the same image SHA-256, runner status/exit 4, one report-end
marker and no following phase/Stable marker. NC2 is only controlled sampled
fact substitution, **not** a CPU-hardware-absence observation. For NC5,
W08's verified three-level map gives image L1 index 1/L2 index 0 while the
fault VA has L1 index 1/L2 index 128 (invalid); the console lies in L1
index 0. The observed FAR and translation syndrome support that exact
unmapped target. These observations do not prove real-hardware behavior.

The default feature build's linked ELF had no `validation_fault` or
`fault_scenario` symbol and no NC3/NC4/NC5 trigger string (`llvm-nm` and
`strings` filtered checks returned no match). Its derived image SHA-256 was
`f9895fa0e365631160fa6b38edf5583b8de77b1609187995c672ad44ce79e152`;
one W10 clean-boot cycle returned `PASS`, retained at
`target/p1-w11-default-smoke/`. This is interim S6 containment evidence
on this branch, not the final integrated S1–S6 review or a 100-cycle claim.
The NC3/NC4/NC5 feature combinations are rejected by `compile_error!`;
one NC3+NC5 build was checked to fail with that exact diagnostic.

An **exploratory manual** 5-second QEMU boot with the same default image and
`virt,virtualization=off` printed `ZELYR P1 BOOT REJECT reason=EL` and
timed out in W01's bounded stop. This is not W10-runner evidence: its current
profile fixes virtualization on. Formal NC1 paired evidence is blocked until
the same runner gains an approved EL2-disabled profile. NC6 is blocked by
the lack of a genuine deterministic unexpected IRQ/FIQ/SError trigger in
P1's masked no-GIC scope. A manual QEMU monitor `nmi` probe on `virt` returned
`machine does not provide NMIs` (diagnostic reported by the integration
reviewer), consistent with [QEMU's machine-specific NMI monitor
contract](https://www.qemu.org/docs/master/system/monitor/); this does not
prove every alternative event source impossible. NC6 is not replaced by a
synchronous instruction or a direct branch into vector code.

## Integrated quality and scope/security status

`cargo fmt --all -- --check`, host Clippy with `-D warnings`, 34 host-test
executions, target default/NC3/NC4/NC5 builds and Clippy with `-D warnings`,
three pure W11 verdict-parser tests, and `git diff --check` passed after the
changes; NC2 feature compilation had passed earlier and the NC2 image above
was built on this branch. The first format check found only import ordering;
`cargo fmt --all` corrected it before the passing check. The first NC3/NC4
Clippy check found dead W08 code in a build that terminates before Stage1;
the module is now excluded only from those scenario builds and the checks
pass. The W11 script is a target-specific verdict layer, not another QEMU
entry or a W10 runner modification.

S1–S6 final review is **not run** on the final integrated baseline. Interim
checks: S2 host model tests assert no W+X; S3 U-016/U-017 received independent
static review; S4/S5 W11 diff adds no Guest/SMP/GIC/recovery or permission
relaxation; S6 default ELF has no linked trigger symbol/string and one normal
boot passed. S1 full input/range audit, S2 final linked map audit, S3 full
tree-to-inventory bidirectional audit, S4/S5 final-tree audit, and S6 after
W10 merge must be recorded per item before P1-V19 can pass. P1-V18 remains
open because NC1 formal runs and NC6 genuine unexpected-event evidence are
missing. No QEMU hardware fault or real-board test was run.

## W10-integrated acceptance review (2026-09-26)

This section supersedes the preceding **historical** NC1/S1–S6 pending
statements. Source baseline: W10 merge `5319995`, W11 local branch rebased at
`88ec142` plus the NC1-profile/documentation diff in this branch. Build and
scenario artifacts are under this worktree's `target/`, not committed or a
durable CI store. Host: Linux x86_64, QEMU AArch64 8.2.2; five target images
were freshly built after the rebase with the pinned Rust toolchain. The
default image SHA-256 is
`f9895fa0e365631160fa6b38edf5583b8de77b1609187995c672ad44ce79e152`.
NC1 used those identical bytes under the fixed `p1-no-el2` profile; NC2–NC5
used `p1-boot-smoke`. Every run used the one `scripts/qemu-runner` entry,
8-second timeout, complete serial/emulator captures, invocation and outcome
records, and a W11 pair summary.

| Scenario | Image SHA-256 | Pair result and diagnostic | Retained evidence |
|---|---|---|---|
| NC1 EL2 unavailable | default hash above | 2/2 passed; exact `BOOT REJECT reason=EL`, no Runtime/Stable/fault marker, runner 0 under rejection-specific oracle | `target/p1-w11-nc1-post-w10/` |
| NC2 injected required fact | `8791edf9d2040fbdea826c77d257a915e0e0d4f137f87bdcf329165b7d2ab5e7` | 2/2 passed; `cap-reject fact=granule-4k`, `ph=capabilities.enter`, terminal panic | `target/p1-w11-nc2-post-w10/` |
| NC3 undefined instruction | `78f094be85fdfb40499573815069e59a89f6176369340cf9346b78a9710129dd` | 2/2 passed; sync fatal, ESR.EC 0, `ph=fatal-path.complete`, terminal report | `target/p1-w11-nc3-post-w10/` |
| NC4 intentional panic | `58caf44e10cda431db48a40f945832c94780f1ec85182541ed5e2e295fe12620` | 2/2 passed; exact static message, `ph=fatal-path.complete`, terminal report | `target/p1-w11-nc4-post-w10/` |
| NC5 post-MMU translation | `62349bf7195e3a0db6dc05ba1f25fc1cd9d21104ff4d4f967c8587bbf1774ad0` | 2/2 passed; data-abort translation, ESR.EC `0x25`, FAR `0x5000_0000`, `ph=stage1.complete`, terminal report | `target/p1-w11-nc5-post-w10/` |
| NC6 genuine unexpected vector | none | **Blocked**; no approved deterministic IRQ/FIQ/SError event source with P1's DAIF mask and no GIC/IRQ scope | none |

Each NC1–NC5 `summary.json` reports `passed=true`, checks the runner exit
against its outcome record and the local image bytes against both invocation
hashes. NC2–NC5 additionally check W07's build identity, CPU/EL, phase,
syndrome and validity, panic location/entry SP/LR or exception PC/SPSR/all
31 GPRs/SP, one report-end marker, and no later P1 marker. NC1's W01
pre-runtime rejection has no W07 report fields or END marker by design;
instead its full serial has exactly one EL rejection line and no other P1
line. NC2's substituted sample is **not** real CPU feature absence. No run
proves real-board, other CPU-model, or hardware fault behavior.

The real NC4 image separately ran through the W10 regression driver as
R2: `target/p1-w11-r2-nc4-post-w10/` recorded `requested=1`, `counted=1`,
`FAIL-PANIC`, runner status 4/`forbidden-marker` and expected driver exit 1.
This replaces W10's provisional NC2 panic-control anchor for P1-V16.
W10's accepted 100-cycle set remains the evidence for its own named normal
image; the default boot profile's QEMU command and marker oracle are
unchanged by the NC1 addition, so that set was not rerun or generalized to
this image.

### S1–S6 scope/security review (W11-DV05 → P1-V19)

Review baseline is the W10-integrated branch/source and freshly linked
default ELF identified above. Outcomes are **source/linked-image review**,
not hardware testing; any material source change requires re-review.

| Item | Outcome | Evidence and boundary |
|---|---|---|
| S1 input/range | Passed, scope-qualified | `hypervisor/src/boot/mod.rs` checks CurrentEL before nonzero `x0`; `context.rs` converts `x0` only to an opaque `PhysAddr` and retains `x1`–`x3` uninterpreted. A source-wide consumer search found no P1 DTB dereference, address arithmetic, range use, or later `BootContext::dtb`/`reserved` consumer; `context.rs` itself only rechecks presence. W03 `capabilities/mod.rs`/`facts.rs` decode bounded register fields and apply Required policy before publication/consumption. Thus no P1 range/alignment-sensitive operation uses the unvalidated DTB pointer. W01 deliberately does **not** validate DTB alignment/content/range or memory adequacy; these remain P2/residual assumptions, not claims established by this review. |
| S2 no unintended RWX | Passed | `stage1/model.rs` has six closed mapping classes: Code/Vectors RX, RoData RO+XN, Data/Stack RW+XN, Console Device RW+XN, with AP[1] RES1; `Tables::verify` checks descriptor/inventory closure and invalid holes. Host `mappings_never_combine_write_and_execute` passed. Linked default ELF sections `.text.boot`, `.p1_vectors`, `.text` were AX; `.rodata` A; `.data`, `.bss`, `.p1_boot_stack` WA; none W+X. This is P1's static image map, not future dynamic mappings. |
| S3 unsafe inventory | Passed | Read-only `rg` walk of `hypervisor/src` unsafe blocks/functions and `docs/security/unsafe-inventory.md` found existing U-001–U-015 boundaries unchanged by W11; the two new isolated asm blocks are source-commented U-016/U-017, inventoried in the same change, and independently statically accepted by `/root/w11_unsafe_review`. No new `static mut`, `transmute`, Core unsafe, or uncategorized W11 segment. Inventory IDs are not required to be contiguous. |
| S4 stage boundary | Passed | Diff from `main` touches only W11 validation feature gates, W09 call hooks, AArch64 fault instruction module, source-shared host test, the existing runner's fixed NC1 profile and verdict/tests/docs. No Guest, SMP/PSCI, GIC/IRQ, allocator, DTB discovery, virtio, Control Domain, board-name branch or production recovery mechanism was added. |
| S5 fault-path controls | Passed | W11's `fault_scenario` calls are after `FatalPath.complete` or `Stage1.complete`; they emit one fixed instruction or static panic and never return normally. W09 `fail_phase` remains terminal; W11 changes no PTE permission, DAIF, trap register, fatal guard or recovery route. NC5's observed terminal report followed an unmapped load, not a control relaxation. |
| S6 default containment | Passed | Default-off feature gates in `Cargo.toml`, `main.rs`, `arch/aarch64/mod.rs` and `lifecycle.rs`; default linked ELF `llvm-nm` had no `validation_fault`/`fault_scenario` symbol and `strings` had no NC3–NC5 trigger string. `target/p1-w11-default-post-w10/` ran one ordinary regression cycle `PASS` with the default image; scenario feature combinations are compile-time rejected. This is one normal boot plus source/link inspection, not another 100-cycle claim. |

`cargo fmt --all -- --check`, 34 host Rust test executions, host Clippy
`-D warnings`, 24 Python script tests, five target builds and five target
Clippy selections (`default`, NC2–NC5), and `git diff --check` passed on this
branch. The host source-shared test now uses a reasoned
`#[expect(unexpected_cfgs)]` rather than an unrestricted allow. No target unit
test harness, real-board run, Guest test, CI QEMU gate or hardware fault
injection ran. S1–S6 support **P1-V19 locally** for this audited baseline.
**P1-V18 remains blocked by NC6**, so W11 and P1 are not declared complete.
