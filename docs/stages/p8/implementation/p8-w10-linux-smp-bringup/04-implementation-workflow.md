# P8-W10 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W10 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
(preflight included), and the sibling contracts cited there. Useful
read-only discovery: `git ls-files` (confirm the actual crate layout) and a
search for any existing SMP/topology statement this design must not
contradict.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 gate has not approved `<VCPU-COUNT>` / the MPIDR mapping rule — no
  value may be embedded ([01 §2](01-architecture-and-state.md)); record the
  blocker;
- the 1-vCPU baseline of [W09](../p8-w09-virtual-console-single-cpu-linux/README.md)
  Stage D is not green — SMP stages are blocked by design (the build-up
  discipline); record the blocked state, do not debug SMP on a broken
  baseline;
- a P7 seam (admission, placement, stop, accounting, block/wakeup) cannot
  express what [02](02-code-contracts-secondary-bringup.md) consumes —
  record an `Architecture Change Request` against the seam; never patch
  scheduler policy locally;
- the P3 Host SMP substrate or TLB transport is unavailable — multi-pCPU
  placement scenarios are blocked; the single-pCPU fallback of
  [01 §7](01-architecture-and-state.md) is a validation limitation to
  record, not an implementation mode to adopt silently;
- W04 DTB CPU nodes disagree with the machine facts — coherence check C1
  fails at creation; fix in the machine contract, not in code;
- implementation seems to need spin tables, >4 vCPUs, cluster topology, or
  new PSCI functions — Reserved/Out of Scope (README classification).

## 2. Ordered implementation steps

### Step 1 — topology registry and coherence checks

Target: `topology_build` and checks C1–C4 of
[02 §2](02-code-contracts-secondary-bringup.md).

Work: implement the registry from approved machine facts + W04 DTB facts,
with creation-failing coherence checks; derive the W06 target table (C3) so
there is exactly one topology source.

**Acceptance:** a coherent 1/2/4-vCPU configuration builds; any injected
inconsistency fails creation with the named check (DV01).  
**Failure/blocker:** missing gate approval is a recorded blocker (§1).

### Step 2 — secondary bring-up pipeline

Target: [02 §3](02-code-contracts-secondary-bringup.md) over the W06 CPU_ON
body.

Work: wire the pipeline as the CPU_ON orchestration body with stage
telemetry; confirm the W03 secondary entry state and P7 admission seams are
exactly the consumed contracts (no local entry-state logic).

**Acceptance:** at 2 vCPU, a secondary commits, is placed, enters at the W03
state, and reaches Linux secondary init — observed via telemetry and console
markers (P1–P2 of [02 §4](02-code-contracts-secondary-bringup.md)).  
**Failure/blocker:** a seam mismatch stops per §1.

### Step 3 — per-CPU integration readiness (M3)

Target: [02 §4](02-code-contracts-secondary-bringup.md) observation hooks
P1–P6.

Work: wire readiness recording from the W07/W08/W09 telemetry hooks and the
console marker; no gating logic anywhere.

**Acceptance:** a brought-up secondary reaches the full P1–P6 record; the
record resets correctly across hotplug cycles.  
**Failure/blocker:** a missing owner telemetry hook blocks that stage's
observation (recorded), never fakes readiness.

### Step 4 — hotplug/offline integration (M4)

Target: [02 §5](02-code-contracts-secondary-bringup.md).

Work: wire the cycle phases to W06 handler/entry telemetry; verify the
quiesce-hook ordering observably completes before OffDone; assert
state-identical re-ON with M3's fresh readiness cycle.

**Acceptance:** a Linux offline/online cycle completes with consistent
state (AFFINITY_INFO correct at every phase); repeated cycles stay
identical (S3b).  
**Failure/blocker:** a quiesce-hook mismatch with W07/W08 stops integration
per §1.

### Step 5 — fault path integration (M6)

Target: [02 §6](02-code-contracts-secondary-bringup.md).

Work: wire the secondary fault path to P7-W07 and the W13 context; confirm
sibling vCPUs continue while one is faulted/stopped.

**Acceptance:** a faulted secondary is contained; diagnostics populated;
siblings unaffected.  
**Failure/blocker:** a containment gap is an `architecture`-severity
finding to fix in the owning designs, not to absorb here.

### Step 6 — scenario engine and correlation (M5)

Target: [03](03-stability-scenarios.md) — run-record structure,
`scenario_correlate`, verdict classes.

Work: implement the run-record capture (telemetry + log + markers) and the
pure correlation function; provide it to W16 as the only verdict path.

**Acceptance:** correlation is reproducible from the same record (DV07);
missing streams yield `inconclusive`, never guesses.  
**Failure/blocker:** a scenario needing a stream no owner provides is
recorded as blocked with the named stream.

### Step 7 — scenario execution and closure review

Work: execute the [05](05-validation-and-handoff.md) matrices as far as
current integration permits — 2 vCPU fully before 4 vCPU; evidence to
`../../verification/p8-w10-linux-smp-bringup-verification.md`; decisions to
`../p8-w10-linux-smp-bringup-record.md`. Completion claims only in the
verification record, only for what ran.

**Acceptance:** matrix entries passed/failed/blocked/not run with evidence;
P8-V14/V15 rows marked run have real output and verdict records.  
**Failure/blocker:** a failed scenario is evidence of a failure — recorded
with severity per [03 §1](03-stability-scenarios.md), fixed through the
owning contracts, never by weakening observables.

## 3. Ordering rationale

Topology (Step 1) → pipeline (Step 2) → readiness (Step 3) → hotplug
(Step 4) → faults (Step 5) → scenarios (Steps 6–7) mirrors both the Guest's
own bring-up order and the risk gradient: identity errors (topology) are
cheapest to catch first, behavioral races (scenarios) last, on top of proven
per-CPU mechanisms.
