# P1-W11 Fault Scenario Matrix

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W11 detailed design](README.md).

## 1. Structure of a scenario

Every scenario states: the phase it targets (W09 vocabulary), the setup and
trigger category, the trigger mechanism and its containment, the expected
diagnostic class (fields owned by W05/W07), the expected terminal outcome
(the W09 route for that phase), the pass condition, and the proof boundary.
Execution always runs through the
[P1-W10](../p1-w10-qemu-boot-regression/README.md) conventions: bounded run,
outcome classification, capture retention, evidence referencing.

Diagnostic-class expectations reference the W07 report fields at plan level:
build identity, CPU/exception level, PC/return state, syndrome (ESR-class
information), fault address, startup phase, and useful register context.
The accepted W05/W07 designs fix the concrete field list and marker tokens;
W11's matrix states the *classes* and defers exact tokens to those contracts.
A scenario whose expected class cannot be expressed once those contracts land
is a recorded coordination issue, never a locally redefined field list.

## 2. Scenario matrix

### NC1 — Unsupported execution environment

- Phase/target: `entry` (W01 rejection boundary).
- Setup/trigger category: environment variation. Suggested technique: run the
  reference platform configuration with EL2 unavailable (CPU virtualization
  disabled), so firmware cannot hand off at Non-secure EL2. Exact property
  spelling per the W01/P0 runner contracts at implementation time.
- Trigger mechanism/containment: none in the image — the disallowed
  environment is the trigger.
- Expected diagnostic class: W01's rejection reason identifying the disallowed
  entry state; no runtime markers after it (no `runtime` completion).
- Expected terminal outcome: W01's bounded stop before transfer; runner
  outcome recorded as the canonical unsupported-entry failure (a defined
  failure, distinct from `ERROR-INVOCATION`).
- Pass condition: rejection diagnostic present and phase-attributed; no
  normal continuation; reproducible twice.
- Proves: the entry boundary fires in a disallowed environment and does not
  continue. Does not prove: real-hardware EL2-absence behavior, or anything
  about environments other than the one tested.

### NC2 — Missing required capability

- Phase/target: `capabilities` (W03 fail-fast policy).
- Setup/trigger category: environment variation selecting a CPU model that
  lacks a capability the W03 contract classifies as required (exact CPU
  property per the W03 required list; suggested technique: a reduced CPU
  model on the reference platform).
- Trigger mechanism/containment: none in the image.
- Expected diagnostic class: W03's explicit required-capability absence
  reason (which capability, why required); **not** an unrelated panic.
- Expected terminal outcome: the `capabilities` route per the W09 matrix
  (terminal via the early route).
- Pass condition: absence reason names the capability; terminal outcome as
  routed; optional-capability absences (if the technique exposes any) remain
  distinguishable and do not abort.
- Proves: fail-fast policy is real and specific. Does not prove: the
  completeness of W03's required list, or hardware CPU-feature behavior.
- Reference-environment limitation: QEMU 8.2.2 exposes no supported way to
  remove W03's Required 4 KiB capability while retaining EL2 entry. Record
  environment-only NC2 as unavailable; never substitute DTB rejection for it.
- Authorized validation-image variant: a compile-time NC2 selection applies
  `inject_nc2_sample(raw: u64) -> u64` immediately after acquisition of
  `ID_AA64MMFR0_EL1` and before its existing decoder. It replaces only bits
  31:28 (TGran4) with `0xf`, preserving every other sampled bit. The function
  is pure, owns no state, allocates nothing, cannot fail, and exists only in
  the selected validation build. The ordinary decoder, classifier, Required
  policy and failure route are unchanged. No runtime or external input may
  select it. Its caller is exactly that acquisition boundary; it is not the
  divergent `fault_scenario` entry used by NC3–NC6.
- Variant acceptance: build default and NC2 images, inspect containment and
  the single-field delta, then execute the variant twice through the runner.
  Both executions must identify the absent required 4 KiB fact and terminate
  in phase `capabilities`, without later normal continuation. Record image
  identities and the injected field explicitly. This is executed policy and
  failure-route evidence for P1-V06, not evidence of CPU hardware absence.

### NC3 — Intentional synchronous fault (post-vector window)

- Phase/target: `exceptions` readiness; executed from a `console`-phase or
  later insertion point (after vectors are installed, before or after MMU per
  the scenario variant recorded at implementation; the primary variant is
  pre-MMU).
- Setup/trigger category: validation-only trigger executing an instruction
  whose synchronous exception is architecturally defined (suggested
  technique: a permanently-undefined instruction encoding).
- Trigger mechanism/containment: `fault_scenario(NC3)` call behind the
  validation selection, inserted at the recorded point (§3).
- Expected diagnostic class: W05 vector entry followed by the W07 report —
  syndrome identifying the exception class, PC/return state, fault address as
  applicable, startup phase attributed.
- Expected terminal outcome: the phase's W09 route (fatal path if ready,
  otherwise the panic route); bounded terminal behavior; no recursion.
- Pass condition: diagnostic class present with correct phase attribution;
  terminal outcome as routed; reproducible twice.
- Proves: synchronous exceptions reach a valid, diagnosable entry path and a
  bounded end. Does not prove: every syndrome class, nested-exception
  behavior, or hardware error handling.

### NC4 — Panic on the normal path

- Phase/target: `console` or later (panic route observable in output).
- Setup/trigger category: validation-only trigger invoking the panic route
  with a recorded static message.
- Trigger mechanism/containment: `fault_scenario(NC4)` behind the selection.
- Expected diagnostic class: W07 panic report — build identity, phase,
  message, and the required context fields.
- Expected terminal outcome: the panic route's bounded terminal behavior.
- Pass condition: report fields per the W07 contract; terminal, non-recursive;
  reproducible twice.
- Proves: the panic path produces its required evidence. Does not prove:
  panic-path behavior under fault-path recursion (W07's own boundary covers
  that; W11 does not re-test it).

### NC5 — Post-MMU translation/access fault

The suggested volatile-read technique below is superseded by the
[integrated trigger reconciliation](04-trigger-reconciliation.md): a single
AArch64 load instruction targets the proven unmapped L2 entry without a Rust
pointer dereference.

- Phase/target: `stage1` completion or later.
- Setup/trigger category: validation-only trigger performing a memory access
  to an address outside W08's mapped classes (suggested technique: a read of
  an unused, aligned address in a region the mapping contract does not map).
- Trigger mechanism/containment: `fault_scenario(NC5)` behind the selection,
  inserted after the MMU-enabled continuation point W08 defines.
- Expected diagnostic class: synchronous fault via the post-MMU entry path;
  W07 report including fault address (and translation information per the
  W05/W07 contract), phase attribution `stage1`-or-later.
- Expected terminal outcome: fatal path (ready since `fatal-path` completed);
  bounded terminal behavior.
- Pass condition: post-MMU diagnostics present and attributed; terminal;
  reproducible twice.
- Proves: fault reporting survives the MMU transition (supports P1-V14
  evidence too). Does not prove: Stage-2/Guest fault behavior (later stages),
  TLB/cache semantics, or that every unmapped access traps identically.

### NC6 — Unexpected vector at the stable state

The [integrated trigger reconciliation](04-trigger-reconciliation.md)
records NC6 as blocked: no genuine deterministic unexpected IRQ/FIQ/SError
event source has been established in the masked, no-GIC P1 runtime. The
suggested selection below is not an implemented or accepted proxy.

- Phase/target: `stable`.
- Setup/trigger category: validation-only trigger raising an exception
  category that W05's contract does not expect at the stable state (exact
  category per the W05 classification contract; suggested technique: an
  event routed to a vector the stable state never legitimately takes).
- Trigger mechanism/containment: `fault_scenario(NC6)` behind the selection,
  inserted at the stable-state attachment point W09 defines.
- Expected diagnostic class: W05's unexpected/unhandled classification with
  origin/context information and phase `stable`.
- Expected terminal outcome: the post-`stable` route (W05 vectors + W07 fatal
  path); bounded; no lifecycle transition (T5 of the W09 state machine).
- Pass condition: classification present, attributed to `stable`; terminal,
  non-recursive; reproducible twice.
- Proves: unexpected events at stable state are bounded and diagnosable. Does
  not prove: interrupt-controller behavior (no GIC in P1), or recovery.

## 3. Trigger containment contract

```text
Name and stability: fault_scenario(id: ScenarioId) -> !; validation-only;
  compiled only under the validation selection; absent from the default
  build (link/compile error if referenced without it).
Purpose and caller: the single intentional-fault entry for all in-image
  scenarios (NC3–NC6). Callers: exactly the insertion points recorded in the
  implementation record, one per scenario.
Inputs / outputs: scenario identifier; never returns (diverges via the
  intended fault or a bounded guard).
Preconditions / postconditions: the scenario's phase prerequisite holds at
  the insertion point (e.g., NC5 requires the post-MMU continuation); the
  trigger takes effect on the boot CPU only.
State and ownership change: none beyond the fault's own effects; triggers
  own no state.
Concurrency/allocation context: boot context; no allocation; no timing
  dependence (determinism requirement of the parent README decision 5).
Errors and failure guarantee: divergence is the contract; the function must
  not return, panic-with-panic, or loop noisily before the intended fault.
Security checks: the selection is build-time; there is no runtime "fault
  mode" switch, no guest/external input can select a scenario, and the
  default image contains no reachable trigger.
  Logic: per scenario — NC3 executes the undefined encoding; NC4 invokes the
  panic route with a static message; NC5 performs the out-of-map access
  through the single instruction fixed by 04-trigger-reconciliation.md;
  NC6 requires a genuine unexpected category and is currently blocked.
  Exact instructions are fixed at implementation per the accepted W05/W08
  contracts and recorded.
Validation: W11-DV02 containment review; W10 regression on the default
  image is the standing guard.
```

Environment-class scenarios (NC1 and the preferred NC2 environment variant) have no in-image trigger; their
containment is that nothing in the image changes at all — verified by using
the identical image identity as the passing R1 run of W10.

## 4. Reproducibility and objective acceptance

- Each scenario is executed twice; both runs must produce the same outcome
  class, the same phase attribution, and the same terminal behavior
  classification. Divergence is recorded as failed reproducibility (a
  P1-V18 failure), not averaged away.
- Acceptance is the conjunction: expected diagnostic class present, phase
  attribution correct, terminal outcome as routed, no evidence of
  continuation after the terminal marker, evidence retained per W10
  conventions.
- What the scenario set proves collectively: every listed failure class is
  bounded and diagnosable on the reference platform. What it does **not**
  prove: hardware fault behavior, completeness of any diagnostic contract,
  behavior of unlisted failure classes, or any recovery property (P1 has
  none).
- Evidence destination: sections NC1–NC6 of
  `../../verification/p1-w11-negative-fault-validation-verification.md`, each
  with both runs' outcomes, captures or their retained locations, and the
  technique actually used.
