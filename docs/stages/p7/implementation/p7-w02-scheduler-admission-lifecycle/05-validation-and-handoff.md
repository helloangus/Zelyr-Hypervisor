# P7-W02 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W02 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence and objective conditions; none is claimed to
have run. Requirement ids map to the P7 task book §6.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W02-DV01 → P7-V02 | W01 input/register precondition review | read W01 register rows P7-IN-01…05; confirm none blocked for W02 | every consumed input is contract-mapped or available; blocked rows affecting W02 have linked issues | the assumed-contract basis is explicit; not that predecessors are implemented |
| W02-DV02 → P7-V02 | admission-boundary review + integration test | code review of gate/exit/bypass against [contracts](03-code-contracts-lifecycle.md) §2; QEMU: scheduler-controlled entry for the validation Guest workload | normal entry occurs only via `admit_for_entry`; returns pass `complete_exit`; bypass instances enumerated against E1–E3; post-activation bypass use detectable | normal execution is scheduler-controlled on target; not preemption, policy, or multi-vCPU behavior |
| W02-DV03 → P7-V03 | transition-table and engine tests | host: exhaustive 63-pair table test; per-error-variant engine tests; IRQ-context boundedness tests; then QEMU negative transitions through control paths | every legal transition succeeds, every illegal pair is typed-rejected with state unchanged; rejection never panics | transition validity is objective and enforced; not that all producers use the engine in every flow yet (covered by DV02/DV04 scopes) |
| W02-DV04 → P7-V04 | invariant/property evidence | host property tests with random legal event sequences; seeded-violation detection for INV-1…INV-5; QEMU sampled checker under single- and multi-vCPU bring-up workloads | checker reports all-Pass on legal runs; seeded violations are detected; on target, no double-run and slot/state agreement hold across sampled windows | the invariants are checkable and hold under tested schedules; not starvation-freedom, fairness, or stress-envelope correctness (W11) |
| W02-DV05 → P7-V04 | accounting-monotonicity seam review | review INV-5 rendering seam with W09 contracts | audit-visible counters never regress in tests; no double-count on repeated lifecycles | monotonicity is observable; full counter semantics are W09 evidence |
| W02-DV06 → P7-V02/V03 | failure-classification review | walk `Fault` sourcing against the P5 taxonomy; verify hypervisor-invariant cases never produce `Fault` | classification seam matches P4-W09/P5-W10 assumed contracts; mismatch recorded, not absorbed | containment terminology is coherent; runtime fault-containment behavior is W07 evidence |
| W02-DV07 → W02 closure | consumer consumability review | read the design as W03 (can I attach placement?), W04 (can I deschedule?), W06 (can I block/wake?), W07 (can I pause/stop?), W09 (are semantics renderable?) | each consumer finds its seam named and its frozen invariants listed | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command/input, reviewer, environment, timestamp, and reason.
Host passes do not substitute for QEMU rows; nothing here proves P7-V05
through P7-V30.

## 2. Error, security, and observability model

**Errors.** Recoverable classes: `Illegal`/`ControlNotAuthorized`/
`InvalidSequencing` (engine), `NotDispatchable`/`PlacementIneligible`/
`NotActive`/`BypassRequired` (gate). Guarantee: rejected requests leave all
state unchanged and emit a rejection trace event; the control loop decides
the next action (requeue, idle) — W05/W08 own the loop policy. Fatal class:
invariant violations (slot mismatch, engine-external mutation, post-
activation bypass use) terminate under the P0 panic policy with P1 crash
diagnostics. Guest-caused faults are never fatal and reach the engine only
as pre-classified `Fault` events.

**Security.** Authorization stays upstream (P5 capability rights via W03/W07
hooks); the engine checks context presence only. No Guest-influenced input
reaches transition legality: the gate's inputs are scheduler-internal state
and the frozen placement predicate. Policy parameters are structurally
absent from these APIs (mechanism/policy guardrail). The bypass register is
a security-relevant review artifact: an unregistered entry path is treated
as an isolation defect, not a convenience.

**Observability.** Semantic events: transition accepted/rejected (with
reason), gate result, activation, bypass use. W09 owns encoding, filtering,
rates, and aggregation; W02's obligation is that every state change and
every rejection is observable, and that diagnostics for a stuck vCPU can
name its state, last transition, owning pCPU, and last gate verdict
(P7-V21 input, delivered via W09).

## 3. Handoff checklist

Before handing W02 to a reviewer, provide:

- the exact changed-file list (expected: lifecycle/admission/audit module
  sources and tests per the repository's approved layout; no crate or
  workspace additions beyond approved decisions);
- W02-DV01…DV07 evidence paths and run status, including explicit not-run
  entries (expected not-run at design acceptance: QEMU rows, consumer
  integrations);
- confirmation: no new `unsafe`, no new dependencies, no public API, no
  trace encoding, no crate-boundary change beyond approved layout;
- confirmation that the state set, edge set, gate semantics, and invariants
  match [the contracts file](03-code-contracts-lifecycle.md) with no local
  additions;
- the final bypass-instance list with classes for reviewer sign-off; and
- open items for consumers: W03 (eligibility predicate seam live), W04
  (deschedule/reason plumbing), W05 (control loop + queue discipline under
  the leaf-lock rule), W06 (wakeup protocol preserving no-lost-wakeup),
  W07 (pause/stop flows with owner attestation), W09 (semantic rendering),
  W11 (invariant targets), W14 (accepted-boundary record into the P8
  handoff) — without resolving their contracts here.
