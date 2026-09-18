# P6-W12 Robustness Case Matrix

**Status:** Proposed detailed design; implementation and validation not
claimed.  
**Parent:** [P6-W12 detailed design](README.md).  
**Audience:** load before running or reviewing any case. Containment classes
follow the P0-W14 classification
([plan](../../../p0/plans/p0-w14-panic-failure-classification.md)); the
authorization boundary is
[P5-W02](../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md) /
[P5-W05](../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md)
as handed over by [P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md).

## 1. Containment classes

Every case outcome maps to exactly one class:

| Class | Meaning | Host continues | Examples |
|---|---|---|---|
| **GuestFault-local** | Guest-caused; structured VM-local error; other VMs and Host unaffected | yes | rejected invalid vIRQ operation; malformed control input |
| **Diagnostic-contain** | hardware/specification anomaly or unclaimed event; counted, dumped, contained | yes | spurious IRQ; unrecognized maintenance pattern (W09 E3) |
| **Fatal-invariant** | hypervisor state-integrity loss; escalated with full diagnostics per P0-W14 and the P1-W07 crash-diagnostics contract | no (controlled fatal path) | orphan completion (W09 E2); contradictory vIRQ state |

Rules: no case may silently repair state (parent README decision 6); a
GuestFault-local outcome must never escalate to fatal; a diagnostic-contain
outcome that recurs persistently becomes an investigation, not a runtime
escalation, unless state integrity is in question (then it is fatal-invariant
material).

## 2. Case matrix

Legend per case: **Input/precondition**, **Injection point** (test-only,
gated per parent README decision 4), **Expected observable**, **Pass
condition**, **Proves / does not prove**, **Repetition**, **Evidence** (all:
the W12 verification record, artifacts linked per the P0-W09 conventions).

### FI-A — invalid virtual-IRQ operations → P6-V21

Sub-cases (each a row in evidence):

| ID | Invalid dimension | Input | Expected | Pass condition |
|---|---|---|---|---|
| FI-A1 | range | vIRQ identifier outside the P6 namespace (e.g., below 32, above the declared ceiling, reserved class) | structured rejection via the P5 error boundary; no state change | rejection observed; Host and other-VM state unchanged |
| FI-A2 | priority/band | operation requesting a priority outside the W10 band encoding | structured rejection | as FI-A1 |
| FI-A3 | target | operation targeting a nonexistent or offline vCPU identity | structured rejection | as FI-A1 |
| FI-A4 | owner | operation on a vIRQ owned by another VM (cross-VM probe using the W11 observation asset) | permission rejection; zero observable effect in the other VM | rejection in the acting VM; other VM's records show nothing |
| FI-A5 | dead object | operation via a stale/destroyed vCPU or VM handle (generation mismatch per the P5 handle model) | structured rejection | as FI-A1 |
| FI-A6 | insufficient right | operation exceeding the caller's rights bitset | structured rejection | as FI-A1 |
| FI-A7 | malformed input | truncated/oversized control input on the authorized path | structured rejection (or safe truncation per the P5 contract) | as FI-A1 |

- **Injection point:** the P5-authorized Guest control path with fuzzed/
  crafted inputs; cross-VM probes use two VMs where the P4 isolation
  contract permits, else BLOCKED.
- **Proves / does not prove:** the implemented authorization boundary rejects
  these classes without Host/other-VM effect in QEMU; not fuzz-discovery
  completeness, not production DoS resistance, not real-hardware behavior.
- **Repetition:** each sub-case N ≥ 3; malformed-input sub-case also runs a
  short generated-input pass within declared bounds (this is the only
  fuzzing W12 declares; a full fuzz platform stays out of scope).
- **Evidence:** rejection records + Host state-unchanged probes.

### FI-B — spurious and unknown Host IRQs → P6-V20

| ID | Condition | Input/precondition | Expected observable | Pass condition |
|---|---|---|---|---|
| FI-B1 | spurious INTID class | W03 lifecycle implemented; declared spurious condition induced via the W03-designed test hook | classified as spurious; counted; safe completion; no Guest injection | no handler/index misuse; Guest observes nothing; Host continues |
| FI-B2 | unknown/unclaimed INTID | INTID outside the handled set, induced via the W03 hook | classified as unknown; counted; contained per the W03 contract | as FI-B1 |
| FI-B3 | Guest cannot inject physical IRQs | attempt via the authorized control path | rejection (GuestFault-local) | no physical IRQ state change |

- **Injection point:** W03-owned test hooks; W12 adds no dispatcher changes.
- **Proves / does not prove:** safe classification/completion of exercised
  spurious/unknown classes in QEMU; not coverage of real interrupt-controller
  misbehavior modes.
- **Repetition:** N ≥ 3 per row. **Evidence:** Host counters + Guest
  null-observation records.

### FI-C — impossible internal states → P6-V20/V21 support (visibility requirement)

| ID | Condition | Injection point | Expected observable | Pass condition |
|---|---|---|---|---|
| FI-C1 | orphan completion (W09 E2) | forced-decode/state hook on the maintenance path | counted, per-vCPU presentation snapshot, escalation on the fatal-invariant path | escalation observed with full diagnostics; no silent absorption; controlled fatal path per P0-W14 |
| FI-C2 | contradictory vIRQ state | induced-state hook on the W07 lifecycle | detection with dump; escalation | as FI-C1 |
| FI-C3 | unrecognized maintenance pattern (W09 E3) | forced-decode hook | diagnostic-contain; counters; no lifecycle mutation | contained; recurring pattern recorded as investigation |

- **Proves / does not prove:** that the written detection/escalation paths
  work as designed; not that all impossible states are enumerated (the
  matrix names the classes the designs own; undiscovered states are covered
  only by the general invariant reviews).
- **Repetition:** N ≥ 2 per row. **Evidence:** escalation dumps and counters
  (dumps land in verification artifacts, sized and redacted per the
  P1-W07 diagnostic contract).

### FI-D — bounded storm smoke → P6-V22

| ID | Stress axis | Input/precondition | Expected observable | Pass condition |
|---|---|---|---|---|
| FI-D1 | vIRQ injection rate | declared injection rate/duration over the W07/W08/W09 paths | all accounting consistent; no unexplained loss; no hang | declared run completes; state invariants hold (I1–I5 spot checks); Guest completes what was injected within the declared bound |
| FI-D2 | timer rate | declared timer rearm rate on the W05/W06 paths | as FI-D1 | as FI-D1 |
| FI-D3 | SGI rate | declared SGI rate on the W04 path | as FI-D1 | as FI-D1 |

- **Limits:** rates, durations, and counts are **declared at run time** from
  the environment and recorded in evidence; the declaration (not a hardcoded
  constant) is the contract-relevant fact.
- **Proves / does not prove:** absence of observed corruption/unexplained
  hang within the declared smoke limits in QEMU; **does not prove production
  DoS resistance** (task book P6-V22 wording, carried verbatim into every
  handoff).
- **Repetition:** N ≥ 2 per axis. **Evidence:** run summaries, invariant
  spot-check results, telemetry correlation.

## 3. Boundary statements for handoff (verbatim-carry rules)

Every W12 handoff and record must carry, unmodified in meaning:

1. "Guest-caused errors remain local to the offending VM context" — scoped
   to the exercised cases and the declared QEMU environment.
2. "Impossible-state failures are visible rather than silently corrupting"
   — scoped to the FI-C classes designed by P6.
3. "Storm smoke within declared limits does not prove production DoS
   resistance" — always attached to P6-V22 statements.
4. "QEMU results do not prove real-hardware fault behavior" — attached to
   every FI row's downstream citation.

## 4. Non-freeze statement

The case IDs, limits, hooks, and probe forms are P6 validation assets. They
freeze no error-code ABI, no diagnostic format, no host-interface, and no
P7/P8 test contract; later approved designs may reshape them without
versioning.
