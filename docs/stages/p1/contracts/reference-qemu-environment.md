# P1 reference QEMU environment

**Status:** Proposed assembly; R1–R6 and 100-cycle reference evidence recorded.\
**Scope:** Canonical runner conventions and evidence acceptance, not CI policy or performance.\
**Version:** v0.2.\
**Owner/change context:** P1-W12 assembly of [W01 selection](../implementation/p1-w01-reference-boot-contract-record.md), [W10 runner record](../implementation/p1-w10-qemu-boot-regression-record.md) and [runner design](../implementation/p1-w10-qemu-boot-regression/01-automation-contract.md), with artifact-custody update, 2026-09-26.\
**Supersedes:** v0.1 wording on raw-evidence location.

The [boot contract](aarch64-boot-contract.md) fixes `virt,virtualization=on`,
`cortex-a57`, one CPU, 128 MiB, serial-only output and the derived ARM64
Image. The only automated QEMU process owner is `scripts/qemu-runner`, which
implements the P0 runner-entry grammar. `scripts/p1-image` converts the
built ELF, while `scripts/p1-boot-regression` repeats the same runner entry;
neither creates a second boot recipe. Host tools and versions belong in each
run's metadata rather than a universal compatibility claim.

For a normal boot, `ZELYR P1 PHASE entry` and `ZELYR P1 PHASE runtime` give
start context; `ZELYR P1 STABLE` is the success token. Any `ZELYR P1 PANIC`,
`ZELYR P1 FATAL`, or `ZELYR P1 BOOT REJECT` class forbids PASS. The verdict
uses bounded token membership, timeout and process-exit classification; a
stable token alone cannot override timeout, forbidden output or abnormal
exit. [W10's scenario matrix](../implementation/p1-w10-qemu-boot-regression/02-scenarios-and-verdict.md)
owns R1 canonical, R2 panic detection, R3 timeout, R4 missing Stable, R5
invocation error and R6 order-independence. A failed or timed-out run retains
capture and a nonzero outcome.

Each cycle retains invocation, serial, emulator and outcome files; the driver
adds `meta.txt` and `summary.txt` in a fresh evidence directory. Emulator
stderr cannot forge serial markers. P1-V17 accepts only 100 consecutive
PASS cycles on the exact recorded image, same Stable token, no forbidden
class and no `FAIL-*`; stop at first failure rather than rerunning it away.
The [W10 verification record](../verification/p1-w10-qemu-boot-regression-verification.md)
reports canonical R1 `PASS`, R3 `FAIL-TIMEOUT`, R4 `FAIL-MARKER`, R5
`ERROR-INVOCATION`, and real-capture R6 order/noise permutation. The original
NC2 R2 control is historical; [W11's NC4
addendum](../verification/p1-w11-negative-fault-validation-verification.md)
ran the real intentional-panic image through the same regression entry and
observed `FAIL-PANIC`, runner status 4, and the expected nonzero driver exit.
This supplies the final R2 anchor for local P1-V16 review. The accepted
`r100-accepted` batch is
100/100 consecutive `PASS` on one recorded image/environment, with exactly
one Stable and no forbidden class per cycle; local raw evidence is retained
in the [artifact archive](../verification/p1-local-evidence-archive.md), not
in a durable CI store or remote backup. This supports P1-V17 for
that image; the NC4 re-anchor does not repeat or generalize that 100-cycle
run.
