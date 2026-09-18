# P8-W17 Measurement Method

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W17 detailed design](README.md).

## 1. Logical artifact groups

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Metric catalog (this file §2) | W17 (this design) | P8-W17 plan scope; P0-W13 namespace; P7 accounting; P6-W13 latency method (assumed contracts) | metric definitions M1–M7; it does not implement counters or define new event semantics |
| Environment and overhead contract (this file §3–§4) | W17 | W16 run-record environment block; ADR-048 configurability | what must be declared for a record to be comparable; it does not tune the host |
| Scenario binding (this file §5) | W17 | W16 matrix rows; W15 fixture; W10/W11 workload profiles | which fixed scenarios are measured; it does not create new workloads |
| Non-KPI boundary (this file §6) | W17 | plan exclusions | what observations may never be used for |
| Baseline-record schema, run procedure, comparison rules | [02-baseline-record-and-workflow.md](02-baseline-record-and-workflow.md) | this file | record structure and validity rules |

## 2. Metric catalog

Rules common to all metrics: one declared clock domain; one declared
observation point; raw per-run values retained (aggregation only in the
summary block per [02](02-baseline-record-and-workflow.md) §5); instrumentation
configuration recorded; no value is judged.

| ID | Metric | What is measured (definition) | Clock domain | Observation point | Unit / aggregation | Cannot prove |
|---|---|---|---|---|---|---|
| M1 | Boot time | elapsed harness time from runner start to each declared W09/W10 boot milestone (earlycon, kernel start, initramfs shell, SMP-complete) | host monotonic harness clock | harness, from the W16 run envelope | milliseconds per milestone per repetition; summary per [02] §5 | Guest-perceived boot speed; hardware boot time; anything about a different host |
| M2 | VM exits and reason distribution | counts of VM-exit events classified by the approved exit-reason taxonomy, accumulated per scenario run | hypervisor accounting counters (EL2 telemetry) | EL2 exit path accounting, per W16 scenario | counts per run, per reason class; no rate derived from M1 unless the derivation is stated in the record | why counts differ across environments; scheduler or guest efficiency |
| M3 | Stage-2 fault observations | counts and classes of Stage-2 faults (mapped to the W13 diagnostic taxonomy; boundary-fault rows counted separately from fault-injection rows) | hypervisor accounting counters | Stage-2 fault handling path | counts per run per class | memory-subsystem quality; absence of faults on untested paths |
| M4 | Timer/vIRQ observations | counts of timer and virtual-IRQ injections per scenario (and per vCPU where the source provides attribution) | hypervisor accounting counters | injection path accounting | counts per run | interrupt-delivery quality on hardware; latency (that is M7) |
| M5 | Scheduler switches | vCPU context-switch counts and switch-reason distribution from the P7 accounting telemetry, per scenario | hypervisor accounting counters | scheduler switch accounting | counts per run per reason | fairness, policy quality, or any ADR-057 algorithm judgment |
| M6 | Host CPU use | QEMU/hypervisor process CPU time consumed per scenario repetition, alongside M1 wall time | host process CPU accounting (declared host tool), reported next to M1 so the ratio is explicit | harness host | CPU-seconds per repetition; ratio stated as derived | EL2 efficiency on hardware; contention behavior under other load policies |
| M7 | Basic interrupt latency | physical-IRQ-to-vIRQ-injection latency samples using **the method delivered by the P6-W13 handoff contract**, re-expressed unchanged for the Linux scenarios | Guest-observed counter/time source as defined by that method | as defined by that method | raw samples retained; summary per [02] §5 | worst-case latency; hardware interrupt behavior; any real-time claim |

Failure boundaries (assumed contracts):

- **M7:** if the P6-W13 handoff delivers no method, M7 is recorded as a
  blocked prerequisite in the baseline record; W17 must not invent a
  substitute measurement, because two methods would make P6 and P8 numbers
  incomparable.
- **M2/M3/M4/M5:** if the P0-W13-governed counter namespace or the owning
  stage's accounting is not evidenced, the affected metrics are recorded as
  blocked prerequisites; no ad-hoc counter string may be introduced as an
  undeclared interface.

## 3. Environment declaration

Every baseline record embeds a complete environment declaration; a record
without it is invalid. Required fields (the W16 run-record environment block
is the minimum; W17 extends it):

```text
host_identity      host CPU model, core count, memory, hypervisor/KVM state
                   of the host, host kernel version, OS and version
host_policy        CPU frequency governor state, enabled cores policy,
                   declared concurrent-load policy ("idle" unless declared)
qemu_identity      QEMU version, acceleration/emulation mode, machine model
build_identity     hypervisor build identity per P0 version governance
fixture_ref        W15 fixture version or generation digest
machine_ref        approved machine-contract version/identity
telemetry_config   compiled-in/compiled-out/filter state of trace and metric
                   events used by M2–M5 (ADR-048 configurability disclosure)
method_version     the W17 method revision used (this file's recorded state)
```

The declaration records the environment; it never changes it. Tuning the host
(thermal management, governor changes, background services) to obtain a
different number is out of scope; if the environment cannot be declared
(e.g., unknown governor), the record states the unknown and the affected
metrics are marked limited-comparability, not silently published.

## 4. Host/platform dependence and overhead disclosure

- **Platform dependence statement (required in every record):** all M1–M7
  values are observations of one QEMU/TCG-or-declared-acceleration
  environment on one declared host. They are not predictions of RK3566 or any
  other hardware, are not comparable across acceleration modes, and are not
  comparable across hosts. Cross-record comparison is valid only under the
  [02](02-baseline-record-and-workflow.md) §6 rules.
- **Overhead disclosure (required):** the record must state which
  instrumentation was active, because ADR-048 makes high-overhead events
  removable — an M2 count from a run with full tracing and one from a
  stripped build are different observations. The record must also name any
  Guest-visible instrumentation used for M7 and its declared effect class,
  without quantifying that effect as a new measurement (that would require a
  separate declared scenario).
- **Known-limitation list (required):** each record lists the declared
  limitations: TCG translation variance, host scheduling noise, absent
  real-time properties, single-VM scope, and any blocked metric per §2.

## 5. Scenario binding

Measurements exist only for declared W16 matrix rows; a measurement outside a
row is not a baseline observation. The binding table records, per metric
group, the rows measured in the initial baseline:

| Metric group | Measured rows | Fixed conditions consumed |
|---|---|---|
| M1 | LG-B1, LG-B2, LG-B4 (per W16 repetition parameters) | W09/W10 milestones; W15 fixture; approved machine config |
| M2, M3 | LG-B1, LG-B4, LG-M4 (larger RAM class), LG-F1 (fault row counted separately) | W16 scenario configs; W13 taxonomy |
| M4 | LG-B1, LG-B2, LG-B4 | W16 rows; timer/vIRQ accounting |
| M5 | LG-SC1, LG-SC2, LG-S4 | W11 declared scheduling scenarios; P7 accounting |
| M6 | LG-B1, LG-B4, LG-S4 | W16 rows; declared host-load policy |
| M7 | LG-B1, LG-B4 (subject to the P6-W13 method's applicability) | P6-W13 method contract |

A row substitution (measuring a scenario not in the table) requires a
reviewed edit of this table with the same rigor as a metric change; silently
swapping scenarios between baseline cycles would invalidate every comparison.

## 6. Non-KPI boundary

The following are prohibited in every W17 artifact and every record that
references it:

1. pass/fail thresholds on any M1–M7 value, including "regression gates" on
   future comparisons;
2. optimization conclusions, tuning recommendations, or claims that a build
   is "fast enough" or "slower than it should be";
3. competitive comparisons against other hypervisors, QEMU configurations, or
   scheduler algorithms (the ADR-057 default-algorithm decision is explicitly
   out of reach of this data);
4. any statement transferring a value to hardware, or omitting the
   environment/limitation statement when quoting a value;
5. deriving *new* judged quantities (e.g., "exits per second budgets") not
   defined in §2.

A baseline comparison may only *describe*: which record, which change, which
declared-environment difference, which direction the raw and summary values
moved. Whether a movement matters is a decision for an authorized design, made
with its own justification — never an automatic consequence of the numbers.
