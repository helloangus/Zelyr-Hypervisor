# P6-W01 Architecture, Objects, and Decision Lifecycle

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W01 detailed design](README.md).

## 1. Logical modules

W01 is code-bearing but hardware-free: three logical modules, all host-testable
in principle and placeable by the implementing agent in the crate layer the
P0–P5 designs establish. Placement rule: capability *types and reconciliation*
are platform-independent Core/Arch-shared knowledge (no board, no SoC name,
no register access); fact *instances* are platform inputs. If the established
crate tree differs from the names below, the placement maps onto the owning
crate by layer; the module contracts, not file paths, are the design.

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `gic-capability-model` | Typed capability inputs, verdict taxonomy, graded decision; the sole vocabulary for GIC capability findings in P6 | None (value types) | P1/P2 fact types (assumed contracts) | `GicCapabilityDecision` | Does not validate DTB encoding, does not touch hardware, does not decide bring-up sequencing |
| `gic-capability-reconcile` | Pure reconciliation of the capability input register ([01](01-scope-and-foundations.md) §3) into findings and a decision; expected-identity derivation for W02 | None | fact instances, posture constants | findings, decision, `ExpectedGicIdentity` | Does not repair or default missing facts; does not interpret register bits beyond identity fields |
| `gic-capability-report` | Emits the decision as the W01 record artifact and as trace events under the P0 telemetry namespace contract | The emitted report buffer | `GicCapabilityDecision` | capability report record; telemetry events | Does not gate boot itself; the boot-time *reaction* to a decision belongs to the initialization owner (P1/P3 bring-up sequencing and W02) |

## 2. Core objects and ownership

### 2.1 Fact objects (inputs, read-only)

`PlatformGicFacts` (from P2) carries: distributor frame descriptor;
Redistributor frame list (base, stride, affinity-keyed or positional) or an
explicit insufficient-coverage marker; declared GIC version indication;
interrupt-range facts (declared device SPI usage by P6-relevant sources,
maintenance PPI, timer PPIs); reservation ranges; firmware security-state
hints. `CpuInterruptFacts` (from P1) carries the CPU-side GIC CPU-interface
form and virtualization-extension indication. `PossiblePcpuSet` (from P2+P3)
carries the affinity values P3 may bring online. All are owned by their
upstream producers; W01 takes shared references and never mutates or caches
them beyond the reconciliation call.

### 2.2 Decision objects (outputs)

`CapabilityFinding` — one immutable value per input: input ID, verdict, the
observed fact (or its absence), and a bounded reason string/enum.  
`GicCapabilityDecision` — the immutable aggregate: findings vector, overall
grade (`ReadyForP6` | `PhysicalOnly` | `Rejected`), and
`ExpectedGicIdentity` when physical bring-up is permitted.  
`ExpectedGicIdentity` — the facts W02 must confirm by probe: family, ARE
requirement, SGI/PPI fixed range, declared SPI maximum, Redistributor stride
expectation, maintenance PPI, timer PPIs, security-state expectation.

Ownership rule: facts are owned upstream; decisions are value objects owned
by their consumer after the handoff (W02 stores the decision it was started
with; no shared mutable decision state exists). No registry, no global, no
lock is authorized — there is no concurrency in W01's model because
reconciliation is a pure function evaluated at a point where its inputs are
already stable (boot-time discovery completion per P2).

## 3. Decision lifecycle

```text
no decision
  -> facts assembled (P2/P1/P3 producers, each already evidenced upstream)
  -> reconcile() evaluated once at boot (host-side fixture evaluation in tests)
  -> decision {grade, findings, expected identity}
       grade = Rejected      -> initialization owner stops interrupt work;
                                diagnostic + escalation record; no W02 start
       grade = PhysicalOnly  -> W02/W03/W04 may proceed after confirmation;
                                W08+ blocked with recorded stage block
       grade = ReadyForP6    -> W02 confirmation may proceed
  -> W02 executes probe contracts; probe mismatch folds back to
     Unsupported/Contradictory findings and the same escalation table
  -> decision archived in the W01 record; P6-V01 evidence derives from it
```

The lifecycle is once-per-boot, append-after-evaluation: findings are never
mutated after the decision is produced. A platform hotplug or re-discovery
event is out of P6 scope (P3 reserves hotplug; no P6 consumer exists).

## 4. Concurrency and allocation model

Reconciliation runs to completion before interrupt hardware is enabled and
before secondary CPUs are released from the boot rendezvous (P3-W05
ordering), so no lock is designed. Allocation: the findings collection is
bounded by the fixed input register ([01](01-scope-and-foundations.md) §3 has
a constant row count); implementers may use a fixed-capacity collection with
a compile-time bound, or dynamic allocation if and where the P2 allocator
contract permits at that boot phase. Both are acceptable; the decision must
not depend on which is chosen. Report emission may allocate only through the
established boot-phase allocator contract.

## 5. Failure and destruction behavior

There is no runtime failure surface (no hardware, no I/O): failures are
*findings*. Destruction/recovery does not apply; the decision object is
immutable and lives as long as its consumer needs it. The only "recovery" is
re-evaluation after an upstream fix, which is a rebuild/reboot event, not a
runtime path.

## 6. Security model

No Guest input exists in W01. The security-relevant rules are:

- upstream facts are validated against architectural ranges before use
  (`Malformed` handling), because platform description input is treated as
  untrusted-adjacent (ADR-007's boundary discipline applied to firmware data);
- the posture never downgrades a `Rejected` to usable;
- `Contradictory` never resolves by authority-of-order (first source wins);
  it fails closed.

## 7. Telemetry

Three trace-event kinds, registered under the P0 trace-event namespace
(assumed contract from
[p0-w13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md) and
consumed by W13): `capability_decision` (grade + input-row bitmask),
`capability_finding` (input ID + verdict, rate-limited to one event per
distinct input), `capability_escalation` (verdict classes that require a
record). Payloads carry no board names — the board identity, if any, lives in
platform fact instances upstream.
