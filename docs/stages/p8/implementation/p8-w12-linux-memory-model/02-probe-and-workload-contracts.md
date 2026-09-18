# P8-W12 Probe and Workload Contracts

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W12 detailed design](README.md).

These are normative interface obligations, not implementations. No crate,
module, file layout, or target is selected here; realization belongs to
[W16](../p8-w16-automated-linux-regression/README.md) (execution/evidence) and
[W15](../p8-w15-reproducible-linux-fixture/README.md) (Guest workload content).
Names are fixed by this design for the P8 stage; renaming requires a revision.

## 1. Contract status and stability

| Contract | Kind | Realized by | Stability |
|---|---|---|---|
| `MemoryScenarioDescriptor` | data type | W16 scenario storage | Stage-local, fixed by this design |
| `MemoryIsolationProbe` | probe record + semantic function | probe addresses by W16 from approved map facts; verdict logic in W16 harness | Stage-local; semantics fixed here |
| `evaluate_memory_isolation` | function (semantic) | W16 harness logic | Stage-local; semantics fixed here |
| `MemoryFaultObservation` + `diagnostic_sufficient` | record + function (semantic) | observation assembly from evidenced P4-W06 fields | Stage-local; feeds W13 |
| Guest memory workloads | functional requirements | W15 fixture content | Requirements fixed here; realization by W15 |

None of these is an EL2 or Guest ABI surface; none enters the
[P8-W14](../p8-w14-machine-abi-compatibility/README.md) compatibility matrix.

## 2. `MemoryScenarioDescriptor`

```text
Name and stability: MemoryScenarioDescriptor — validation-side data record;
  stage-local.
Purpose and caller: captures one declared memory-matrix experiment (01 §3–§5)
  for review, execution (W16), and evidence. Callers: W12 review steps; W16.
Inputs / outputs: not applicable (record). Fields:
  matrix_id        M12-1…M12-10 row under test
  ram_class        Small | Normal | Larger (capacity recorded separately with
                   its justification)
  layer            L1 | L2
  workload         the 02 §5 program invocation(s) or boot-only
  probes           for L2: the MemoryIsolationProbe list
  expectations     per probe: expected classification and contained outcome
  window           observation window and required counter/trace coverage
Preconditions / postconditions: validates only when every L2 probe's
  expectation is derivable from approved map facts; an un-derivable expectation
  makes the descriptor invalid (blocked), not speculative.
State and ownership change: immutable once reviewed; capacities recorded in the
  implementation record with map bounds.
Concurrency/allocation context: realization-owned.
Errors and failure guarantee: validation failures are review failures; no
  partial execution from an invalid descriptor.
Security/authorization checks: descriptors may not declare runtime map/unmap
  of Guest RAM (README decision 6) or any Host-memory exposure.
Validation: W12-DV02/DV03 review; executed instances match reviewed
  descriptors (W12-DV04/DV05).
```

## 3. `MemoryIsolationProbe` and `evaluate_memory_isolation`

```text
Name and stability: MemoryIsolationProbe — record + semantic evaluation fixed
  by this design; realized in W16's harness design.
Purpose and caller: define and judge one L2 boundary/negative case. Caller:
  W16 harness; W12 review steps.
Inputs / outputs:
  probe record — probe_kind (N1|N2|N3|N4|N5), the Guest-side action shape
                 (declared IPA offsets/patterns relative to approved map
                 facts), access type, and expected classification
  evaluation   — inputs: the probe record, the observed Stage-2 event (as a
                 MemoryFaultObservation), the console/console-health evidence,
                 and post-run ownership query results
               — output: ProbeVerdict { ExpectedFaultObserved |
                 WrongOutcome(observed_class, evidence_ref) | Blocked(reason),
                 contained: bool, context_sufficient: bool }
Preconditions / postconditions: total per probe; a missing Stage-2 event where
  a fault is expected is WrongOutcome, never a pass; Blocked requires a stated
  reason (missing mechanism/contract).
State and ownership change: none; read-only over observations and ownership
  queries. W12 performs no mapping operation (README decision 6).
Concurrency/allocation context: realization-owned; evaluation never mutates
  hypervisor state.
Errors and failure guarantee: ambiguous or incomplete observations yield
  WrongOutcome/Blocked with evidence refs; no interpretation defaults.
Security/authorization checks: probe addresses derive from approved map facts
  recorded in the implementation record; Host-owned ranges are named only as
  exclusion expectations, never as Guest-accessible targets; probe outcomes
  are Guest-produced observations, not trusted control input.
Logic (outline, not production code):
  run the declared Guest action on the booted Linux (via fixture probe);
  gather the Stage-2 event, classification context, and console health;
  compare observed classification with the probe expectation;
  query ownership state post-run for the single-owner/bounds checks;
  return the verdict with evidence refs.
Validation: W12-DV05; reviewed expectations (W12-DV02).
```

## 4. `MemoryFaultObservation` and `diagnostic_sufficient`

```text
Name and stability: MemoryFaultObservation — record of one observed Stage-2
  event; diagnostic_sufficient — predicate fixed by this design, aligned with
  the W13 minimum-context contract (see ../p8-w13-guest-fault-diagnostics/README.md).
Purpose and caller: make "actionable Stage-2 diagnostics" (P8-V17) checkable
  per event. Callers: W12 evaluation; W13 taxonomy intake; W16 assertions.
Inputs / outputs:
  observation fields — vm_id, vcpu_id, fault classification (translation /
    permission / other per P4-W06), IPA, access type, syndrome fields, mapping
    query result (mapped/unmapped/protected), exit reason, vCPU state, trace
    window ref, build/version identity (P0-W12)
  predicate output   — true when the observation carries every field the W13
    minimum-context contract requires for its class, with no Host physical
    address in any Guest-visible portion
Preconditions / postconditions: fields are populated from evidenced P4-W06
  diagnostic paths only; the predicate never substitutes defaults.
State and ownership change: none.
Concurrency/allocation context: realization-owned.
Errors and failure guarantee: insufficient context is a recorded finding that
  fails the row's "actionable" clause; it is never silently tolerated.
Security/authorization checks: host-leakage check is part of sufficiency (01 §6).
Validation: W12-DV05 context sufficiency; feeds W13-DV04.
```

## 5. Guest memory workload functional requirements (fixture needs)

W15 realizes these in the pinned fixture; W16 invokes them per descriptor.
Marker strings are fixed here because W16's expected markers must be stable.

| Program | Invocation shape | Functional requirement | Stable marker (prefix) |
|---|---|---|---|
| `mem-exercise` | `mem-exercise <mb> <passes>` | Allocates, writes, verifies, and frees the declared working set for the declared passes; writes one marker per pass with pass result; exercises allocator and Guest page-table structure; stays within its declared working set | `MEM-MARKER pass` |
| `fork-storm` | `fork-storm <children> <iterations>` | Forks the declared number of children (COW + kernel/user switching), each performing declared trivial iterations; parent joins and writes a completion marker | `MEM-MARKER fork` |
| `map-probe` | `map-probe <probe-id>` | Performs the single Guest-side access pattern declared for probe kind N1/N2/N3/N4/N5 as recorded in the implementation record; writes one marker naming the probe id before the access so the transcript correlates with the Stage-2 event | `MEM-PROBE start` |

Requirements and constraints:

- Parameters come only from the command line; probe addresses are recorded in
  the implementation record derived from approved map facts — they are never
  hard-coded in the fixture.
- `map-probe` deliberately causes Linux to fault; the expected Linux-visible
  consequence (kernel abort/panic) is part of the transcript and must be
  captured by the W09 console path. The fixture must not suppress or handle the
  fault in a way that hides the Stage-2 event.
- Workloads are memory/process tools only; no device access, no hypercalls, no
  IRQ manipulation (those belong to W18's security scenarios).
- Marker and elapsed formats are machine-readable and fixed in the W15
  manifest so W16 parsing does not guess.

Validation: W12-DV03 (fixture-consistency review with W15); W12-DV04/DV05
(markers and probes observed as declared during execution).
