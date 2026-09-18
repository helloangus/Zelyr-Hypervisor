# P4-W06 Architecture, Objects, and State Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W06 detailed design](README.md).

## 1. Logical modules

Placement rule: fault diagnosis interprets architecture syndromes and
Stage-2 fault semantics, so the decode module is Arch-layer; the expectation
and report modules use only Core-visible vocabulary. Crate/file placement
follows the P0 workspace design (assumed contract M8 in W02's foundations);
this design fixes logical modules and boundaries, not file paths.

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `exit-decode` (Arch) | ESR decode (`EsrView`), access-type and fault-status-family extraction, faulting-IPA reconstruction from FAR/HPFAR | none (pure functions) | `GuestExitFrame` | decoded detail values | action decisions (W04), mapping state (W02) |
| `fault-diag` | The `ExitDiagnostic` record: assembles decode output, W02 `query` snapshot, verdict, and retained raw values | none persistent (value produced per exit; retained by the stop path) | frame, `QueryResult` | `ExitDiagnostic` | changing actions, mutating the vCPU or space |
| `isolation-expect` | The IS-series expectation matrix, expectation values, and the matcher producing `MatchVerdict` | the matrix (static, versioned with the VG table) | `ExitDiagnostic`, expectation id | `MatchVerdict` | scenario bodies (W05), episode sequencing (W07) |
| `diag-report` | Bounded human-readable report and the ledger-derived mapping dump into caller buffers | none (writes caller buffers) | `ExitDiagnostic`, space (for dump) | formatted text, dump text | event emission itself (routing via the P0 baseline), persistence (none in P4) |

Layering check: `exit-decode` is the only module naming architecture
syndrome layouts; `fault-diag`, `isolation-expect`, and `diag-report` use
Core-visible types (`GuestPhysAddr`, `QueryResult`, `S2Flags`, W04's
`ExitClass`) and never descriptor bits or sysreg names. No board, SoC, or
QEMU name appears in any module (W01 A7); QEMU divergences become
Specification Investigation records.

## 2. Core objects and ownership

### 2.1 `ExitDiagnostic` (value, module `fault-diag`)

- **Owned content:** fault domain, W04 `ExitClass`, raw ESR, `EsrView`
  decode, access type, fault-status family, IPA reconstruction result, Guest
  PC/PSTATE, vCPU identity (from the run context), `QueryResult` snapshot for
  the faulting IPA, mapping-agreement verdict, expectation verdict (if an
  expectation was declared), and the stop cause W04 already chose.
- **Immutable after production.** The producer is the exit handler's diagnosis
  step; consumers are the report, the run record (W07), and post-stop
  post-mortem reading.
- **Not owned:** the exit frame (pCPU-owned per W04), the mapping ledger
  (W02), the stop decision (W04).
- **Retention:** the last `ExitDiagnostic` of an episode is retained
  alongside W04's retained last exit frame until overwritten by a later
  episode (same retention duty and overwrite rule W04 documents for frames).
- The type carries no authority: it is data for humans and for the expectation
  matcher; nothing executes from it and no address in it is dereferenced as a
  Host address (Coding Guidelines untrusted-data rule).

### 2.2 `IsolationExpectation` and the matrix (module `isolation-expect`)

- Static, versioned table (version `IS-T1`, bumped together with the VG table
  it refines, per open item O2). Each entry: expectation id, referenced VG
  scenario id, probe class, expected W04 class, expected access type, IPA
  predicate, expected mapping state, and the post-stop EL2-liveness flag.
- **Owner:** W06. **Consumers:** W07 (episode verdicts), W08 (automation
  expectations). A change that alters a W05-observable outcome goes through
  the W05 §6 joint-change rule first.

### 2.3 No new persistent state

W06 creates no registry, no global, and no background task. Its entire
contribution is value computation at exits plus static expectation data.
"State" in this package reduces to: the retained last diagnostic (owned by
the same retention duty as W04's frame) and the static matrix. This is
deliberate: diagnosis is derived data, never an authority (Plan Agent
guardrail: registries are not implicit owners).

## 3. Classification flow

```text
Guest exit (W04 path)
  1. stub captures frame, restores host state            [W04, unchanged]
  2. classify_minimal + decide -> action                 [W04, unchanged]
  3. if action == Stop(cause):
       a. vcpu state -> Stopped(cause)                   [W04]
       b. diagnose(frame, space) -> ExitDiagnostic       [W06, this design]
       c. match declared expectation -> MatchVerdict     [W06]
       d. emit diag.* events; format report; dump        [W06, after stop]
  4. if action == Reenter: no W06 diagnosis is required   [W06 stays silent;
       the planned re-entry scenario's episodes are diagnosed when they stop]
```

Rules:

- Diagnosis never runs before the stop decision (D7): the ordering is
  containment first, then explanation.
- The re-entry path (W04 `Reenter`) intentionally produces no
  `ExitDiagnostic`; P4's only re-entry producer is the planned VG-009
  sequence, and each of its terminating episodes is diagnosed when it stops.
  If a future design wants per-episode diagnostics on re-entry, that is a
  W07/W06 joint extension, not a silent change.
- Every produced `ExitDiagnostic` is emitted as structured events and rendered
  once; rendering is idempotent (pure formatting), so post-stop tooling can
  re-render from the retained value.

## 4. Fault-domain model (Guest vs Hypervisor)

```text
fault origin                    domain        handling path
--------------------------------------------------------------------------
Guest EL1 behavior captured
  by the Guest-exit stub        Guest         W06 diagnosis -> contained,
                                              VM-facing (already Stopped)
EL2-context fault (stub, host
  code, P1 vector path)         Hypervisor    P1-W07 fatal path; W06 is not
                                              invoked and never re-labels it
```

- The domain distinction is structural (capture path), not heuristic (D3).
  `ExitDiagnostic.domain` is `Guest` by construction for every value produced
  from a `GuestExitFrame`; the field exists so the record is self-describing
  and so a future multi-origin diagnostic design has the slot.
- The reverse direction — proving no Guest fault can reach the host path — is
  W04's containment property (non-recursive capture; faults during capture
  terminate in the P1 fatal path). W06 contributes the standing review row
  (DV09) rather than new code: if a Guest-caused condition ever appears in
  the fatal path, the P4-V09 containment review fails.
- Escalation split: Guest domain → `Stop(GuestFault(...))` outcomes only
  (never fatal); Hypervisor domain → the P0 failure-classification fatal path
  (M5). The `MappingMismatch` verdict (D5) is the one Guest-domain event that
  *also* raises an invariant-review item: the Guest access is contained as
  usual, but the ledger/hardware disagreement is recorded as failed isolation
  evidence rather than a passing diagnosis.

## 5. Concurrency and context model

- Diagnosis runs in the exit handler's context: after the stub restored host
  state, with host interrupts still masked per W04's run-segment policy, on
  the pCPU that took the exit. It performs no locking of its own; the W02
  `query` call takes the space lock internally per W02 D5/D9 (shared read,
  bounded duration).
- The stop decision has already been made, so no diagnostic step can extend
  Guest execution: total diagnosis work is bounded (constant-time decode +
  one ledger lookup); report and dump run after the stop with bounded buffers
  and a bounded ledger size (P4's one Guest, one space).
- No allocation: all W06 functions write into caller-provided fixed buffers
  or return fixed-size values; heap use is prohibited in this path (M5-era
  allocator rules; Coding Guidelines).
- Re-entrancy: impossible in P4 (interrupt-masked segment; the stop path runs
  once per episode). A second concurrent diagnosis attempt would be a host
  invariant violation and escalates per M5, like W04's double-entry rule.

## 6. Isolation expectation model (IS-series, version `IS-T1`)

Each row refines the host-side observable of the referenced VG scenario
(provenance and joint-change rules per [01 §5](01-scope-and-foundations.md)
O2). "Probe class" asserts where the trigger address lies; membership is
checked by W06 against the W03 layout record and P2-derived platform facts,
never against QEMU folklore.

| ID | VG ref | Probe class / trigger | Expected W04 class | Expected access | IPA predicate | Expected mapping state | Proves / does not prove |
|---|---|---|---|---|---|---|---|
| IS-01 | VG-004 | `UnmappedGap` (in-RAM-declared gap IPA, unmapped) | `Stage2Translation` | Read | equal to the boot-info probe address | `Unmapped` | an unmapped access faults and is diagnosed; not that every unmapped address is probed |
| IS-02 | VG-004 | `RamBoundary` (first page past `ram_base + ram_size`) | `Stage2Translation` | Read | equal to the boundary probe | `Unmapped` | Guest-RAM bounds are Stage-2-enforced; not general memory-model completeness |
| IS-03 | VG-004 | `HypervisorOwned` (IPA whose identity HPA lies in a P2-protected/Hypervisor range) | `Stage2Translation` | Read | equal to the hypervisor probe; membership asserted from P2 facts | `Unmapped` | Hypervisor-owned ranges are never guest-mapped and accesses stay contained with EL2 live; not DMA/IOMMU isolation |
| IS-04 | VG-005 | read-only window store | `Stage2Permission` | Write | inside the RO window | `Mapped` with `S2Access::Read`, `ExecuteNever` | write enforcement is immediate and diagnosed; not all permission corner cases |
| IS-05 | VG-006 | XN data-window execute | `Stage2Permission` | Execute | inside the XN window | `Mapped` with `S2Execute::ExecuteNever` | execute vs write permission faults are distinguishable; not instruction-set behavior |
| IS-06 | VG-010 | architecture-guaranteed illegal instruction | `IllegalExecution` | n/a (non-memory) | n/a | n/a | controlled illegal behavior stays VM-facing and diagnosable; not a full illegal-opcode taxonomy |
| IS-07 | VG-011 | routed unknown synchronous exception | `UnknownSync` | n/a | n/a | n/a | unknown synchronous conditions retain the full raw syndrome; class expectations follow the delivered W04 routing (joint review if routing diverges) |
| IS-08 | VG-007/VG-008/VG-012 | WFI (planned: WFE) | `Wfi` (`Wfe`) | n/a | n/a | n/a | defined diagnosable results for wait instructions; not timer/scheduler semantics |

Common postcondition for every row: after the stop, EL2 remains live and
diagnosable (console, logging, and the run record continue — the W07 run
record carries the verdict), which is the P4-V07 "EL2 diagnostically live"
condition. Matcher mechanics are in
[04 §3](04-code-contracts-diagnostics-isolation.md).

## 7. Telemetry points (W06-scope)

Routed through the P0 baseline (W01 A8), stage-local names recorded as
implemented facts by W09: `diag.exit` (per diagnosed exit: class, Guest PC),
`diag.fault` (translation/permission detail: access, IPA, mapping agreement),
`diag.isolation.verdict` (expectation id, verdict), `diag.report` (report
rendered, byte length), `diag.dump` (dump rendered, entries). Events carry
only addresses and classifications already required for diagnosis — never
Guest data content. W07 consumes the events for counters and fault
correlation; W08 consumes verdicts via the run record.
