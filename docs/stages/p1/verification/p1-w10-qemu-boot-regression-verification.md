# P1-W10 QEMU Boot Regression — Verification Record

**Status:** Foundation mechanism checks passed; integrated QEMU evidence not run.  
**Date:** 2026-09-25  
**Implementation:** [Record](../implementation/p1-w10-qemu-boot-regression-record.md).

## Foundation checks

Command: `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts -p 'test_*.py' -v`.
Result: 13 tests passed, no skipped tests. These are host tooling mechanism
checks using controlled Python child processes; they are neither the canonical
Cargo host-test gate nor evidence of EL2, QEMU, MMIO, cache/TLB or hardware.

Covered: observed completion, late fatal after stable, global timeout after
stable, unexpected/silent process exit, missing executable, full capture on
output-limit failure, stderr isolation, final drain on termination, invalid
parameter evidence, refusal to overwrite prior evidence, invalid timeout
domain, and fixed-token matching under all permutations and byte boundaries.
This supplies foundation portions of W10-DV03/DV04 and matcher review for
W10-DV01/DV06; it does not satisfy their integrated execution requirements.

## Outstanding execution

R1 canonical boot, R2 panic detection, R3 real QEMU timeout, R4 missing marker,
R5 invalid image invocation, R6 permutation of real capture and 100 consecutive
boots: **not run**, awaiting W09's integrated runtime. P1-V16 and P1-V17 are
not yet satisfied. No result here closes deferred W01/W02 execution evidence.
The hard-timeout default remains provisional until exploratory timing exists.
