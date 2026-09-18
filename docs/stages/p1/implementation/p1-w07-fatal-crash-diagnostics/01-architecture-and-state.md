# P1-W07 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W07 detailed design](README.md).

## 1. Logical module map

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Report model | field sets per kind, ordering, availability labels, marker prefixes | none (declarative + pure formatting) | kind-specific inputs | bounded line set | transport, event capture, decisions |
| Panic entry | the `#[panic_handler]` body (superseding W02's via the recorded seam) | guard (transferred; §4) | `PanicInfo` | full panic report; terminal stop | exception capture (W05), phase routing (W09) |
| Exception entry | render the classified-exception report | none beyond the shared guard | `ExceptionFrame`, `SyndromeClass`, `ExceptionDisposition` (W05) | full exception report; terminal stop | capture (W05), classification policy (W05) |
| Phase-failure entry | render the phase-failure report for W09's routes | none beyond the shared guard | `InitPhase`, reason | full phase-failure report; terminal stop | the routes themselves (W09 `fail_phase`) |
| Arming | readiness assertions + the readiness declaration | `FATAL_PATH_READY` once-flag | W09 phase call (`fatal_path_step`) | the readiness signal W05 consumes | the assertions' subjects (transports, identity — read, not owned) |
| Transport seam | select and drive W06 channel or W02 early writer | none | rendered lines | transmitted bytes | channel internals (W06), writer internals (W02) |

## 2. The three-kind entry model (work seq 2)

```text
failure event
  ├─ Rust panic                -> p1_panic(info)                  [kind P]
  ├─ classified exception      -> report_fatal_exception(...)     [kind E]
  │                              (from W05's router, post-arm)
  └─ phase failure             -> report_fatal_phase(...)         [kind F]
                                 (from W09 fail_phase: Stage1|Stable;
                                  FatalPath misuse per its matrix)
all kinds:
  common context block -> kind-specific core -> end marker -> bounded stop
```

Call-direction invariant: the fatal path is entered only from the panic
entry, W05's post-arm router, or W09's `fail_phase`. It never calls back
into boot-path mechanisms (no tracker advances, no channel init, no
baseline queries that could route) — it only reads: tracker position,
identity accessor, frame contents, availability flags.

## 3. State ownership register

| State | Owner | Written when | Read by | Never written by |
|---|---|---|---|---|
| Fatal-path guard | W07 fatal path (transferred from W02 per the seam) | first fatal entry | all three entries | everyone else |
| `FATAL_PATH_READY` | arming | `fatal-path` phase body, once | W05 router; reviews | consumers |
| Report line buffers | the renderer (stack-local per report) | during one report | transport seam | — |
| Marker-class prefixes | W07 (link-time literals) | link time | W05 summary (fatal class), W10 matching | everyone |
| `ExceptionFrame` | W05 (read-only here) | exception entry | exception entry, W05 summary | W07 |
| UART (via transports) | W06 channel / W02 writer per preference | each report | — | W07 directly (access only through the transports) |

## 4. Panic-handler ownership transfer (the W02 seam)

The transfer is the mechanical content of parent-README decision 2:

- **Moves:** the `#[panic_handler]` registration; the single-entry guard
  (one static, now the shared fatal-path guard); the bounded-stop
  discipline; the report body (W02's minimal set superseded by the report
  model of §02).
- **Stays:** W02's establishment-order stage 8 ("panic route ready" before
  `Runtime.complete`) — still true, since the handler is linked from image
  start; W02's `early_write_bytes` and its constant — still the
  channel-independent fallback transport, now consumed through the
  transport seam; W02's single-consumer rules — unchanged in their
  windows.
- **Never moves:** the W01 pre-transfer rejection reporter (separate path,
  W01 §3); the W05 entry-path guard (a distinct, earlier-stage guard —
  see §6).

The transfer happens once, in W07's implementation, and is recorded in the
implementation record; W02's minimal body does not survive alongside it
(two bodies would be two routes).

## 5. Arming lifecycle and pre/post-MMU availability (work seq 3)

```text
LINKED-NOT-READY (image start .. fatal-path phase)
  panic entry: full report, any phase after runtime establishment
  exception events: W05's pre-arm summary (W05 §4 routing — this design
    owns nothing there)
  phase-failure entry: reachable only for phases that route here after
    arming (Stage1/Stable); before arming, W09's fail_phase uses the
    panic route per its §7 match
  -> fatal-path phase body: readiness assertions
       (identity resolves-or-recorded-degradation; guard clear; a
        transport is reachable: channel_available() or the linked writer;
        renderer linked — by construction)
  -> FATAL_PATH_READY (once-flag set; assertion failure arms nothing and
     routes via W07's own non-recursive readiness boundary, §6)
  -> post-arm: exception events produce the full report through W05's
     router; Stage1/Stable phase failures produce kind F reports
MMU transition (W08 `stage1` phase, after arming):
  the preference order is MMU-state-independent: both transports are
  mapped windows under W08's recorded classes before `SCTLR_EL2.M` is
  set, so the same evaluation — channel if available, else writer — holds
  on both sides of the transition. No report is ever lost to the
  transition itself; a post-MMU fault reports through the identical path
  (W09 matrix `stage1`/post-`stable` rows).
```

## 6. Failure boundaries of the fatal path itself (work seq 4)

| Situation | Behavior | Rationale |
|---|---|---|
| Second fatal entry while the guard is held (fault during report; panic inside exception report; misuse) | silent bounded stop — no output attempt | the first report may already be mid-line; silence is the safe terminal (P1-V12); W02's guard-true rule, now stage-wide |
| Arming assertion fails | the `fatal-path` phase's own readiness boundary: one minimal marker line via the early writer (best effort), then bounded stop; the readiness flag stays false so W05's router keeps using the pre-arm summary | W09 matrix `fatal-path` row ("must not recurse"); honest degradation, not fabricated readiness |
| Transport stalls mid-report | the selected transport's own semantics (W06: polled, harness-bounded; W02: architectural poll, stop reached regardless); the report does not switch transports mid-report | re-negotiation would be a second failure mode on the fatal path |
| Tracker position unreadable/degraded | decode yields `Unknown(raw)` and renders as such (W09 §2); the report continues | attribution must never cost the report its life |
| Identity unavailable | W02's recorded unavailable literal renders; the report continues | never fabricated (W02 §2 rule) |
| Fault while a boot-path guard other than W05's is held (e.g. during a once-cell write) | the event enters the fatal path through the normal entries; the shared guard serializes reporting; corrupted-cell state is *reported*, not repaired | recovery is Out of Scope (plan); reports reflect observed state |

## 7. Concurrency model

Boot CPU only; masks set in every context that can reach the path (boot
context by establishment, exception context by W05's discipline). No
allocation, no locks, no atomics beyond the guard flag and the once-flags
(the audited once-write cell family; SAFETY: single boot CPU, DAIF masked,
one boot path; each written once, read after). The path is callable from
boot context (panic, phase failure) and exception context (post-arm
exception reports); the guard plus the contracted call direction make
interleaving impossible in P1 — the same single-CPU justification family
as W02/W05/W06.

## 8. Assumed contracts and failure boundaries

| Seam | Supplied by | Used for | Failure boundary if it delivers differently |
|---|---|---|---|
| Extension seam (report body, guard, stop discipline) | [W02](../p1-w02-minimal-rust-el2-runtime/README.md) (accepted design, §5 of its panic/identity contracts) | the ownership transfer of §4 | a mismatch stops the transfer step and is raised to the W02 owner; no parallel handler is created |
| `ExceptionFrame` fields; classification inputs; post-arm router | [W05](../p1-w05-el2-exception-entry-baseline/README.md) (parallel design) | kind E inputs; the pre-arm boundary that precedes this path | a field gap is a W05/W07 coordination issue; W07 does not capture registers itself |
| `fatal_path_step` placement; `fail_phase` arms; tracker read; `LifecyclePosition` | [W09](../p1-w09-initialization-sequencing/README.md) (accepted design) | arming timing; kind F routing; the phase field | seam mismatch raised per W09 §1; never adapted locally |
| Channel availability + write; framing rules | [W06](../p1-w06-early-console-logging/README.md) (parallel design) | preferred transport post-availability | if the channel cannot serve the renderer, the preference falls back by contract; no third transport is built |
| `early_write_bytes`; `BuildIdentity` accessor | [W02](../p1-w02-minimal-rust-el2-runtime/README.md) (accepted design) | fallback transport; identity field | identity gap degrades per decision 6; writer gap is a W02 coordination issue |
| Transport windows across the MMU transition | [W08](../p1-w08-host-stage1-address-space/README.md) (parallel design) | post-MMU diagnostic availability (its plan work seq 4 obligation) | a window gap is a W08 design defect caught by its own reviews and W11's NC5; W07 adds no mitigation |
| Failure-class semantics; minimal crash-information principle | P0-W14 / P0-W12 (planned) | the classification posture of reports; the required-context floor | upstream defect recorded per [workflow](04-implementation-and-review.md) §1; no substitute taxonomy |

Produced for consumers: the report model and marker classes; the three
entry contracts; the readiness declaration; the transport preference; the
guard discipline.
