# P1-W05 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W05 detailed design](README.md).

## 1. Logical module map

W05's logical modules are exception-scope units. Their physical crate/file
placement is owned by the P0 workspace baseline and the module-tree decisions
of the designs that own the surrounding boot path; this design owns the
logical boundaries and every contract that crosses them.

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Vector table (assembly) | sixteen stubs; per-stub origin stamp; branch to the capture entry | the table region (static) | the taken exception (hardware) | a running capture entry with origin/category materialized | classification, routing, any control-register policy |
| Context capture | copy the fixed register set into `ExceptionFrame` | `CAPTURED_FRAME` static slot; recursion guard (shared with routing, §6) | live register state at entry | a completed frame pointer | interpretation, reporting, stack management |
| Vector install | baseline assertions, `VBAR_EL2` write + read-back, declaration | `VECTOR_STATE` declaration static; the audited sysreg/asm primitives | W04 declaration API; the linked table | vectors installed and declared | baseline values (W04), tracker position (W09) |
| Classification | map (origin, ESR) to `SyndromeClass` and `ExceptionDisposition` | none (pure) | frame pointer | classification result | capture, output, policy changes |
| Routing | carry the classified event to the defined outcome | none | frame + classification | pre-arm summary or W07 report; terminal stop | report body (W07), channel transport (W06), recovery |
| Phase body | the `exceptions` phase's single entry: assert → install → verify → declare | orchestration only | W09 phase call (`exceptions_step`) | vectors live or fatal route | sequencing around the phase (W09) |

## 2. Vector and origin model (work seq 1)

The AArch64 vector table has sixteen entries: four exception categories
(synchronous, IRQ, FIQ, SError) × four origins (current EL with SP0; current
EL with SPx; lower EL AArch64; lower EL AArch32). P1's runtime state fixes
which origins are legitimate:

| Origin class | Legitimate in P1? | Reason |
|---|---|---|
| CurrentEL + SPx (SP_EL2 in use) | **Yes** — the only reachable origin | SPSel=1 since W02 establishment stage 2 (W04 C1a asserted); P1 never runs with SP_EL0 selected at EL2 |
| CurrentEL + SP0 | No | P1 never selects SP_EL0 at EL2 |
| Lower EL AArch64 | No | P1 enters no lower EL; no EL1/EL0 code exists |
| Lower EL AArch32 | No | same, and P0 baseline targets AArch64 only |

Coverage rule: all sixteen entries exist and are valid entry paths
(P1-V08). The twelve non-legitimate origins classify as `InvalidOrigin` and
take the fatal disposition — they are diagnosed, never silently ignored and
never allowed to fall through.

Category expectations in P1:

| Category | Expected in normal P1 operation? | P1 classification outcome |
|---|---|---|
| Synchronous | Never (W04's no-trap policy: HCR_EL2 has no trap groups; FP/SIMD and debug accesses are denied, so even misuse faults loudly) | `FatalSyndrome` — an invariant violation of P1's own code |
| IRQ | Never (DAIF.I masked since establishment; HCR_EL2 IMO/FMO/AMO=0; no GIC configured) | `UnexpectedEvent` — mask/enabling violation; no acknowledge, no EOI |
| FIQ | Never (DAIF.F masked; same routing posture) | `UnexpectedEvent` |
| SError | Never under normal operation | `FatalSyndrome` — physical error abort; always terminal in P1 |

IRQ and FIQ entries perform **no** acknowledge, mask-change, or EOI work:
those are GIC/IRQ-subsystem mechanisms and are Out of Scope (plan). The
entries capture, classify, and terminate.

## 3. State ownership register

Every piece of exception-path state has exactly one owner:

| State | Owner | Written when | Read by | Never written by |
|---|---|---|---|---|
| `VBAR_EL2` | W05 install | the `exceptions` phase, once | hardware (vector fetch); W05 read-back | every other P1 package (W04 records it unowned) |
| Vector-table region contents | W05 (build/link); hardware (fetch) | link time | hardware | runtime code (table is read-only by class — see W08) |
| `CAPTURED_FRAME` slot | capture module | exception entry only | routing, W07 report | boot-path code outside the exception path |
| Recursion guard | exception path (capture + routing) | exception entry | routing, W07 boundary | anyone else |
| `VECTOR_STATE` declaration | W05 install | phase completion | W06/W07/W08/W09/W11 via query API | consumers (read-only) |
| W04 baseline categories | W04 (asserted here, never rewritten) | `el2-baseline` phase | W05 assertions | W05 |

The single-consumer rule on `CAPTURED_FRAME` holds because P1 has one
executing CPU and the recursive-entry guard closes the only re-entrancy
path; both facts are reviewed (W05-DV05), not assumed.

## 4. Exception-entry state machine

```text
exception taken by hardware
  -> vector stub (one of 16; origin/category stamped)
  -> recursion guard check:
       guard already set -> MINIMAL_BOUNDED_STOP (no output, no capture)   [R1]
       else set guard
  -> capture registers into CAPTURED_FRAME                                [R2]
  -> classify(origin, esr) -> (SyndromeClass, ExceptionDisposition)       [R3]
  -> route(disposition, frame):
       fatal path armed (W07) -> report_fatal_exception(frame, class)      [R4a]
       else                    -> pre-arm summary via W02 early writer     [R4b]
  -> bounded terminal stop (never returns; no ERET exists)                [R5]
```

Rules:

- R1 The guard makes recursion terminate at the earliest point, with no
  output attempt (an output path already failing is the likely cause of the
  second entry). This is the P1-V09 "without unbounded recursive failure"
  property; it is structural, not a retry policy.
- R2 Capture is bounded by construction: a fixed field set (§ of
  [entry capture](03-code-contracts-entry-capture.md) §2), fixed order, no
  stack crawl, no FP/SIMD state.
- R3 Classification is total and deterministic: every (origin, category)
  pair maps to exactly one outcome; the EC table is closed (unknown EC
  values classify as the table's explicit unknown class, never as a panic
  from inside classification).
- R4 The route decision reads W07's readiness declaration only; it reads no
  channel state (transport failure inside the chosen output is handled by
  that output's own contract, not by route switching).
- R5 The stop is the same bounded-stop discipline as W02's panic route;
  nothing after it executes.

The exception path is not a lifecycle transition (W09 T3/T5): the tracker
position is read for attribution and never advanced by the exception path.

## 5. Concurrency model

One executing CPU; `DAIF` fully masked from W02 establishment onward, and
the exception path never unmasks anything. Therefore:

- No lock is needed on `CAPTURED_FRAME` or the guard: the single CPU is
  either in the exception path or not, and the guard closes the second
  entry. The guard is a plain static flag inside the audited boundary, the
  same justification pattern as W02's panic-entry guard.
- The path may be entered from boot context (a fault during any phase after
  install) or, post-`stable`, from the idle loop (W09 T5). Both are the same
  CPU with masks set; no nested-interrupt model exists.
- No allocation anywhere: fixed-size frame, static storage, no heap (P1 has
  none — W02 scope).
- Instruction/data visibility for the vector table follows the same
  recorded implementation note as W04's writes: table contents are linked
  data, fetched after installation via an architecturally aligned
  `VBAR_EL2` write and `isb` (contract in
  [vector install](02-code-contracts-vector-install.md) §2).

## 6. Failure boundaries of the exception path itself

| Situation | Behavior | Rationale |
|---|---|---|
| Exception while the guard is set (recursive entry) | minimal bounded stop, no output | the first path may already be producing output; silence is the safe terminal (P1-V12) |
| Exception before `exceptions.complete` (unowned window) | outside P1's owned surface (W09 limitation; W05 narrows the window, does not eliminate it — install happens inside the phase) | recorded lifecycle limitation; no pre-install mitigation is designed |
| Output transport fails during the pre-arm summary | the summary loop inherits the early writer's architectural poll semantics; the terminal stop is reached regardless (W02 R4 posture) | the stop must not depend on successful output |
| W07 report path misbehaves (post-arm) | W07's own boundary owns recursion containment for its report; W05's guard is already set for the duration of the route | one guard, transferred discipline per the W02 seam precedent |
| Classification meets an EC value outside the table's named classes | classify as the table's unknown class; proceed to route | closed-vocabulary rule; an unclassifiable syndrome is still reportable |

## 7. Assumed contracts and failure boundaries

| Seam | Supplied by | Used for | Failure boundary if it delivers differently |
|---|---|---|---|
| `el2-baseline` categories C1–C8 established; declaration API (`baseline_status`) | [W04](../p1-w04-el2-architectural-state-baseline/README.md) (accepted design) | install-time assertions; FP-free capture premise; masks/routing posture | a NotEstablished category stops install (phase-attributed `exceptions` failure via panic route); W05 never rewrites W04-owned controls |
| `exceptions` phase placement; tracker attribution; post-`stable` route | [W09](../p1-w09-initialization-sequencing/README.md) (accepted design) | when install runs; how failures and post-stable faults are attributed | seam mismatch raised per W09 §1; not adapted locally |
| Panic route; `early_write_bytes`; bounded formatter discipline | [W02](../p1-w02-minimal-rust-el2-runtime/README.md) (accepted design) | install-failure route; pre-arm summary transport | if the writer cannot carry the summary vocabulary, W02/W05 coordination issue; no second output path is built |
| `report_fatal_exception(frame, class)` seam; readiness/arming declaration | [W07](../p1-w07-fatal-crash-diagnostics/README.md) (parallel design) | post-arm full report | until W07's items exist the armed branch does not compile; W05 delivers the pre-arm path and records the deferred branch (W02 decision-6 precedent). No stub report, no fake readiness |
| Channel availability (post-arm transport preference) | [W06](../p1-w06-early-console-logging/README.md) (parallel design) | full-report transport when available; exception-context callability | W05 consults W07/W06's recorded preference order; it never probes the UART itself |
| Region attributes for the vector table (post-MMU) | [W08](../p1-w08-host-stage1-address-space/README.md) (parallel design) | vectors remain executable/read-only across the MMU transition | attribute conflict is a W05/W08 coordination issue raised, never absorbed |
| No-FP build guarantee | P0 target semantics (planned) | capture contains no FP state | recorded dependency, verified by review; escalation per W04's recorded reservation |

Produced for consumers: the installed, verified vector baseline and its
declaration; the `ExceptionFrame` and its field contract; the classification
vocabulary and dispositions; the token classes; the region identity for W08.
