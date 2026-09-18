# P7-W10 Scenario Suite: Regression and VG-SCHED Catalog

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W10 detailed design](README.md).  
All conditions below are planned pass/fail conditions and proof boundaries —
no result is claimed anywhere in this design.

## 1. Scenario entry format

Every scenario records: ID; upstream contract under test; topology
(VMs/vCPUs/pCPUs, placement class); Guest behavior; Guest markers; telemetry
correlates ([P7-W09](../p7-w09-accounting-diagnostics/README.md) events);
pass condition; authority (which observation side decides); what it proves /
does not prove; prerequisites; scaling notes for W11.

## 2. Single-vCPU regression (P7-V22)

**Declaration.** The maintained upstream scenario sets — P4-W05
VG-001–VG-012 (mandatory subset per P4-W05 acceptance), the P5-W07
HVC/security set, and the P6-W11 VG-TIMER/VG-IRQ sets — run **unchanged** on
1VM/1vCPU/1pCPU under the P7 scheduler-controlled entry path
([P7-W02](../../plans/p7-w02-scheduler-admission-lifecycle.md) seam).

- **Pass condition:** every inherited scenario produces its original expected
  markers and exit classifications on the scheduled path; the run completes
  with the same determinism the upstream records declare; scheduler telemetry
  shows the vCPU entering and leaving via scheduler control (W09
  `vcpu_switch` with `InitialDispatch` and normal exits).
- **Authority:** Guest markers decide Guest-visible behavior; telemetry
  decides that entry/exit is scheduler-controlled. Both must hold.
- **Proves:** scheduler-controlled entry preserves the inherited Guest
  behavior (no regression from admission/pause-check/block hooks).
- **Does not prove:** any multi-vCPU, M:N, or SMP property; hardware
  behavior.
- **Prerequisites:** P4/P5/P6 assets and records; W02 admission design.
- **Scaling:** none — fixed shape by definition (decision 1 of the README).

## 3. VG-SCHED catalog (P7-V23)

### VG-SCHED-01 — CPU-bound progress under overcommit

- **Under test:** [P7-W04](../../plans/p7-w04-preemption-context-switch.md)
  preemption; [P7-W05](../p7-w05-shared-mn-multivm/README.md)
  M:N progress.
- **Topology:** 1 VM, 2 vCPUs, 1 pCPU (1:2 overcommit), shared placement.
- **Guest behavior:** each vCPU runs a bounded counter loop (iteration-
  counted, no timing dependence), emitting a progress marker every K
  iterations.
- **Markers:** interleaved progress sequences from both vCPUs; each ends
  with its completion marker.
- **Telemetry:** `vcpu_switch` events with `TimesliceExpiry` /
  `PreemptedReconsideration` reasons; both vCPUs show nonzero `guest_runtime`.
- **Pass condition:** both vCPUs reach completion; neither monopolizes (both
  progress marker sequences interleave at observable granularity); no
  duplicate-running invariant trip (P7-V04 checks in telemetry).
- **Authority:** telemetry decides scheduling facts; markers decide
  Guest-visible progress.
- **Proves:** a CPU-bound Guest cannot monopolize a shared pCPU; M:N
  rotation works.
- **Does not prove:** fairness ratios, latency, or specific slice behavior;
  hardware preemption timing.
- **Prerequisites:** W04/W05 designs; W09 records.
- **Scaling (W11):** vCPU counts to 4:8 per P7-V24; iteration counts scalable.

### VG-SCHED-02 — Periodic WFI block and release

- **Under test:** [P7-W06](../p7-w06-block-wakeup/README.md) blocking.
- **Topology:** 1 VM, 2 vCPUs, 1 pCPU.
- **Guest behavior:** vCPU-A executes a fixed number of iterations then WFI
  (no wake programmed); vCPU-B runs a bounded counter loop.
- **Markers:** vCPU-A emits `WFI-ENTER` then (after a cross-vCPU signal from
  VG-SCHED-04's producer role, or at test teardown for the pure-block check)
  nothing further until woken; vCPU-B progresses monotonically during A's
  block.
- **Telemetry:** `vcpu_block` for A; B's switches continue; A consumes no
  pCPU capacity while blocked.
- **Pass condition:** A blocks with no busy re-entry (telemetry shows no
  repeated dispatch of A while blocked); B completes; a preexisting eligible
  event at A's WFI prevents the block (variant run).
- **Authority:** telemetry.
- **Proves:** blocking releases pCPU capacity; no busy-loop.
- **Does not prove:** idle power behavior; hardware WFI semantics.
- **Prerequisites:** W06 design; P4 WFI-exit classification.
- **Scaling:** blocked-vCPU count scalable.

### VG-SCHED-03 — Cross-vCPU SGI wake

- **Under test:** W06 wake path, `VirtualIrq`/SGI source; P6-W11 Host-SGI
  support.
- **Topology:** 1 VM, 2 vCPUs, 1 pCPU.
- **Guest behavior:** vCPU-A WFI-loops; vCPU-B sends a declared SGI to A
  every loop iteration; A counts received SGIs and emits a marker per
  receipt.
- **Telemetry:** `vcpu_wake` with source `VirtualIrq` for each wake; wake
  count equals send count (no lost wake) in the bounded run.
- **Pass condition:** A wakes on every SGI; receipt markers are sequence-
  numbered without gaps; no duplicate wake for one SGI.
- **Authority:** markers decide delivery ordering; telemetry decides wake
  accounting equality.
- **Proves:** event wake without loss or duplication under overcommit.
- **Does not prove:** SGI latency bounds; GIC hardware behavior.
- **Prerequisites:** W06 design; P6-W07/P6-W11 contracts.
- **Scaling:** sender/receiver pairs scalable for W11 race amplification.

### VG-SCHED-04 — Virtual-timer wake

- **Under test:** W06 `TimerExpiry` wake; P6-W06 vCPU timer.
- **Topology:** 1 VM, 2 vCPUs, 1 pCPU.
- **Guest behavior:** vCPU-A programs its vCPU timer (per the P6-W06 Guest
  interface) for a declared tick count, WFI-loops, and emits a marker per
  timer wake; vCPU-B idles via WFI as well (forcing the deadline-home/idle
  interlock of W06's design to be exercised).
- **Telemetry:** `vcpu_wake` with source `TimerExpiry`; deadline events on
  the home pCPU.
- **Pass condition:** A receives exactly the programmed number of timer
  wakes with gap-free sequence markers; B's block is unaffected.
- **Authority:** markers decide delivery; telemetry decides source
  classification.
- **Proves:** timer-driven wake of blocked vCPUs including via an idle
  pCPU's deadline fold.
- **Does not prove:** timer accuracy or monotonicity (P6-W05/W06 own their
  evidence); hardware timer behavior.
- **Prerequisites:** W06 design; P6-W05/W06 contracts and evidence.
- **Scaling:** tick counts scalable within declared bounds.

### VG-SCHED-05 — Shared-memory progress under M:N

- **Under test:** W05 progress/fairness basis; switch isolation context of
  W04.
- **Topology:** 1 VM, 2 vCPUs, 1 pCPU; one Guest-shared memory region
  (established per the P4/P5 Guest-memory conventions).
- **Guest behavior:** both vCPUs increment distinct sequence slots in shared
  memory with the Guest-side ordering the P4/P5 contracts support; an
  observer pass at completion checks both sequences.
- **Telemetry:** interleaved switches; per-vCPU runtime both nonzero.
- **Pass condition:** both sequences complete without gap or overlap of
  sequence numbers; the completion markers verify both vCPUs executed.
- **Authority:** markers (Guest-visible memory correctness); telemetry
  corroborates scheduling.
- **Proves:** vCPU switch isolation preserves Guest memory consistency
  assumptions under M:N.
- **Does not prove:** formal memory-model correctness; multi-VM isolation
  (P7-V11 is W05/W11's matrix).
- **Prerequisites:** W04/W05 designs; P4 Guest-memory conventions.
- **Scaling:** vCPU/region counts scalable.

### VG-SCHED-06 — HVC-heavy exit workload

- **Under test:** W04 preemption/switch machinery under high exit rates;
  P5-W07 HVC scenario compatibility.
- **Topology:** 1 VM, 2 vCPUs, 1 pCPU.
- **Guest behavior:** both vCPUs issue a declared mix of valid and
  deliberately-invalid HVCs (P5-W07 classes) in bounded loops, marking
  results.
- **Telemetry:** high switch counts with normal exit classifications; no
  `FaultedGuest` from invalid HVCs (they are Guest-facing denials per P5,
  not faults); accounting coherent.
- **Pass condition:** all HVC results match the P5-W07 expected classes; the
  run completes; counters (P7-V19) coherent under the exit storm.
- **Authority:** markers for HVC results; telemetry for classification and
  coherence.
- **Proves:** scheduler behavior stays correct under Guest-controlled exit
  storms; invalid control stays contained.
- **Does not prove:** HVC throughput or latency; P5 ABI semantics (those are
  P5's evidence).
- **Prerequisites:** W04/W05 designs; P5-W07 set; W09 records.
- **Scaling:** loop counts scalable.

### VG-SCHED-07 — Pause/stop observation under scheduling

- **Under test:** [P7-W07](../p7-w07-pause-stop-fault/README.md) pause/stop
  semantics as Guest-observable events.
- **Topology:** 2 VMs (the P5-W07 two-context setup), 2 vCPUs total, 1–2
  pCPUs.
- **Guest behavior:** VM-1's vCPU runs a marker loop; control-plane actions
  pause/resume it via the authorized P5 path from VM-2's context; VM-1 marks
  execution resumption.
- **Telemetry:** `control_commit` events with dispositions; `vcpu_switch`
  reasons `PausedControl` on pause.
- **Pass condition:** after pause, VM-1 emits no further markers until
  resume; after resume, markers continue in sequence without gap; a stopped/
  faulted vCPU (controlled-fault variant from the P4-W05 set) never re-executes.
- **Authority:** markers; telemetry corroborates dispositions.
- **Proves:** pause/resume/stop are Guest-observable, ordered, and
  contained.
- **Does not prove:** VM fault policy; snapshot behavior.
- **Prerequisites:** W07 design; P5 capability path; P4 controlled-fault
  triggers.
- **Scaling:** pause/resume repetitions scalable for W11 race stress
  (P7-V25).

### VG-SCHED-08 — Multi-VM coexistence workload

- **Under test:** [P7-W05](../p7-w05-shared-mn-multivm/README.md)
  multi-VM progress (scenario basis for P7-V11's full matrix, owned by
  W05/W11).
- **Topology:** 2 VMs, 2 vCPUs total, 1 pCPU (then 2 pCPUs variant).
- **Guest behavior:** each VM's vCPU runs VG-SCHED-01's counter loop
  independently (separate memory, no shared region — cross-VM isolation
  remains P5's domain).
- **Telemetry:** switches attribute each dispatch to its VM; both VMs
  progress.
- **Pass condition:** both VMs complete; no VM's markers stall while the
  other runs; placement constraints (if pinned) hold in telemetry.
- **Authority:** telemetry for placement; markers for progress.
- **Proves:** two independent VMs make progress under shared scheduling.
- **Does not prove:** proportional fairness; full P7-V11 matrix (W05/W11).
- **Prerequisites:** W05 design; P5-W07 two-context asset.
- **Scaling:** the 4:8 two-VM shapes are W11's P7-V24 amplification.

## 4. Coverage map to task-book validation

| Task-book row | Discharged by |
|---|---|
| P7-V22 | §2 regression declaration |
| P7-V23 (counter workloads) | VG-SCHED-01, -05, -08 |
| P7-V23 (WFI/wakeup) | VG-SCHED-02, -03, -04 |
| P7-V23 (SGI/IRQ) | VG-SCHED-03 |
| P7-V23 (timer) | VG-SCHED-04 |
| P7-V23 (HVC) | VG-SCHED-06 |
| W07 behavior observation | VG-SCHED-07 |

Every scenario row also carries the global boundary: QEMU success does not
prove hardware behavior; RK3566 evidence is a later-stage responsibility
(README decision 7).
