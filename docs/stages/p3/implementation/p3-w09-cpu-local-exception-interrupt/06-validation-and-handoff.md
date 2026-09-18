# P3-W09 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W09 detailed design](README.md).

## 1. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W09-DV01 | Slot contents legal and initialized | layout/init review + tests | init/epoch validation on a slot-fake; begin/end pairing and depth invariants | every slot initializes; pairing/depth exact; corruption paths fatal | contract correctness of the state; not hardware vector behavior |
| W09-DV02 | Legal per-CPU entry/local context on boot and secondary roles | intentional synchronous-fault scenarios (per the P1-W05 acceptance pattern) | induced fault per CPU role on the reference boot; slot state and diagnostics captured | each role's path records correct local context; nesting bound terminal; P1 non-recursion holds per CPU | per-CPU correctness of the supported paths; not GIC/IRQ delivery (unimplemented at P3) |
| W09-DV03 | CPU identity correct in exceptional paths | rank-table tests + role scenarios | injected header states (valid / invalid + MPIDR / unusable); rank and fields asserted per state | exact rank and fields per injected state; no write before rank resolution | the attribution rule; not attribution under hardware-level register corruption beyond the modeled cases |
| W09-DV04 | Non-overlapping exception state | NO-review + tests | enumerate writable locations reachable from vector entry; each must be own-area or console; cross-CPU fault storm test touches only own slots | enumeration clean; storm shows no cross-CPU writes | the non-overlap invariant for reviewed code; not P0–P2 state (that is W10's audit with the same criteria) |
| W09-DV05 | CPU-attributed fatal diagnostics | fatal-path tests | forced fatal per role; console contention injected (poisoned lock) | every report attributed (rank recorded); record write-once; bounded fallback, no hang | the fatal integration; not crash storage or post-mortem tooling (out of scope) |
| W09-DV06 | Safe simultaneous diagnostic/logging | concurrency tests | N-emitter simultaneous logging; contended fatal case; CL-3 attribution check on every line | no partial-line interleaving (normal case); contended output marked/attributed/bounded | the logging rules; not console hardware behavior or W11 catalog coverage |
| W09-DV07 | Integration and posture recording | closure review | read integration map vs W10–W13 plans; confirm P1 posture unchanged (no new interrupt source), no vector-mechanics edits | consumers' obligations citable; posture statement recorded | handoff readiness; not that consumers are done |

Task-book trace: P3-V09 requires correct CPU identity and local
diagnostic context on supported boot and secondary CPU exceptional paths.
W09 delivers the per-CPU foundations and single-pass role evidence;
repeated execution across the 1/2/4/8 matrix is
[P3-W13](../p3-w13-qemu-smp-regression/README.md) and the concurrent
fault/logging stimuli belong to
[P3-W12](../p3-w12-smp-stress-failure-tests/README.md). Evidence states
(passed / failed / blocked / not run) are recorded with command,
environment, and timestamp in
`../../verification/p3-w09-cpu-local-exception-interrupt-verification.md`;
implementation decisions go to
`../p3-w09-cpu-local-exception-interrupt-record.md`. Neither file is
created by this design.

## 2. Error, security, and observability model

- **Errors:** exceptional paths do not return errors; they classify —
  recoverable-diagnostic (slot bookkeeping, bounded nesting) or terminal
  fatal (bound exceeded, corruption, invariant violation) per the P0-W14
  classification. Slot corruption is always fatal-with-attribution,
  never silently repaired.
- **Security:** the surface is host-only; there is no guest reach and no
  authorization decision. The security-relevant property is integrity of
  attribution: a wrong or missing CPU identity in a fatal report is
  treated as a diagnosability defect of the same severity class as the
  fault itself. Untrusted-input rules do not apply at P3 (no guest), and
  this boundary is recorded for P4, whose guest-fault diagnostics must
  add validation before any guest-influenced value reaches these paths.
- **Observability:** every exceptional-path emission carries the CL-3
  attribution fields; per-CPU fatal records are the durable post-mortem
  surface (cross-CPU read-only). W11 owns catalog/format governance; W09
  guarantees the fields and emission points.

## 3. Handoff checklist

Before handing W09 to a reviewer, provide:

- the changed-file list and the `unsafe` inventory entries (register
  reads in the attribution path; any entry-path hook placement);
- DV01–DV07 evidence paths with run status, including explicit not-run
  entries (repeated-matrix execution, IRQ-driven paths, hardware
  behavior);
- the recorded `EXCEPTION_NESTING_MAX` value and rationale;
- the DV04 non-overlap enumeration result (also delivered as a W10
  audit input);
- the posture statement: no new interrupt source enabled; P1 vector
  mechanics unmodified; conflict status of any P1-contract boundary
  encountered;
- open items for W10 (audit criteria consumption), W11 (catalog for
  emission events), W12/W13 (concurrent-fault and matrix scenarios),
  W14/P4 (diagnostic foundation as handoff content; guest-input
  validation obligation recorded), and the future host-IRQ design owner
  (Reserved triggers) — without resolving their contracts here;
- confirmation that no GIC access, interrupt enablement, vector-table
  redesign, global exception buffer, or new log sink was introduced.
