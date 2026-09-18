# P2-W09 Scenarios and Evidence Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W09 detailed design](README.md).

## 1. Scenario numbering

`W09-C<nn>`; IDs stable, revisions recorded (same policy as W08,
[01 §5](../p2-w08-host-robustness-regression/01-scope-and-foundations.md)).

## 2. Scenario matrix

Every row: executed per the configuration's repetition count
([02 §3](02-configuration-matrix.md)); every boot must satisfy the row's
pass condition.

| ID | Scenario | Input / precondition | Expected observable | Pass condition | Proves / does not prove |
|---|---|---|---|---|---|
| W09-C1 | Full-chain boot, baseline config | `virt-cpu1-memB`, P1 recipe, pinned image | Boot reaches the inspection render; intake marker; allocator/heap ready markers | All expectation rows ([02 §4](02-configuration-matrix.md)) hold in the boot log; no diagnostic above WARN-class recorded states | The P2 chain works on the reference platform; not hardware semantics, not other configs |
| W09-C2 | CPU-count variants | `virt-cpu2-memB`, `virt-cpu4-memB` | CPU inventory rows equal `-smp` per config | Per-config expectation rows hold; topology count exact | Discovery tracks the described inventory; not SMP operation (no AP starts) |
| W09-C3 | RAM-size variants | `virt-cpu1-mem512`, `virt-cpu1-mem2g-hs` | RAM bank rows match §2 dumps; map summary totals track `-m` | Bank count/totals exact; equation exact | Multi-region map at integration strength (if multi-bank pinned); not all RAM topologies |
| W09-C4 | Accounting domain | Every boot of every config | `ram = allocatable + protected` exact; protected total in `[min, max]`; allocator conservation | Holds in every boot log | Accounting stability in situ; complement to (not substitute for) W08 S308's property soak |
| W09-C5 | Repeated-boot stability | Baseline ×5, variants ×2 | Identical expected rows across boots of the same config, incl. allocator/heap totals and metadata placement | Zero drift; any drift = failed row with both renders attached | No uninitialized-state dependence on the reference platform; not a statistical proof, not hardware determinism |
| W09-C6 | Inspection active-state consistency | Every boot | W06 sections present; C1–C4 pass (no `ConsistencyBroken`) | Present and passing in every boot | W06's active-state property in situ; not W06's host evidence |
| W09-C7 | Offline/boot cross-check | Baseline boot; W07 `virt` fixture if acquired | Boot-time DTB (dumped) vs W07 expectation | Differences recorded as findings; binding-class agreement on P2-required rows (or findings recorded for W10) | Fixture fidelity; not W07's own DV rows |
| W09-C8 | Negative observation sanity (bounded) | One deliberately corrupted input per failure class is *not* run on QEMU; instead: confirm boot diagnostics for absent/invalid DTB via the runner's parameter surface if P1 exposes one, else record not-run | Distinct W01 diagnostics in boot log, or explicit not-run | Each attempted class yields its diagnostic; unattempted classes listed not-run with reason | Boundary diagnostics survive integration for attempted classes; host matrices (S1xx) remain the primary negative evidence |

Rationale for C8's shape: P2-V01/V02 negative evidence is host-side; W09
adds only what the runner surface naturally exposes and must not turn into
a QEMU fuzzing exercise (Reserved).

## 3. Execution order

C1 first (pins baseline expectations per
[02 §2](02-configuration-matrix.md)); then C2/C3 variants (×2 each); then
C5 repetitions for the baseline to complete ×5; C4/C6 are evaluated on
every boot log collected; C7/C8 last. Expectation pinning always precedes
binding: a configuration's first boot populates its expectation file, and
subsequent boots are judged against it.

## 4. Environment and capture contract

- Runs use the P0-W09 runner entry exclusively; capture (serial log to
  file), timeout, and exit-status semantics follow its documented
  conventions (assumed contract A1).
- QEMU version, host OS, image identity (hash or version string per the
  P0-W16 conventions when they exist), and configuration label are
  recorded per run.
- Serial logs are stored as evidence artifacts alongside the verification
  record; expectation files and any dumped DTBs are stored in the fixture/
  evidence area with provenance notes ([W07 §2](../p2-w07-offline-dtb-compatibility/03-fixture-and-expectation-matrix.md)
  pattern).

## 5. Evidence record

Destination: `docs/stages/p2/verification/p2-w09-qemu-integration-regression-verification.md`
(created when evidence exists; never pre-filled).

Per-run record schema (format contract W10 references):

```text
run ::=
  scenario: W09-C<nn>
  configuration: <label from 02 §3>
  boot_index: <i of N>
  status: passed | failed | blocked | not run
  command: <exact runner invocation>
  environment: <QEMU version, host OS, image identity, date/time>
  expectations: <expectation file version used>
  observed: <per-row outcome vs expectation; both renders attached on drift>
  artifacts: <serial log path, dump paths>
```

Plus a matrix summary: per-configuration status, per-scenario status,
totals, and the explicit not-run list with reasons. P2-V11 is satisfied
only when every required configuration has its full repetition count
passed and C4/C6 held on every boot.

## 6. Failure handling and revision rules

- A failed boot is re-run once in isolation to separate transient host
  issues from product findings; both runs are recorded. A confirmed
  failure is evidence against the owning package (A3) — W09 does not patch
  product code.
- Expectation files are never edited to convert a failure into a pass. A
  legitimate platform change (new QEMU version, changed P1 recipe) is a
  new dated run section with a new expectation version and a recorded
  rationale, keeping the old evidence intact (same principle as
  [W07 §7](../p2-w07-offline-dtb-compatibility/03-fixture-and-expectation-matrix.md)).
- Blocked runs (runner, P1, environment) record the upstream owner and are
  retried only after the upstream record shows resolution.
