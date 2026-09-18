# P3-W11 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W11 detailed design](README.md).

## 1. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W11-DV01 → P3-V11 | catalog governance review | inspect the counter catalog and event inventory against the P0-W13 namespace and P0-W12 channel/trim contracts as delivered | every plan-scope area (lifecycle, notification, transport, contention, boot sync + timing) has at least one counter and/or event; ids/names/domains registered per governance; trim and channel classes present per row | the observable surface is declared and governed; not that the observed mechanisms work |
| W11-DV02 → P3-V11 | counter semantics tests (host-side) | fake per-CPU block + simulated exception reentrancy + wrap/boundary unit tests | exact totals under reentrancy; fetch-max high-water correct; capacity fit; documented wrap rule observed; no locks/allocations in any tested path | counter primitives are sound in isolation; not that real upstream seams call them |
| W11-DV03 → P3-V11 | attribution and no-board-leakage review | inspect every catalog row and rendering path for the identity triple and typed target/source; search for platform/board/QEMU identifiers | no event or dump row without attribution; directed events carry typed targets; no platform-name content anywhere (ADR-043/ADR-052) | meaningful CPU attribution by construction; not boot-level capture quality (DV05) |
| W11-DV04 → P3-V11 | seam acceptance review | read the seam register against W06/W07/W08/W03/W02/W05 designs and implementation records | each §6 register row is accepted by its owning design or recorded as a named gap with coverage consequence | the seams are agreed; accepted seams do not prove the mechanisms function |
| W11-DV05 → P3-V11 | capture review under the declared test environment | boot in the declared environment (minimum declared CPU count), capture SMP-ready counter dump and event presence; render the fatal-path dump if W09 consumes it | dump renders per-CPU rows with correct attribution; lifecycle/rendezvous events present per boot; `boot.timing.raw` present but unconsumed by any criterion | observability is live and attributable at boot; not performance, not correctness of the observed mechanisms, not the full matrix (W13) |
| W11-DV06 → P3-V11 | consumer-consumability review | read the snapshot/dump contracts as W12 (accounting), W13 (assertions), W14/P4 (handoff) | each consumer can state its checks without inventing new surfaces | handoff readiness; not that consumer packages are done |
| W11-DV07 → W11 closure | closure review | run the handoff checklist below and the requirement-mapping table in the entry README | every mapped requirement has design-location and evidence path; open gaps listed | W11 design closure; P3-V11 is satisfied only by DV01–DV06 evidence |

Record each validation as **passed**, **failed**, **blocked**, or
**not run** with command, input, environment, timestamp, and reason. The
plan's out-of-scope note applies to every row: no result may be claimed
from planned events alone, and nothing here proves performance.

## 2. Error, security, and observability model

- **Infallible surface.** Counter increments, emissions, and the timing
  record cannot fail the caller: no allocation, no lock, no error path.
  Observability failures (capacity conflicts, rejected seams, channel
  divergences) are *design-time* blockers recorded per the workflow §1
  boundaries — never runtime failures.
- **Failure-state guarantee of the observed systems is untouched.** W11
  adds no code to W03/W05/W07/W08 failure paths except their designated
  call points; a bug in W11 must be unable to corrupt or block the
  mechanisms (the one-way dependency is stated as a review check).
- **Security posture.** Events and dumps render typed identities and
  counts only; no guest-reachable surface, no serialization, no platform
  names. Attribution is sourced from W04's validated headers, so a
  rendering cannot attribute activity to a CPU identity the intake layer
  did not validate.
- **Observability of observability.** The boot-timing overflow flag and
  the seam register's gap list are themselves evidence: coverage gaps are
  declared, not silent (plan step 6).

## 3. Handoff checklist

Before handing W11 to a reviewer, provide:

- the exact changed-file list and the seam register with per-seam status
  (accepted / named gap);
- DV01–DV07 evidence paths and their run status, including explicit
  not-run entries (full matrix execution, W12/W13 checkpoint usage);
- confirmation that the catalog fits the W04-recorded capacity and slot
  order matches `layout_version`;
- confirmation that no transport, ring buffer, filter engine, lock,
  timer, or guest-visible interface was added, and no new `unsafe` beyond
  what the arch counter-read seam already requires (owned by the
  architecture layer, not this design);
- the non-guarantees handed downstream: no timing criteria, no cross-CPU
  coherence, event ids are a versioned namespace registration not a
  frozen ABI, and counter wrap is defined-but-unreachable at declared
  rates;
- open items for W12 (snapshot accounting obligations live in its
  scenario contract), W13 (matrix assertions), and W14 (P4 handoff
  summary of the observability contract).

## 4. Future record paths

Implementation record: `../p3-w11-smp-observability-record.md` (created
only when implementation begins). Verification record:
`../../verification/p3-w11-smp-observability-verification.md` (created
only when evidence exists). Neither exists today; neither may be claimed
into existence by this design.
