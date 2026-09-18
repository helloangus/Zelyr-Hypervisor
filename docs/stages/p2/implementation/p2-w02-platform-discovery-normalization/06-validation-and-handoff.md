# P2-W02 Validation, Error Model, and Handoff Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W02 detailed design](README.md).

## 1. Scope of validation for this package

W02's own evidence is host-side over synthesized DTB fixtures (certified by
W01's validator, so the intake→discovery chain is exercised end to end on
host). Fixture DTBs representing the real QEMU `virt` and RK3566 outputs,
plus any QEMU observation, are W07/W09 evidence and appear here only as
not-run entries until their packages run.

## 2. Error, security, and observability model

- **Error model.** Exactly four fatal diagnostics
  ([01 §6](01-scope-and-foundations.md)) with all-or-nothing publication;
  everything else is a five-state fact. No path converts malformed *input*
  into an invariant panic (P0-W14 discipline): a hostile DTB yields
  recorded states or one fatal diagnostic, never a crash.
- **Security model.** Untrusted DT values are decoded with checked
  arithmetic and bounded copies; W02 never dereferences a discovered
  address; capabilities prevent false claims (`NotDiscovered` vs
  `Absent`); no board names (ADR-043/052) — enforced by a dedicated
  workflow step (Step 6). Residual risk: fact *content* reflecting a
  hostile platform is passed on honestly; containment is W03's conflict
  logic and the W10 handoff states that division.
- **Observability.** Discovery counters (skipped nodes/properties,
  anomalies, dropped zero-size entries) travel inside `PlatformInfo`; W06
  renders them; W09 boot logs get a single "platform facts normalized"
  marker with counts (CPUs, banks, reservations) — no DT content.

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W02-DV01 → P2-V03 | Pipeline assembly | Host integration test over fixture corpus | Intake + normalize on every fixture | Either complete `PlatformInfo` or exactly one expected fatal diagnostic | Chain coherence; not QEMU blob fidelity |
| W02-DV02 → P2-V03 (P2-B01) | CPU inventory | Host unit tests | Enabled/disabled/unknown, multi-cell reg, capacity, DT-order | Entries complete and ordered; defaults applied | Decoder + walker correctness; not real firmware CPU descriptions |
| W02-DV03 → P2-V03 (P2-B01) | Boot-CPU relation | Host unit tests | Match, mismatch, absent inventory | Match recorded; mismatch → `BootCpuUnmatched` fatal | Relation logic; not P3 topology policy |
| W02-DV04 → P2-V03 (P2-B02) | RAM banks | Host unit tests | 1-cell/2-cell cells, multi-bank, zero-size, malformed reg | Banks verbatim; zero-size dropped+counted; malformed → Unusable | Bank facts; not alignment/overlap judgment (W03) |
| W02-DV05 → P2-V03 (P2-B03) | Reservations | Host unit tests | rsvmap entries, /reserved-memory children with no-map/reusable, missing reg, capacity | Verbatim merged records with sources and flags | Reservation capture; not W03 protection |
| W02-DV06 → P2-V03 (P2-B04–B06) | GIC/timer/PSCI facts | Host unit tests | v3 complete, v2 only, absent, bad stride/cells, PSCI 0.2/1.0/0.1/no-method, timer variants | Every state class produced by its fixture; states never collapse | Fact fidelity; not device usability in later stages |
| W02-DV07 → P2-V03 (P2-B07) | Chosen facts | Host unit tests | stdout-path direct/aliased/with options/unresolvable, bootargs, initrd pair, inverted initrd | Facts recorded per README Decision 6; artifact flagged when malformed | Chosen capture; not console behavior |
| W02-DV08 → P2-V04 (P2-B08/C02) | State distinguishability | Host property test + type review | Randomized fact outcomes; attempt to construct collapsed states | Five states always distinguishable; `NotDiscovered` only in designated capabilities | Capability honesty; not downstream stage decisions |
| W02-DV09 → P2-V04 (P2-C01) | Normalization/no-board-branch | Design review + mechanical search | Search for board/platform strings; check consumers get facts only | Zero hits; `PlatformInfo` is the only surface | Layering conformance; not future consumer behavior |
| W02-DV10 → P2-V04/P2-V10 (P2-C03) | Determinism | Host property test | Run normalize twice per fixture; compare records byte-wise | Identical outputs | Repeatability; not hardware-idempotence |

Evidence statuses are passed / failed / blocked / not run with command,
input, environment, timestamp. Host validation does not prove QEMU boot
integration, real-firmware DTB handling, or any later-stage mechanism.

## 4. Handoff checklist

Before handing W02 to review, provide:

- changed-module list; confirmation of zero allocation, zero `unsafe`
  (beyond borrowing W01's handle types), zero board/platform names;
- W02-DV01–DV10 evidence statuses, with not-run entries for QEMU
  integration (W09), real fixtures (W07), and P3/P4 consumption (W10);
- confirmed consumer readiness: W03 (banks/reservations/artifacts records),
  W06 (fact types + counters), W07 (host-runnable walkers), W08/W09
  (determinism and stability criteria), W10 (semantic contract inputs);
- open items recorded, not resolved: physical module placement (P0-W03),
  P0/P1 assumed contracts, fixture DTBs for W07, PSCI 0.1 and >2-cell
  policies left Reserved;
- explicit note that P2-ACR-01 does not arise in W02 (no memory objects
  designed) and that the reservation records are inputs to W03's
  protection logic, not claims.
