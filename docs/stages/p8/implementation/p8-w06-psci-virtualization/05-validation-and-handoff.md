# P8-W06 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W06 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none asserts that a test has run. Scenario IDs
S1–S5 are referenced from the README mapping table and the workflow. Evidence
destination: `../../verification/p8-w06-psci-virtualization-verification.md`
(do not create it before evidence exists).

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W06-DV01 → gate | frozen-fact single-source review | inspect code and `VmPsciConfig` construction | subset, conduit, version come only from approved machine facts (Step 1 acceptance) | the freeze is respected in code; not that the values are correct — that is W02/P8-V02/V03 |
| W06-DV02 → P8-V09 S1 | version/features probe | Linux-style probe sequence over the full DEN 0022 ID set | VERSION returns the frozen value; FEATURES reports exactly the frozen subset; everything else NOT_SUPPORTED | standard probe compatibility; not Linux boot success by itself |
| W06-DV03 → P8-V09 S2 | secondary start via CPU_ON | boot Linux 2-vCPU fixture (W15/W16 harness); observe secondary's first entry at the W03 state | secondary enters at entry IPA with x0=context_id and reaches Linux secondary init | the standard path works; not scheduler fairness or 4-vCPU scale (W10/W11) |
| W06-DV04 → P8-V09 S3 | CPU_OFF + AFFINITY_INFO hotplug cycle | offline a CPU via Linux hotplug; poll AFFINITY_INFO during and after | post-off read is OFF; offline completes without hang; cycle S3b repeats OFF→ON→OFF with identical state | lifecycle round-trip integrity; not hotplug performance |
| W06-DV05 → P8-V09 S4 | SYSTEM_OFF/SYSTEM_RESET containment | poweroff the Guest; observe Host and (when W10 exists) a second fixture VM | Guest execution ceases; Host shell/Host state unaffected; reset path re-enters via W03 boot inputs | VM-scoped powerdown; not Host power-management behavior |
| W06-DV06 → P8-V09 S5 / P8-V24 | malformed-input containment | drive S5a–S5f from the W18 illegal-PSCI set (host-driven harness or fixture) | each returns the declared code; VM continues or powers down per scenario; no Hypervisor fault, no cross-VM effect | guest-untrusted containment of the PSCI path; not general fault containment (W13) |
| W06-DV07 → P8-V09 | no private-HVC dependency review | inspect Linux boot log and DTB consumption; verify no W06-specific hypercall is required anywhere in boot | Linux uses only standard PSCI DTB-discovered calls | the task book's "no private HVC" requirement; not that all PSCI features Linux might ever use exist |
| W06-DV08 → P8-V03 | host-fact non-leakage review | code review + response-value audit of dispatch and AFFINITY_INFO | no pCPU/Host/platform-derived value reaches Guest responses | the machine's host independence on this path; not full machine-ABI independence (W02) |
| W06-DV09 → W14 handoff | compatibility-dimension review | compare the dimension list of [03 §6](03-code-contracts-cpu-lifecycle.md) against W14's drift checklist | all PSCI dimensions present and observable | compat-test coverage readiness; not drift-test execution (W16) |

Record each validation as passed/failed/blocked/not run with command, input,
environment, timestamp, and reason. A Linux boot reaching `start_kernel`
proves only DV02/DV03 partially — P8-V09 requires the lifecycle behaviors
(secondary/off/system-off), not boot alone.

## 2. Error, security, and observability model

- **Error model:** two explicitly separated classes (README decision D7):
  Guest-triggerable malformed input → DEN 0022 error codes with zero state
  residue; Hypervisor-internal failure → VM-scoped fault escalation (W13),
  never a PSCI status, never a Host panic for guest-triggerable paths (ADR
  §19). The seam between them is the code review focus of Step 2/Step 3.
- **Security model:** the Guest is untrusted on every input (function IDs,
  MPIDRs, IPAs, reserved bits); target resolution is VM-scoped at exactly one
  point ([02 §3](02-code-contracts-psci-dispatch.md)) and re-checked under
  lock in CPU_ON; no capability/role substitution occurs because the path has
  no management surface at all (ADR-051). SYSTEM_OFF containment (D3) is the
  boundary that keeps a Guest from powering off anything but itself.
- **Observability:** one structured telemetry event per dispatch outcome
  (accepted/rejected/internal), bounded per-VM counters, VM-facing diagnostic
  context for W13. Events feed W17's baseline and are prunable/filterable per
  ADR-048. No unbounded buffers exist on this path.

## 3. Handoff checklist

Before handing W06 to a reviewer, provide:

- the exact changed-file list and the logical-module mapping (M1–M5 → actual
  units) as implemented;
- the approved W02 gate values used (subset, conduit, reported version) and
  their authority location;
- DV01–DV09 evidence paths with run status, including explicit not-run
  entries (e.g. 4-vCPU scale awaits W10; second-VM containment awaits W10);
- confirmation that no scheduler policy, P5 hypercall number, DTB byte, or
  Linux-side change was introduced, and an inventory of any new `unsafe`
  with SAFETY justification (expected: none — this design requires none);
- confirmation of Step 6 seam agreement with W07/W08/W10 designs;
- open items: W02 gate status; P5 numbering-collision resolution; W14/W16/W18
  consumption readiness — without resolving their contracts here.
