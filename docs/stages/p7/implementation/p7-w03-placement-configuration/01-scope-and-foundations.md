# P7-W03 Scope, Foundations, and Prerequisite Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W03 detailed design](README.md).

## 1. Foundation analysis

For the plan goal ("static pinned equivalence and explicit affinity,
pinning, dedicated/shared, and invalid-control semantics") to be true, these
concrete artifacts must exist:

1. A bounded, typed pCPU-set representation (`CpuSet`) — semantics cannot
   be "explicit" while the container is undefined.
2. A total validation function over all placement inputs with a closed
   error set — "non-silent rejection" requires every invalid input class to
   have a named outcome.
3. A frozen per-vCPU resolved placement plus an authoritative ledger —
   consumers (gate, picker) need one place to ask "where may this vCPU run".
4. A pure eligibility predicate usable by W02's gate and W05's picker — the
   same rule must govern admission and queueing or the matrix (P7-V06)
   becomes unenforceable.
5. A pinned-mode behavioral statement tied to the P4 baseline — equivalence
   needs a comparison basis.
6. Semantic telemetry fields for applied/rejected configuration.
7. An authorization seam upstream of validation.

Items 1–7 are this design's foundation deliverables; none exists today
(repository is a documentation scaffold).

## 2. Prerequisite contracts and failure boundaries

| Prerequisite | Delivering package / plan path | What W03 assumes | Failure boundary |
|---|---|---|---|
| Platform CPU topology and capacity facts | P2-W10 (`../../../../stages/p2/plans/p2-w10-p3-p4-handoff-contract.md`); P7-IN-03 | a platform CPU maximum usable as the `CpuSet` capacity constant; stable logical pCPU ids | capacity/id model absent or unstable → W03 blocked at `CpuSet`; record per W01 §5 |
| pCPU registry states | P3-W03 + P3-W14 (`../../../../stages/p3/plans/p3-w14-p4-smp-handoff.md`); P7-IN-04 | states `Online` / not-`Online` distinguishable; `Failed` pCPUs never become dispatch targets | registry semantics differ → eligibility rule re-derived via ACR; W05/W08 consumers blocked |
| Lifecycle authority | P7-W02 design (`../p7-w02-scheduler-admission-lifecycle/README.md`) | placement attaches only at `Offline`; gate consumes `is_eligible` | if W02's accepted gate differs, the attach point is renegotiated in design, not in code |
| Capability rights model | P5-W10 (`../../../../stages/p5/plans/p5-w10-closeout-p6-handoff.md`); P7-IN-06 | a rights check exists whose rights can express "scheduling-policy control" | no suitable right → prerequisite mismatch recorded; W03 does not mint rights |
| Allocatable memory for scheduler objects | P2-W10; P7-IN-03 | ledger and sets allocate through the P2 allocator contract | allocator mismatch → configuration path blocked; no static-array bypass invented |

## 3. Itemized scope classification

**Required:** R1 `CpuSet` (capacity from topology; total operations); R2
`PlacementMode` + `PlacementSpec` + `ResolvedPlacement`; R3
`validate_placement` with the closed error set
(`EmptyAffinity`, `UnknownPcpu`, `PcpuNotOnline`, `CapacityExceeded`,
`ExclusiveConflict`, `DuplicateConfiguration`, `ReconfigurationNotSupported`,
`NotAuthorized`); R4 placement ledger with O(1) exclusivity query; R5
`is_eligible` pure predicate; R6 static-pinned equivalence statement + P4
baseline comparison basis; R7 telemetry semantics (`placement_applied`,
`placement_rejected`); R8 authorization hook.

**Reserved:** dynamic reconfiguration (weights, masks, migration);
NUMA-aware placement; hotplug reaction beyond the online re-check; quota/
share semantics; RT classes (ADR-017); wire/config encodings of placement.

**Out of Scope:** configuration ABI/Control Domain interface; runqueue and
pick policy (W05); pause/stop flows (W07); remote reschedule/idle (W08);
counter/trace encoding (W09); guest workloads (W10); stress (W11); any
architecture-register or timer access from placement code.

## 4. Mechanism vs policy split

| Concern | Classification | Owner |
|---|---|---|
| `CpuSet`, validation, ledger, eligibility | Mechanism (policy-free placement authority) | W03 |
| Pinned/shared semantics and exclusivity rule | Policy shape fixed by P7 stage scope | W03 (this design) |
| Default placement when unspecified | Policy | W03 (decision 4 in the contracts file: all-online shared set) |
| Which of the eligible pCPUs a woken vCPU lands on | Policy | W05 |
| Load balancing / migration / weights | Reserved policy | later stages |
| Who may configure placement | Authorization | P5 rights model via the W03 hook |
| Reconfiguration while active | Reserved; rejected explicitly | later stage |

## 5. ADR-016 conformance statement

Pinned, affinity, dedicated, and shared are modes of the single vCPU
placement object; no `VmKind` or vCPU subtype exists; the scheduler type
system cannot distinguish a "pinned VM" from a "shared VM" except through
the resolved placement contents. This is the reviewable form of "static
binding is scheduler policy": the binding lives in configuration data
validated by this module, never in type-level or code-path splits.
