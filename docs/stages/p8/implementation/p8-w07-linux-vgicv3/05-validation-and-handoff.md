# P8-W07 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W07 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none asserts that a test has run. Evidence
destination: `../../verification/p8-w07-linux-vgicv3-verification.md` (do not
create before evidence exists). Scenario IDs S1–S6 are referenced from the
README mapping table and the workflow.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W07-DV01 → gate | machine-fact single-source review | inspect region/INTID/TYPER population | all Guest-visible GIC values from approved machine facts (Step 1) | freeze respected in code; value correctness is W02/P8-V02/V03 |
| W07-DV02 → P8-V10 S1 | boot-time GIC init | boot Linux 1-vCPU fixture ([W09](../p8-w09-virtual-console-single-cpu-linux/README.md) baseline); observe GIC probe markers | Linux detects GICv3, enumerates redistributors, enables group 1; no spurious RAZ/WI complaints on the implemented subset | required register behavior at boot; not ITS/MSI (excluded) |
| W07-DV03 → P8-V10 S2 | secondary-CPU GICR bring-up | 2/4-vCPU fixture ([W10](../p8-w10-linux-smp-bringup/README.md)); observe per-CPU GICR init | each secondary wakes (WAKER), enables its SGI/PPI frame, joins interrupt handling | per-vCPU model correctness; not scheduler placement (W11) |
| W07-DV04 → P8-V10 S3 | SGI matrix | Linux IPI stress: self-IPI, targeted, broadcast, cross-vCPU pairs | all SGIs delivered with target isolation; invalid-target drops counted, contained | the §2 path; not general IPI performance |
| W07-DV05 → P8-V10 S4 | timer IRQ path | with [W08](../p8-w08-linux-timer-integration/README.md): periodic timer PPI under load | timer interrupts delivered per configured edge/level; EOI completes state | the PPI path; not timer semantics themselves (W08's matrix) |
| W07-DV06 → P8-V10 S5 | mask/unmask and pending/active fidelity | scripted enable/disable/set-pending/EOI cycles including disable-with-pending and re-enable | no lost or duplicated pending state across the cycles; active state exact | the §4/§5 state model; not priority fairness |
| W07-DV07 → P8-V10 S6 | declared multi-CPU stress | bounded interrupt storm mixes (SGI + timer + console SPI) per the scenario contracts of [W10 §5](../p8-w10-linux-smp-bringup/README.md) | state fidelity holds (no loss/duplicate/cross-vCPU leak); Host remains responsive; rates bounded by fixture | robustness under declared stress; not throughput (excluded) or real-hardware behavior |
| W07-DV08 → P8-V24 | malformed-access containment | W18 illegal-MMIO set: out-of-region, bad widths, INTID overflow, register abuse, storm | declared contained outcomes only; VM continues or is cleanly faulted; Host unaffected | guest-untrusted containment; not full fault taxonomy (W13) |
| W07-DV09 → P8-V03 | host-independence review | audit TYPER/IIDR/frame facts for Host-derived values | none present; all machine-gated | path-level host independence; not whole-machine review (W02) |
| W07-DV10 → W14 | compatibility-dimension review | compare Guest-visible GIC facts against W14's drift checklist | regions, subset, TYPER values, INTID allocations all listed | compat readiness; not drift-test execution (W16) |

Record each validation as passed/failed/blocked/not run with command, input,
environment, timestamp, reason. Boot reaching `start_kernel` alone satisfies
only part of DV02 — P8-V10 requires the interrupt-behavior rows. Stress rows
(S6) are declared-boundary tests: exceeding declared rates is a fixture
configuration question (W16), not automatically a design failure, but any
fidelity failure (lost/duplicate/leak) is a design failure.

## 2. Error, security, and observability model

- **Error model:** three contained classes — Direct/RAZ-WI (normal
  emulation), Reject (fault-classified per W05 with bounded W13 diagnostic
  context), and internal-fault (hypervisor-side failure, escalates via W13,
  never Guest-visible as success). No GIC path can panic the Host for
  Guest-reachable input (ADR §19).
- **Security model:** the Guest is untrusted on every access (region,
  offset, width, INTID fields, SGI target lists). INTID indexing is
  machine-table-bounded with checked arithmetic; SGI targeting is
  VM-scoped at one validated point; no cross-vCPU or cross-VM register
  reach exists. State truth lives in P6 lifecycle operations — the register
  layer holds only derived, locked mirrors, so emulation bugs cannot silently
  fork delivery state.
- **Observability:** per-access telemetry classes; injection/EOI/SGI outcome
  events with source tags; bounded per-VM/per-vCPU counters feeding W17's
  baseline (interrupt-latency method inherited from P6-W13's declared
  measurement approach); Reject diagnostics recorded for W13. All events are
  prunable and runtime-filterable (ADR-048); no unbounded buffers.

## 3. Handoff checklist

Before handing W07 to a reviewer, provide:

- the exact changed-file list and M1–M6 → actual-unit mapping as implemented;
- the approved W02 gate values used (regions, INTID table, TYPER facts) and
  their authority location;
- DV01–DV10 evidence paths with run status, including explicit not-run
  entries (e.g. S6 multi-CPU storm awaits W10; SPI rows await W09's console);
- confirmation that no P6 lifecycle logic was reimplemented, no Host GIC
  driver behavior changed, no DTB bytes written, and no QEMU-derived constant
  entered the code; inventory of new `unsafe` with SAFETY justification
  (expected only at arch register-access boundaries owned by the arch layer);
- confirmation of Step 6 seam agreement with W06/W08/W09/W10 designs;
- open items: W02 gate status; EOImode-1 trigger status (Reserved); W14/W16/
  W18 consumption readiness — without resolving their contracts here.
