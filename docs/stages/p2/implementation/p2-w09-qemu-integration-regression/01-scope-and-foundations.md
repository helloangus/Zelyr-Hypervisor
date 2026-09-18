# P2-W09 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W09 detailed design](README.md).

## 1. Package outcome

One executed integration matrix over QEMU `virt` whose evidence record
shows, for every required configuration and the required boot repetitions,
that the P2 chain (intake → discovery → map → page allocator → heap →
inspection) boots, produces the expected platform facts, keeps map
accounting inside its documented domain, passes the W06 consistency
checks, and is stable across repeated boots — or an evidence record that
says precisely which runs did not happen and why. P2-V11 is satisfied only
by the former, in full.

## 2. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W09 relies on | Failure boundary if delivered differently |
|---|---|---|---|---|
| A1 | One documented QEMU runner entry with parameter carrying (machine/CPU/RAM), serial capture, timeout, exit status, and evidence conventions | P0-W09 (planned) | All execution | If the entry cannot express a matrix parameter (e.g. RAM size), record blocked upstream defect; no parallel runner ([README](README.md) Decision 1) |
| A2 | A bootable EL2 image: P1 entry, early console, DTB handoff (A1 pair), image range (A3), host access window (A2) | P1-W01/W06/W08/W09/W10 (planned) | The boots themselves | Missing or unstable P1 state: blocked; W09 must not patch P1 or improvise a boot path |
| A3 | W01–W06 chain functioning per their contracts, emitting the boot diagnostic points incl. the inspection render | P2 designs W01–W06 | All expected observations | A failed expectation is evidence against the owning package's verification record; W09 never patches product code to make a boot pass |
| A4 | Host-side negative/property baseline already green | W08 evidence | Interpretation context: integration failures are not retried as host failures | If W08 rows are failing/blocked, W09 runs are still possible but their evidence must reference the open host rows; W10 records the state |
| A5 | QEMU host availability and version pinning convention | P0-W09 evidence conventions | Environment rows in evidence | Version drift mid-matrix invalidates comparability: pin the version in the environment row; a new version is a new dated run section |

## 3. QEMU-vs-hardware proof boundary (binding)

What QEMU integration evidence proves: the implemented P2 chain works on
the reference platform for the exercised configurations; the facts, map
accounting, allocation, and inspection are stable and mutually consistent
there; the boot path tolerates the configuration variations in the matrix.

What it does not prove:

- real-hardware behavior — cache/TLB semantics, DMA, IRQ timing, firmware
  variety, power/errata behavior (ADR-003's reason for having both
  platforms; P15 is where hardware evidence starts);
- that omitted hardware rules (barriers, cache maintenance, TLB
  invalidation discipline) are acceptable — QEMU success is never
  evidence for skipping them (Coding Guidelines);
- SMP behavior (no AP starts in P2; `-smp` variations only change the
  *described* inventory);
- Orange Pi 3B or any board's support (offline fixture status only, W07);
- correctness beyond the exercised configurations and repetitions.

## 4. Observation instrument

The W06 inspection render is the sole observation surface for P2 facts in
boot logs (README Decision 5): sections `[platform]`, `[memory-map]`,
`[page-allocator]`, `[heap]` plus the boot-context section. The P1
console/diagnostic lines (intake marker, allocator-ready marker) are
secondary markers for chain ordering. Expectations reference section
fields, never ad-hoc prints; if evidence seems to require a new print,
that is a W06/W01 design conflict to raise.

## 5. Determinism expectation

For a fixed configuration and fixed image: identical expected rows across
boots — including allocator/heap totals and metadata placement. Bare-metal
boot has no randomization source in P2 (no time-of-day, no RNG use); drift
across boots is therefore a defect signal (uninitialized state), not noise
(README Decision 4). Any observed drift is a failed row with both renders
attached.
