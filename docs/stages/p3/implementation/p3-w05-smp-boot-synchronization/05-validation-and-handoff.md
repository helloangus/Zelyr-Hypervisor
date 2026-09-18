# P3-W05 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W05 detailed design](README.md).

## 1. Validation matrix

Each item is recorded as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason in the
verification record. QEMU items appear only where W05's scope produces
them; the repeated matrix and stress evidence belong to
[P3-W13](../p3-w13-qemu-smp-regression/README.md) and
[P3-W12](../p3-w12-smp-stress-failure-tests/README.md).

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W05-DV01 | Once-only authority (→ P3-V05) | protocol tests + review | unit tests: double publication, double declaration, out-of-order transitions all fatal; review designated-actor checks | exactly one successful transition per edge; every illegal attempt fatal or refused | the once-only property of the machinery; not that boot code always calls in order (integration review) |
| W05-DV02 | No secondary use of incomplete global state (→ P3-V05) | gate tests + capture review | unit tests of `require_published` refusals; QEMU captures where the phase sequence precedes dispatch in every boot | gate refuses pre-publication access; every captured boot publishes before dispatch | structural ordering on the reference platform; not adversarial firmware behavior on other platforms |
| W05-DV03 | Per-CPU local initialization required (→ P3-V05) | protocol tests + QEMU capture | simulated wait terminations (ready/failed mixes); captures show per-CPU ready events before admissions | admission only after signal; wait bounded by terminals, no timer | readiness discipline; not W04 install correctness (W04's evidence) |
| W05-DV04 | Degraded failure observability (→ P3-V05) | unit tests + failure-case capture | simulated failed participants; QEMU capture with the W02 absent-CPU input showing `Degraded` with the failed set | degraded record exact; nothing converts Degraded to Ready; failed set derivable from the registry | failure accounting; not recovery (none exists at P3, by design) |
| W05-DV05 | Consumer contracts integrated (→ W05 closure) | closure review | verify W02 gate point, W04 signal point, W03 admission calls, W12/W13 assertion surfaces against implemented names | each consumer contract matches; no silent semantic drift | handoff readiness; not consumer correctness |
| W05-DV06 | Repeated rendezvous behavior and timing observations (→ P3-V05) | repeated captures + informative timing | per declared count: two consecutive boots' phase sequences compared; optional counter-based rendezvous latency recorded as informative | identical phase sequences and consistent degraded accounting across repeats; latency observations recorded (not asserted) | capture-scale repeatability on the reference platform; not regression-grade repeatability (W13), not hardware platforms, not wall-clock timing semantics (P6/W11 Reserved) |

A rendezvous capture does not prove lock correctness elsewhere (W06), nor
lifecycle admissibility by itself (W03), nor stress-path behavior (W12).
The timing observation is diagnostic; it proves no latency property.

## 2. Error, security, and observability model

- **Error model.** Ordering violations (double transitions, early
  dispatch, early signals, admission refusals) are fatal invariants with
  phase and CPU attribution; participant failure is a terminal,
  diagnosable `Degraded` outcome with a recorded failed set; the
  coordinator has no failover at P3 — coordinator failure is the fatal
  boot path (P1-W07).
- **Security.** The rendezvous is not guest-reachable. Its signaling
  surface accepts only self-signed signals (a CPU signals its own
  identity); the phase authority is coordinator-only. The design adds no
  guest-visible or hypercall-adjacent behavior; guest SMP coordination is
  P8's.
- **Observability.** Every phase transition, ready signal, admission,
  degradation, and the final `SmpReadyState` emit attributable events per
  the P0 governance; `smp_ready_state` is the programmatic assertion
  surface for W12/W13; the optional latency observation is recorded
  informationally until real timing ownership arrives (P6/W11).

## 3. Handoff checklist

Before handing W05 to a reviewer:

- the exact changed-file list, with crate/module placement traced to the
  approved workspace design;
- W05-DV01–DV06 evidence paths with run status, including explicit
  not-run entries (counts not reachable; matrix deferred to W13; timing
  semantics deferred to P6/W11);
- confirmation that new `unsafe` is zero (or the single fence intrinsic
  usage is inventoried per P0 governance) and that no lock, reusable
  barrier, timer, or reset API exists in the package;
- consumer contract confirmations: W02 (gate point), W03 (admission
  ownership), W04 (signal point), W06 (ordering baseline handed over),
  W10 (once-only/per-CPU audit rules), W12 (assertion surface + failure
  checkpoint), W13 (required boot assertions), W15 (readiness conditions
  and limits);
- the recorded open question: halt-vs-continue boot policy for degraded
  results (default continue-with-diagnostics stands for the reference
  boot; owner: boot-integration integration decision);
- any recorded blocker (sibling-contract coordination; diagnostics
  namespace) with its decision owner.
