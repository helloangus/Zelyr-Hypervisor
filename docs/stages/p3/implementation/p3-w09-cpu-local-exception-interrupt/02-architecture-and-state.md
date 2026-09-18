# P3-W09 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W09 detailed design](README.md).

## 1. The per-CPU exceptional-path model

```text
EXCEPTION ENTRY (one CPU)                        OTHER CPUs
-------------------------                        ----------------------
vector entry (P1-W05 mechanics)                  unaffected; their own
  |                                              slots are untouched
  v
resolve identity (attribution ranks, §2)
  | rank 1: current() header valid
  | rank 2: MPIDR fallback (pre/during install)
  | rank 3: unknown
  v
exception_begin(): own slot depth++, flag set,
  vector-class note recorded
  |
  v
P1 origin classification / capture (P1 contract)
  | recoverable-diagnostic path        fatal path
  v                                     v
diagnose: console under Diagnostics    fatal_record() into OWN slot
  lock (W06); attribution line           (lock-free, CPU-private)
  mandatory                              then bounded-try-lock console;
  |                                      fallback = best-effort with
  v                                      interleave marker
exception_end(): depth--, flag clear   terminal state (no recovery)
```

Everything writable on the path except the console bytes is inside the
faulting CPU's own `PerCpuArea`. The console is the single cross-CPU
shared resource, governed by the W06 Diagnostics rules.

## 2. Attribution ranks (normative summary; contracts in 04)

| Rank | Condition | Identity source | Attribution content | Writes allowed |
|---|---|---|---|---|
| 1 | `current()` header validates (magic, self_ptr, install_state Installed) | W04 area header | logical id, hardware id, boot flag, lifecycle state (W03), nesting/flags from own slot | own slot; console per rules |
| 2 | Rank 1 fails; MPIDR readable and maps via the W01/W02 mechanisms | MPIDR (`MpidrValue`) | hardware id (+ logical id if topology lookup succeeds) | console only (per rules) — no per-CPU state writes |
| 3 | MPIDR unusable or lookup fails | none | explicit `unknown` marker | console only, single best-effort line |

Rank selection is evaluated per exception entry and recorded in the slot
(attribution snapshot) when rank 1 holds.

## 3. Logical modules

| Logical module | Responsibility | Inputs | Outputs | Owned state | Non-responsibility |
|---|---|---|---|---|---|
| A. Exception-local state | slot contents, begin/end bookkeeping, nesting bound | entry/exit points | per-CPU exceptional context | the slot (per CPU) | vector mechanics (P1); capture layout (P1) |
| B. Attribution | rank resolution, snapshot fields | `current()` header, MPIDR, W03 state | `CpuAttribution` value for every emission | none (derived) | topology authority (W01/W03) |
| C. Fatal integration | per-CPU fatal record, bounded console strategy, terminal behavior | P1-W07 field set, attribution | attributed fatal report; terminal state | own slot's fatal record | recovery (none at P3); crash storage (P1 out of scope) |
| D. Logging discipline | Diagnostics-lock rules, interleave markers, line atomicity | W06 rules | safe simultaneous emission | none | catalog/format authority (W11/P0-W12) |

## 4. Objects and ownership

| Object | Count | Owner | Writers | Readers |
|---|---|---|---|---|
| `ExceptionLocalSlot` | one per CPU (in `PerCpuArea`) | W09 (contents); W04 (placement) | the owning CPU only | cross-CPU diagnostics read-only (W10/W12 surfaces) |
| Fatal record | one per CPU, inside the slot | W09 | the owning CPU (once, at its fatal point) | console emitter; post-mortem diagnostics |
| Diagnostics lock | one per console | the console owner design, class Diagnostics per W06 | any emitting CPU | nobody (lock) |
| Attribution metadata | per emission | W09 rules | derived | W11 consumers |

No cross-CPU writable state exists on any exceptional path except the
console (serialized). This is the non-overlap invariant in object form.

## 5. State machine (per CPU, exceptional-path bookkeeping)

```text
ThreadContext --exception entry--> InException { depth = 1 }
InException  --recoverable-diagnostic exit--> ThreadContext { depth = 0 }
InException  --fault while handling--> DiagnosticNested { depth = 2, bound }
DiagnosticNested --exit--> ThreadContext
DiagnosticNested --fault again--> TerminalFatal (no further capture;
                                   attribution from slot snapshot)
```

- The nesting bound is `EXCEPTION_NESTING_MAX = 2` (one diagnostic
  nested level) — stage-local constant recorded with rationale; exceeding
  it is terminal by design (README decision 4).
- `depth` and the flag live in the owning CPU's slot; no other CPU ever
  writes them. A depth/flag inconsistency observed on a healthy CPU is
  slot corruption — fatal diagnostic with attribution.
- The P1 entry path integrates `exception_begin`/`exception_end` at the
  boundaries its contract defines; W09 does not redefine those
  boundaries.

## 6. Failure model

- **Fault before install / during W02 entry** (rank 2): degraded but
  present attribution; no per-CPU writes; the W02 outcome-map/mailbox
  path carries the result when applicable; the console line is
  best-effort.
- **Fault with corrupt locality** (rank 1 fails, rank 2 fails): rank 3 —
  a single attributed-unknown line; the CPU then takes the terminal
  path. Corrupt locality is never "repaired" at runtime.
- **Fatal path contention**: bounded try-lock; on failure the fallback
  emission may interleave with another CPU's output — acceptable,
  bounded, and marked; the per-CPU fatal record (already in the slot) is
  the durable source for post-mortem reading (W10/W12 surfaces).
- **Console lock corruption/loss**: the fallback path is the
  containment; a lost console is a diagnosability failure to record, not
  a hang.

## 7. Consumer integration map

| Consumer | Surface consumed | Contract point |
|---|---|---|
| W10 (audit) | non-overlap invariant; attribution-rank rules as audit criteria for P0–P2 diagnostic state | [03 §6](03-code-contracts-exception-local-state.md); [04 §2](04-code-contracts-attribution-and-logging.md) |
| W11 (observability) | attribution field set; exceptional-path emission points | [04 §2–§4](04-code-contracts-attribution-and-logging.md) |
| W12 (stress) | concurrent-fault and simultaneous-logging stimuli; per-CPU fatal records | [04 §4](04-code-contracts-attribution-and-logging.md) |
| W13 (regression) | boot/secondary exceptional-path scenarios as matrix rows | [06-validation-and-handoff.md](06-validation-and-handoff.md) §3 |
| W14/P4 (handoff) | per-CPU diagnostic foundation (attribution, bounded fatal, safe logging) | entry README handoff |
