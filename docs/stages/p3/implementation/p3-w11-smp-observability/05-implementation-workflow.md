# P3-W11 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W11 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the current tree. Useful
read-only discovery: `git ls-files` (confirm which sibling designs and
records actually exist) and reading the W04 implementation record for the
recorded counter-block capacity and `layout_version` in force.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the P0-W12 or P0-W13 approved contracts exist and diverge from the
  assumptions in [01](01-scope-and-foundations.md) §1.2 (channel
  properties, namespace shape) — adapt where the contract merely differs,
  stop and record where it blocks a catalog row;
- the W04 implementation record's capacity cannot hold the §2 catalog —
  capacity is W04's; record the conflict for a W04 design change, never
  truncate the catalog or widen the block locally;
- a seam owner (W06/W07/W08 or their designs) rejects a §6 call point —
  record the gap; do not relocate their logic or add polling observers;
- making counters or events work appears to require a telemetry transport,
  lock, allocator change, or timer — that belongs to P0-W12/W06/P6, and
  reaching for it here is a scope violation.

## 2. Ordered implementation steps

### Step 1 — confirm the storage seam

Target: the W04 per-CPU area's `TelemetryCounters` block.

Work: read W04's implementation record for the block's capacity,
alignment, and `layout_version`; map the
[counter catalog](03-code-contracts-counter-block.md) §2 onto the slots.

**Acceptance:** every catalog counter has a slot; reserved slots remain;
slot order equals catalog order.  
**Failure/blocker:** capacity shortfall is a recorded W04-conflict
blocker (§1); no silent truncation.

### Step 2 — implement the counter primitives

Target: `SmpCounterId`, `increment`, `record_contention`
([03](03-code-contracts-counter-block.md) §1–§4).

Work: implement with host-side tests using a fake per-CPU block and a
simulated-exception reentrancy harness.

**Acceptance:** W11-DV02 tests pass: reentrancy totals are exact,
fetch-max high-water updates are correct, wrap behavior matches the
documented rule, no test uses a lock or an allocator.  
**Failure/blocker:** a test failure means the implementation deviates from
the contract — fix the implementation, not the test; an apparent need for
stronger ordering than Relaxed is a design question to raise, not a local
ordering change.

### Step 3 — implement the read surfaces

Target: `snapshot`, `aggregate`, `render_counter_dump`
([03](03-code-contracts-counter-block.md) §5–§6).

Work: implement over the W04 lookup table and W03 `OnlineSet` with the
W05 phase gate; host-side tests with fake blocks and a fake online set.

**Acceptance:** rows cover exactly the online set (or all records for the
diagnostic option); aggregation matches hand-computed sums; rendering is
allocation-free.  
**Failure/blocker:** a missing upstream surface (W03/W04/W05) blocks this
step — record which prerequisite failed, do not substitute a parallel
access path.

### Step 4 — implement attribution and the event catalog

Target: `CpuAttribution`, `SmpEventId`, `emit`
([04](04-code-contracts-event-catalog.md) §1–§3).

Work: implement the exhaustive enum and the emission routing per channel
and trim class; register the inventory under the P0-W13 namespace rules as
they exist at implementation time.

**Acceptance:** every catalog row emits through `emit` with mandatory
attribution; `DebugOnly` rows compile out under the declared trim
profile; no row content contains a platform name or untrusted data.  
**Failure/blocker:** a namespace/channel contract divergence follows the
§1 boundaries.

### Step 5 — wire the seams and the boot-timing record

Target: call points in W02/W03/W05 code where those designs already
designate them; `BootTimingRecord` at W05's designated points
([04](04-code-contracts-event-catalog.md) §4–§6).

Work: add the emission/increment calls at the owning packages' designated
points (their designs and implementation records own the placement);
implement the timing record writes where W05 designates.

**Acceptance:** the seam register records each point as accepted by the
owning package's design/record, or as a named gap with its coverage
consequence; boot captures show phase and rendezvous rows per boot.  
**Failure/blocker:** a rejected seam is a recorded gap (§1); observability
gaps never justify changing the observed mechanism.

### Step 6 — governance and no-board-leakage review

Target: the catalog, ids, and rendered content.

Work: review naming, domains, and version fields against the P0-W13
contract as delivered; review channels, levels, and trim semantics
against P0-W12; grep the catalog and rendering for platform/board/QEMU
identifiers (ADR-043/ADR-052); confirm no pass criterion anywhere
consumes `boot.timing.raw`.

**Acceptance:** W11-DV01 and W11-DV03 review evidence exists under the
stated environment.  
**Failure/blocker:** a governance contradiction is raised per §1; it is
never absorbed by rewording this design's rows silently.

### Step 7 — evidence and records

Target: implementation record and verification record.

Work: collect the P3-V11 capture evidence (SMP-ready counter dump plus
event presence across at least the minimum declared CPU count in the
declared test environment); record results honestly as passed/failed/
blocked/not-run in
`../../verification/p3-w11-smp-observability-verification.md`; record
decisions and deviations in
`../p3-w11-smp-observability-record.md`. Matrix execution across the full
1/2/4/8 CPU counts and repetitions belongs to
[P3-W13](../p3-w13-qemu-smp-regression/README.md) and is not claimed here.

**Acceptance:** the verification record distinguishes what ran from what
is deferred to W12/W13.  
**Failure/blocker:** a failed or blocked item is evidence of a failure —
record it; do not widen scope to make it pass.
