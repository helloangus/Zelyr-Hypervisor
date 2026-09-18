# P3-W11 Code Contracts — Event Catalog and Emission

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W11 detailed design](README.md).

Contracts follow the project function/type template. Event ids below are
design-level identifiers in the P0-W13 namespace shape; the approved
P0-W13 contract owns the exact string/naming scheme and final domain
assignment (failure boundary in [01](01-scope-and-foundations.md) §1.2).
The catalog is a versioned namespace registration, not a stable external
ABI.

## 1. `CpuAttribution` and `BootRole`

```text
Name and stability: CpuAttribution { logical: LogicalCpuId,
    hardware: HardwareCpuId, role: BootRole }; BootRole ∈ { Boot,
    Secondary } — value types; internal.
Purpose and caller: the mandatory per-event identity triple (plan
    requirement); callers: emission API, snapshot rows, dump rendering.
Inputs / outputs: sourced from the installed per-CPU header (W04) —
    never from caller-supplied identity arguments for the emitting CPU.
Preconditions / postconditions: reflects the header W04 installed and
    validated; immutable after install.
State and ownership change: none (value type).
Concurrency/allocation context: Copy; no allocation.
Errors and failure guarantee: none.
Security/authorization checks: none.
Logic: derived read of the area header's identity fields + boot flag.
Validation: W11-DV03 attribution review and tests.
```

## 2. Event inventory

Context columns: **Ch** = channel class (T = structured trace, C = crash
governed); **Trim** = trim class (A = Always, D = DebugOnly); **Ctx** =
allowed execution context (B = boot/process context only,
E = exception-safe, R = rendezvous-time on coordinator).

| Group | Id (design-level) | Context fields beyond attribution | Emitted by / seam owner | Ch | Trim | Ctx |
|---|---|---|---|---|---|---|
| Lifecycle | `cpu.lifecycle.transition` | from, to (W03 states), cause | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) transitions | T | A | B/E |
| Lifecycle | `cpu.discovered` | class (W01 TopologyClass), exclusion note if any | topology intake (W01) at freeze | T | A | B |
| Lifecycle | `cpu.start.requested` | target: LogicalCpuId | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) requester | T | A | B |
| Lifecycle | `cpu.start.outcome` | target, outcome (W02 `SecondaryStartOutcome`), phase (W02 `StartPhase`) | W02 outcome paths | T | A | B/E |
| Notification | `cpu.notification.sent` | target, result (accepted/refused: W07 vocabulary) | [P3-W07](../p3-w07-cross-cpu-notification/README.md) send path | T | D | E |
| Notification | `cpu.notification.received` | source if W07's protocol carries one | W07 reception path | T | D | E |
| Transport | `cpu.tlb.request.sent` | target set (typed mask) | [P3-W08](../p3-w08-tlb-shootdown-transport/README.md) request path | T | D | B/E |
| Transport | `cpu.tlb.request.received` | requester if W08's protocol carries one | W08 reception path | T | D | E |
| Transport | `cpu.tlb.complete.observed` | request correlation per W08's protocol | W08 completion path | T | D | E |
| Contention | `cpu.contention.observed` | site, wait_iterations | [P3-W06](../p3-w06-concurrency-synchronization/README.md) via `record_contention` | T | D | E |
| Boot sync | `boot.phase` | from, to (W05 `BootPhase`) | [P3-W05](../p3-w05-smp-boot-synchronization/README.md) transitions | T | A | B |
| Boot sync | `boot.rendezvous.result` | ready count, degraded list | W05 coordinator | T | A | B |
| Boot sync | `boot.smp_ready` | `SmpReadyState` (Ready/Degraded + failed set) | W05 declaration | T/C | A | B |
| Boot sync | `boot.timing.raw` | phase marker, raw counter value/delta | this design's record writer | T | D | B |

Coverage check against the plan scope: CPU discovered (row 2),
start-requested (3), entered-EL2 (4 via `entered` outcomes; slot 2 counts
them), online (1 via `Online` transitions), failed (1/3/4), notification
sent/received (5/6), TLB transport request/completion (7–9),
synchronization contention (10), boot-synchronization timing (14 plus the
record). Every row carries `CpuAttribution`; directed rows carry the
typed target/source. No row carries a platform, board, or SoC name.

Registration: the inventory is registered under the P0-W13 namespace at
implementation time per its new-event review rule; the registration
records id, domain, version, and owning package (W11) for every row.

## 3. `emit`

```text
Name and stability: emit(event: SmpEventId, ctx: EventContext) — internal;
    the only event emission path.
Purpose and caller: routes one catalogued event to the P0-W12 channel
    with mandatory attribution; callers are the owning packages at the
    seam points of §2.
Inputs / outputs: event id and a typed context (attribution + the row's
    fields); no return.
Preconditions / postconditions: precondition — the event id is
    catalogued (compile-time exhaustive enum; adding an id is a catalog
    change, never an ad-hoc string — the P0-W13 rule) and the caller
    satisfies the row's Ctx column. Postcondition — the channel received
    the event or, for `DebugOnly` events compiled out under the declared
    trim profile, the call is a no-op; neither case can fail the caller.
State and ownership change: none in W11 state; channel state is the
    P0-W12 contract's.
Concurrency/allocation context: no allocation, no lock, no call into
    arbitrary subsystems — an E-ctx emission is exactly as safe as a
    counter increment; the channel contract must share this property for
    E-ctx rows (failure boundary recorded in [01] §1.2).
Errors and failure guarantee: infallible; observability must never turn a
    mechanism failure into a new failure mode (ADR-048's premise).
Security/authorization checks: content review (no untrusted data, no
    platform names) is a catalog-review obligation (W11-DV03), not a
    runtime check.
Logic (pseudocode):

    emit(id, ctx):
        if cfg-trimmed(id): return          # DebugOnly under declared profile
        channel = channel_of(id)            # fixed per catalog
        channel.record(id, ctx.attribution, ctx.fields)

Validation: W11-DV01 catalog review; W11-DV02 trim test; W11-DV03
    attribution completeness review.
```

## 4. Contention observation contract (W06 seam)

Named requirement on [P3-W06](../p3-w06-concurrency-synchronization/README.md):
at every site W06 designates as contention-observable, a contended
acquisition calls `record_contention(site, iterations)` after the
acquisition completes and before any further shared-state work; W06 also
emits `cpu.contention.observed` (§2) when `DebugOnly` events are enabled.
Requirements and limits:

- The observation must not extend the critical section: call placement is
  after release of the contended resource (or after acquisition where W06's
  design states the wait has ended — the owning design fixes the point).
- `iterations` is W06's own poll/loop count, not a wall-clock measure.
- Uncontended acquisitions are unobserved (zero-cost path); a site that
  cannot detect contention cheaply is recorded by W06 as unobserved and
  listed in the gap register — coverage is declared, not assumed.

If W06's approved design cannot accept these points without changing its
lock semantics, that is a conflict between the two designs resolved per
the sibling-conflict rule; W11 does not redesign locks to make them
observable.

## 5. Boot-timing observation contract

```text
Name and stability: BootTimingRecord — fixed-capacity boot-only record;
    internal; capacity covers the W05 phase transitions plus rendezvous
    start/end markers (bounded small constant fixed in the implementation
    record with rationale).
Purpose and caller: the P3 form of boot-synchronization timing; caller:
    the boot CPU at W05's designated points and at rendezvous start/end.
Inputs / outputs: write(phase_marker) captures one raw counter reading
    from the arch seam (opaque raw value); read access renders deltas.
Preconditions / postconditions: writes occur only pre-SMP-ready on the
    boot CPU; record is full if W05 calls beyond capacity — a full record
    drops further entries and sets an overflow flag (diagnosable, never
    fatal, never blocks boot).
State and ownership change: append-only pre-SMP-ready; read-only after
    W05's publication fence.
Concurrency/allocation context: single-writer (boot CPU), no lock;
    publication coherence rides W05's `declare_smp_ready` fence
    ([P3-W05](../p3-w05-smp-boot-synchronization/README.md) §5).
Errors and failure guarantee: none (drop-plus-flag on overflow).
Security/authorization checks: raw values only; no unit, frequency, or
    wall-clock conversion exists anywhere in this module.
Logic: append (marker, raw); delta view = adjacent raw subtraction for
    display only.
Validation: W11-DV01 classification review (informative-only rule);
    W11-DV05 capture.
```

Hard rule, restated from the entry README decision 6: no pass/fail
criterion in any P3 package may consume `boot.timing.raw` values or
deltas; consumers needing timing semantics wait for the P6 timer baseline
(Reserved, entry README §2.2).

## 6. Seam acceptance register

Each §2 row whose emitter is another package is a *requirement on that
package*, tracked in a seam register maintained by this design's
implementation record:

| Seam | Owning package | Required call point | Acceptance status |
|---|---|---|---|
| lifecycle.transition | W03 | inside each legal transition, after the CAS succeeds | recorded at implementation start |
| discovered | W01 | at topology freeze per entry | recorded at implementation start |
| start.requested / start.outcome | W02 | requester after each CPU_ON; outcome paths on terminal results | recorded at implementation start |
| notification.sent / received | W07 | send path per attempt; reception path per accepted arrival | recorded at implementation start |
| tlb.request.sent / received / complete | W08 | request, reception, acknowledgement points | recorded at implementation start |
| contention.observed | W06 | designated sites, per §4 | recorded at implementation start |
| boot rows | W05 | phase transitions, coordinator result, declaration | recorded at implementation start |

A seam the owning design cannot accept is recorded as a named gap with
its coverage consequence; unresolved gaps at W11 closure are listed in
[06-validation-and-handoff.md](06-validation-and-handoff.md) §3 and block
only the affected P3-V11 coverage, per the validation matrix's honest-
status rule.
