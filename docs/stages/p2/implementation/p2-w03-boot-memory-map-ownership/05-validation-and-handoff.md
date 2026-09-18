# P2-W03 Validation, Error Model, and Handoff Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W03 detailed design](README.md).

## 1. Scope of validation for this package

W03's own evidence is host-side over synthetic fact records. The W04
metadata-planning interplay (draft → plan → seal) is validated jointly with
W04 but the seal contract is exercisable now with hand-built plans. QEMU
accounting evidence (map totals versus observed RAM) is W09; conflict
regression coverage is W08's harness over this package's policy table.

## 2. Error, security, and observability model

- **Error model.** Six fatal classes
  ([02 §9](02-architecture-and-state.md)) — boot stops with one diagnostic
  carrying source identities; five recorded anomaly classes (R4, R7, R8,
  R9, R10) plus clips. No input defect can produce a panic: interval
  arithmetic is checked and total.
- **Security model.** The hard gate is structural: allocation authority
  does not exist before seal; protected-wins clipping means a hostile RAM
  description can only *reduce* allocatable memory; unknown future classes
  default to protected (fail-closed); re-derivation audit at seal means
  the accounting equation is checked, not assumed. Residual risk: a
  platform whose *protection statements* omit a genuinely reserved range
  (under-declaration) is invisible to W03 — it cannot protect what
  firmware does not declare; this limit travels in the W10 handoff.
- **Observability.** `MapSummary` plus anomaly/clip counters are queryable
  and rendered by W06; one boot-log marker "memory map sealed" with
  totals (ram/allocatable/protected frames, clip count). Diagnostics name
  source identities, never memory contents.

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W03-DV01 → P2-V05 (P2-D05) | Frame conversion | Host unit tests | Unaligned base/len, zero len, overflow boundary fixtures | Exact `MapFatal` classes; no wrap; empty ranges impossible | Conversion correctness; not firmware alignment behavior |
| W03-DV02 → P2-V05 (P2-D02) | Image protection | Host tests | Image range inside/at/outside RAM; absent image range | Protected in all in-RAM cases; absent → blocked-defect stop path exercised | Image gate; not P1's range authority |
| W03-DV03 → P2-V05 (P2-D03) | DTB + rsvmap protection | Host tests | DTB in RAM, rsvmap entries incl. duplicates and out-of-RAM | `ActiveDtb` + `DtbReservation(i)` entries with R7/R10 outcomes | DTB-side protection; not DTB content validity (W01) |
| W03-DV04 → P2-V05 (P2-D04) | Reserved-memory + artifacts | Host tests | no-map/reusable children, initrd artifact, malformed artifact | All recorded protected with sources and flags | Record fidelity; not release policies (Reserved) |
| W03-DV05 → P2-V05 (P2-D05) | Conflict policy R1–R3, R6 | Host tests | Every fatal row of the policy table | Fatal class with correct identity detail; boot-stop path | Ambiguity is fatal, not guessed; not real-firmware behavior |
| W03-DV06 → P2-V05 (P2-D05) | Anomaly policy R4, R7–R10 | Host tests | Every anomaly row | Recorded with counters, never fatal, never protection-reducing | Anomaly containment; not downstream interpretation |
| W03-DV07 → P2-V05 (P2-D06) | Normalization invariants | Host property test | Random bank/protected sets through the builder | Sorted, disjoint, full RAM coverage on every run | Normalization soundness; not exhaustive platform space |
| W03-DV08 → P2-V05 (P2-D07) | Hard-gate property | Host property test | Random sets; for each frame in RAM assert class; cross-check allocatable spans vs protected set | No frame both allocatable and protected; clip log explains every RAM shrink | The central P2 safety property at map level; not allocator behavior (W04) |
| W03-DV09 → P2-V05 (P2-D08) | Seal contract | Host tests (with W04 for planner interplay) | Valid plans; each R11 violation; double seal attempt | Seal accepted only for plan-in-allocatable; metadata class present; re-seal impossible | Metadata-protected-before-allocation property; not W04's planner quality |
| W03-DV10 → P2-V05/P2-V13 (P2-G01–G03) | Extension foundation + ACR | Design review | Simulate adding a future class/source; check fail-closed default; confirm no object design crept in; P2-ACR-01 restated | Unknown classes default protected; ledger identities stable; no `MemoryObject`/`MemoryRegion` anywhere | Extension readiness and ACR visibility; not P4 mechanisms |

Evidence statuses are passed / failed / blocked / not run with command,
input, environment, timestamp. Host validation does not prove QEMU boot
integration (W09), allocator behavior (W04), or real-firmware map fidelity.

## 4. Handoff checklist

Before handing W03 to review, provide:

- changed-module list; confirmation of zero allocation, zero `unsafe`, zero
  board/platform names, zero mutation APIs on the sealed map;
- W03-DV01–DV10 evidence statuses with not-run entries (QEMU accounting →
  W09; planner interplay → W04; consumer rendering → W06);
- confirmed consumer readiness: W04 (draft queries + seal handshake +
  sealed authority), W06 (summary/clip/anomaly queries), W08 (policy
  table as regression oracle), W09 (accounting totals), W10 (ledger
  identities, extension protocol, under-declaration limit);
- open items recorded, not resolved: P0/P1 assumed contracts, physical
  placement (P0-W03), DTB copy/release Reserved, P2-ACR-01 unresolved and
  blocking any memory-object work;
- explicit statement of the hard-gate division of responsibility: W03
  guarantees the map; W04 must derive allocation only from it.
