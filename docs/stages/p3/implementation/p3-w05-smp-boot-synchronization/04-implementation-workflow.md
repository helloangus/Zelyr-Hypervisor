# P3-W05 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W05 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing anything, the implementer verifies it has loaded the entry
README, the Coding Guidelines, and the routed documents, and performs
read-only discovery. Implementation proceeds only when:

- [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) exposes the
  gate-assertion point and terminal outcome map per its design;
- [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) exposes
  `admit_online` and the terminal-state queries per its design;
- [P3-W04](../p3-w04-per-cpu-runtime/README.md) calls the ready signal as
  its install tail per its design;
- the P0 diagnostics governance is available.

W05 sits at the end of the W01–W04 chain, whose designs are being prepared
in parallel on this branch. If an upstream contract cannot be located when
implementation starts, record the coordination blocker and resolve the
contract first; do not improvise ordering semantics.

Stop and obtain direction when: an implementation step seems to need a
lock, a reusable barrier, or a timer (scope/design violation — W05 is
atomics plus one fence); a consumer asks W05 to coordinate runtime
(stop-the-world, hotplug) — refuse, Reserved; or the boot-integration
owner demands a halt-on-degradation policy — that is their Reserved
decision to record, not a silent change to `declare_smp_ready`.

## 2. Ordered implementation steps

### Step 1 — phase word and gate

Target: [03-code-contracts-boot-rendezvous.md](03-code-contracts-boot-rendezvous.md)
§1–§2.

Work: implement `BootPhase`, `publish_global_init`, `require_published`,
with CAS transitions and the fatal double-transition path; wire the gate
assertion into the W02 dispatch point and the secondary local-init start.

Acceptance: transitions are once-only and coordinator-only; gate
refusals are loud; phase events emitted.

Failure/blocker: a consumer that cannot tolerate a blocking-free gate
(i.e., wants to wait on the phase) is a design smell — the gate is
fail-closed, not a wait primitive; raise it.

Evidence: implementation record
(`../p3-w05-smp-boot-synchronization-record.md`).

### Step 2 — ready gate

Target: rendezvous file §3.

Work: implement the per-CPU ready flags and
`signal_local_init_complete` with identity cross-check and exactly-once
detection; wire the call into W04's install tail.

Acceptance: one signal per CPU; mismatched or repeated signals are fatal
invariants; flag publication uses release semantics.

Evidence: implementation record; verification record for tests.

### Step 3 — coordinator

Target: rendezvous file §4.

Work: implement `coordinate_rendezvous` with the terminal-bounded wait,
the admit-online loop over signaled CPUs, and degraded-participant
diagnostics. Structure the registry/outcome reads behind test seams so
the wait logic is host-testable.

Suggested observation: host-side tests under the P0 gate.

Acceptance: the wait terminates for every simulated combination (all
ready; mixed ready/failed; all failed); admissions happen only for
signaled CPUs; refusal is fatal in tests as designed.

Failure/blocker: a combination the wait cannot terminate on indicates a
missing terminal source — check W02's outcome guarantee; never add a
timeout to patch it.

Evidence: verification record
(`../../verification/p3-w05-smp-boot-synchronization-verification.md`).

### Step 4 — declaration and query

Target: rendezvous file §5.

Work: implement `declare_smp_ready` (degraded record, fence, CAS,
diagnostics) and `smp_ready_state`; wire the declaration into the boot
sequence after the coordinator returns and the state dump point after it
(W03's dump follows declaration).

Acceptance: declaration is once-only; the fence is present and
justified; `Degraded` carries the exact failed set; readers see a
coherent state.

Failure/blocker: dropping the fence because "atomics suffice" is a
design change — stop and record, never silently simplify.

Evidence: implementation record.

### Step 5 — consumer wiring and diagnostics

Work: confirm the phase-gate call in W02, the signal in W04, the
admission in W03, the assertions in W12/W13 surfaces (by contract;
their code is theirs); emit phase/rendezvous events per the P0
governance and coordinate event ids with
[P3-W11](../p3-w11-smp-observability/README.md).

Acceptance: every transition and the declaration emit attributable
events; consumers' contracts reference the implemented names.

Evidence: implementation record.

### Step 6 — rendezvous evidence and closure review

Work: via the P0 QEMU entry path, capture per declared count (1, 2, 4, 8
as reachable): the full phase sequence in boot output, the SMP-ready
result, and (for at least one failure case, e.g., the W02 absent-CPU
input) the `Degraded` record with the failed CPU listed. Repeat each
capture twice to observe rendezvous repeatability at capture scale, and
record the optional latency observation as informative. Then run the
closure review against
[05-validation-and-handoff.md](05-validation-and-handoff.md) and the
handoff checklist.

Acceptance: captures show once-only publication, per-CPU signaling,
declaration after the declared condition, and correct degraded
accounting; the full repeated matrix is explicitly deferred to
[P3-W13](../p3-w13-qemu-smp-regression/README.md).

Failure/blocker: a boot where SMP-ready precedes a missing CPU's terminal
record is a failed item — diagnose the terminal condition; do not relax
the readiness declaration.

Evidence: verification record.

## 3. Evidence destinations

- Implementation record:
  `docs/stages/p3/implementation/p3-w05-smp-boot-synchronization-record.md`
  (created when implementation starts).
- Verification record:
  `docs/stages/p3/verification/p3-w05-smp-boot-synchronization-verification.md`
  (created when evidence exists).

Neither file is created by this design.
