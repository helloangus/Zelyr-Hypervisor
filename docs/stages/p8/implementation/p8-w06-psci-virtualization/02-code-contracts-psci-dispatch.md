# P8-W06 Code Contracts — PSCI Dispatch, Version/Features, Error Mapping

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W06 detailed design](README.md).  
**Modules covered:** M1 (dispatch), M2 (validation/query), M5 (telemetry
hooks). Lifecycle contracts are in
[03](03-code-contracts-cpu-lifecycle.md).

All names are logical contract names owned by this design (README decision
D8); physical crate/module placement follows the approved workspace decision.
All pseudocode is an implementation outline, not runnable production code.

## 0. Shared data contracts

### 0.1 Function-ID namespace

Values in the SMC32/SMC64 PSCI range are fixed by Arm DEN 0022; this design
does not invent any ID. The frozen-after-W02 mandatory subset:

| Function | DEN 0022 SMC32 ID | SMC64 variant | Class | Semantics owned by |
|---|---|---|---|---|
| PSCI_VERSION | `0x84000000` | — | Query | [§2.1](#21-psci_version) |
| PSCI_FEATURES | `0x8400000A` | — | Query | [§2.2](#22-psci_features) |
| CPU_ON | `0x84000003` | `0xC4000003` | Lifecycle | [03 §2](03-code-contracts-cpu-lifecycle.md) |
| CPU_OFF | `0x84000002` | — | Lifecycle | [03 §3](03-code-contracts-cpu-lifecycle.md) |
| AFFINITY_INFO | `0x84000004` | `0xC4000004` | Query/lifecycle | [03 §4](03-code-contracts-cpu-lifecycle.md) |
| SYSTEM_OFF | `0x84000008` | — | VM system | [03 §5](03-code-contracts-cpu-lifecycle.md) |
| SYSTEM_RESET | `0x84000009` | — | VM system | [03 §5](03-code-contracts-cpu-lifecycle.md) |

Every other PSCI function ID (including CPU_SUSPEND, SYSTEM_RESET2, and all
v1.2+ additions) is **Reserved**: it answers `NOT_SUPPORTED` and is reported
unimplemented by PSCI_FEATURES (README decision D1, Reserved list). The exact
IDs above must be re-verified against the pinned DEN 0022 revision during
implementation; a mismatch is an implementation bug, not a design change.

### 0.2 Error-code contract

PSCI status values are the DEN 0022 codes (`SUCCESS`, `NOT_SUPPORTED`,
`INVALID_PARAMETERS`, `INVALID_ADDRESS`, `DENIED`, `ALREADY_ON`, `ON_PENDING`);
they are spec-fixed. The mapping rules this design owns:

| Internal condition | Guest-visible status | Rationale |
|---|---|---|
| Function ID outside frozen subset | `NOT_SUPPORTED` | Standard extension behavior; lets Linux probe |
| Reserved parameter bit non-zero | `INVALID_PARAMETERS` | DEN 0022 requires reserved bits to be zero |
| Target MPIDR not in the calling VM's virtual topology | `INVALID_PARAMETERS` | VM-scoped target resolution (decision D6); a Guest cannot address another VM |
| CPU_ON target is the calling vCPU, or is online or pending | `ALREADY_ON` | DEN 0022; the calling vCPU is by definition on; also covers pending-on state deterministically (no observable `ON_PENDING` window in the synchronous model, but the code exists for spec conformance) |
| Entry IPA outside Guest RAM or not Stage-2-mappable per P4 contracts | `INVALID_ADDRESS` | Untrusted input (ADR §19) |
| CPU_OFF rejected by lifecycle rules | `DENIED` | DEN 0022 semantics |
| Hypervisor-internal failure | No PSCI code: VM-scoped fault path (W13/P7-W07) | Decision D7 — internal errors never masquerade as firmware status |

### 0.3 VM PSCI configuration view (data contract)

```text
Name and stability: VmPsciConfig — internal; layout free; created at VM creation
Purpose and caller: authoritative record of the W02-frozen PSCI facts for one VM;
  read by dispatch (M1) and query handlers (M2)
Contents (logical):
  reported_version:  u32        — value frozen by the W02 machine-contract gate
  implemented:       set of PsciFunction — exactly the frozen subset
  conduit:           PsciConduit::Hvc — until W02 selects otherwise
Preconditions: populated only from approved machine facts; immutable at runtime
Postconditions: every dispatch decision on this VM derives from this view;
  no handler may hard-code a subset membership decision
Errors: construction fails (VM creation fails) if machine facts are absent —
  a VM without frozen PSCI facts is not bootable
Validation: schema review (DV01) plus P8-V02/V03 gate review of the values
```

## 1. Dispatch contract

```text
Name and stability: psci_handle_hvc — internal; the sole PSCI entry point
Purpose and caller: called from the unified Guest synchronous-exception path
  when the HVC function ID falls in the PSCI SMC namespace; returns the
  response register state
Inputs:
  frame:        read-only Guest exception frame (x0..x3 as call args)
  vcpu:         the calling vCPU (carries its VM)
Outputs: response register set (x0 mandatory; x1..x3 only where a function
  defines them — none in the frozen subset)
Preconditions:
  - the vCPU is Running in Guest context on a pCPU (VM-exit context)
  - VmPsciConfig exists for the VM (VM creation guarantees it)
Postconditions:
  - exactly one response value written to x0; caller state otherwise unmodified
  - one telemetry event emitted (accepted | rejected-by-code | internal-fault)
  - no blocking, polling, or unbounded work performed (see concurrency)
State and ownership change: none directly; delegated effects are owned by the
  lifecycle modules in [03](03-code-contracts-cpu-lifecycle.md)
Concurrency/allocation context: VM-exit context; may take bounded VM/vCPU
  lifecycle locks; must not allocate dynamically (README/01 §5); must not wait
  on another vCPU
Errors and failure guarantee: unknown/unsupported ID -> NOT_SUPPORTED without
  touching VM state; malformed input -> spec error per §0.2; internal failure
  -> escalate through the W13 fault path; in every failure case the Guest's
  register state except x0 is preserved
Security/authorization checks: namespace membership; reserved-bit zero rules;
  VM-scoped resolution of any target-bearing argument (enforced again in the
  lifecycle handlers; defense in depth is intentional)
Logic (pseudocode):
  fn psci_handle_hvc(frame, vcpu) -> Response:
      fid = PsciFunctionId::decode(frame.x0)     # returns Unknown(raw) otherwise
      cfg = vcpu.vm.psci_config                  # VmPsciConfig
      match fid:
          VERSION      -> respond(psci_version(cfg))
          FEATURES     -> validate zero-reserved(x1); respond(psci_features(cfg, x1))
          CPU_ON       -> validate smc64/smc32 width per cfg;
                          respond(cpu_on(vcpu, x1, x2, x3))       # [03 §2]
          CPU_OFF      -> respond(cpu_off(vcpu))                  # [03 §3]
          AFFINITY_INFO-> validate pwr_level; respond(affinity_info(vcpu, x1, x2))
          SYSTEM_OFF   -> respond(system_off(vcpu))               # [03 §5]
          SYSTEM_RESET -> respond(system_reset(vcpu))
          Unknown(_)   -> telemetry(rejected, NOT_SUPPORTED); respond(NOT_SUPPORTED)
      # every respond() path writes x0 only and emits exactly one M5 event
Validation: dispatch-table review; P8-V09 S1/S5 scenarios; unknown-ID fuzz
  cases in the W18 illegal-PSCI set
```

Classification note (README decision D5): the recognition rule "x0 in the
SMC32/SMC64 PSCI namespace ⇒ PSCI" is ordered **before** the P5 hypercall
dispatch. If the P5 hypercall numbering (planned contract,
`docs/stages/p5/plans/p5-w02-hypercall-abi-error-boundary.md`) ever assigns a
number inside that namespace, the collision is resolved by an explicit ABI
coordination decision — not by re-numbering in code — and is labeled
`ADR Required` if it forces either numbering to change.

## 2. Version and features contracts

### 2.1 PSCI_VERSION

```text
Name and stability: psci_version(cfg: &VmPsciConfig) -> u32 — internal
Purpose and caller: first call Linux makes when probing firmware; answers from
  the frozen machine fact, never from Host firmware
Inputs: VmPsciConfig
Outputs: encoded version (major.minor.patch per DEN 0022 layout) frozen by W02
Preconditions: cfg.reported_version is an approved machine fact
Postconditions: pure function; no state change; identical across all vCPUs of
  the VM and across repeated calls
State/ownership change: none
Concurrency/allocation: none (no locks, no allocation)
Errors: none (always succeeds)
Security checks: none required (no Guest input); Host PSCI version must never
  be read on this path (decision D6)
Logic: return cfg.reported_version
Validation: P8-V09 S1; W14 compatibility row (version drift detection)
```

### 2.2 PSCI_FEATURES

```text
Name and stability: psci_features(cfg: &VmPsciConfig, fid: u32) -> PsciStatus — internal
Purpose and caller: Linux probes each function before use (v1.x behavior);
  must agree exactly with the implemented set
Inputs: candidate function ID from Guest x1 (untrusted)
Outputs: SUCCESS (bit-0 feature flags, always 0 for this subset) for a
  member of the frozen subset; NOT_SUPPORTED for everything else
Preconditions: cfg.implemented matches the frozen subset exactly
Postconditions: pure function; no state change
State/ownership change: none
Concurrency/allocation: none
Errors: NOT_SUPPORTED is a normal response, not a failure
Security checks: reserved-bit rule — non-zero bits in x1 outside the function
  ID field -> INVALID_PARAMETERS (DEN 0022 requirement, enforced here so the
  lifecycle handlers never see dirty IDs)
Logic (pseudocode):
  fn psci_features(cfg, fid) -> PsciStatus:
      if reserved_bits_set(fid, mask := FUNCTION_ID_ONLY): return INVALID_PARAMETERS
      if cfg.implemented.contains(PsciFunctionId::decode(fid)): return SUCCESS(0)
      return NOT_SUPPORTED
Validation: P8-V09 S1 probe matrix (every function in DEN 0022 probed; only
  the frozen subset reports SUCCESS); W18 probe-abuse row (wildcard IDs)
```

## 3. Validation helper contract

```text
Name and stability: validate_psci_target_mpidr(vcpu, raw_mpidr) -> Result<&Vcpu, PsciStatus> — internal
Purpose and caller: canonical VM-scoped target resolution for CPU_ON and
  AFFINITY_INFO
Inputs: raw 64-bit MPIDR-shaped value from Guest registers (untrusted)
Outputs: reference to the target vCPU of the calling VM, or a spec error
Preconditions: VM topology (vCPU id -> virtual MPIDR map) exists and is immutable
Postconditions: resolution considers only the calling VM's topology; never any
  pCPU or Host CPU identity
State/ownership change: none (read-only topology lookup)
Concurrency/allocation: lock-free read of immutable topology; no allocation
Errors: INVALID_PARAMETERS for out-of-topology values and non-zero reserved
  affinity bits that the machine contract forbids; policy outcomes
  (self-target, already-on) belong to the CPU_ON handler, not this helper
  (it resolves any well-formed topology member)
Security checks: this is the single point that prevents cross-VM addressing
  (ADR §19); callers must not re-implement topology comparisons
Logic (pseudocode):
  fn validate_psci_target_mpidr(vcpu, raw) -> Result<&Vcpu, PsciStatus>:
      if !machine_mpidr_form(raw):            return Err(INVALID_PARAMETERS)
      t = vcpu.vm.topology.by_virtual_mpidr(raw) or return Err(INVALID_PARAMETERS)
      return Ok(t)
Validation: P8-V09 S5 (bad MPIDR rows: other-VM-shaped, malformed, reserved
  bits); reused by W18 topology-abuse scenarios
```

## 4. Telemetry event contract

```text
Name and stability: record_psci_event(vm, fid, outcome) — internal
Purpose and caller: structured observability for every PSCI dispatch outcome;
  consumed by W17 baselines and W13 diagnostics
Inputs: VM, decoded function ID, outcome class {accepted(error-code),
  rejected(code), internal-fault}
Outputs: one structured event on the P0 trace-event namespace (PSCI events get
  a reserved sub-namespace; exact IDs per the P0 trace contract)
Preconditions: telemetry subsystem initialized
Postconditions: counters bounded and wrap-tolerant; no unbounded buffering;
  events must be compile-time prunable and runtime-filterable (ADR-048)
State/ownership change: per-VM counters only (01 §2)
Concurrency/allocation: must be safe in VM-exit context; no allocation on the
  hot path; dropped events are counted, never blocking
Errors: telemetry failure never changes the Guest-visible PSCI response
Security checks: events contain no Guest memory contents; MPIDR values logged
  are the virtual ones
Logic: increment counter; emit event if enabled by runtime filter
Validation: W17 baseline consumes event presence/shape; review that failure
  cannot alter responses
```

## 5. Malformed-input acceptance scenarios (contracts for W18/W16)

Each scenario is a declared contained outcome; none may panic the Hypervisor,
touch Host state, or affect another VM:

| Scenario | Guest action | Required response |
|---|---|---|
| S5a unknown ID | HVC with unmapped PSCI-namespace ID | `NOT_SUPPORTED`; VM continues |
| S5b feature-probe abuse | PSCI_FEATURES over random IDs including reserved bits | `NOT_SUPPORTED` / `INVALID_PARAMETERS`; VM continues |
| S5c bad target | CPU_ON with MPIDR outside topology | `INVALID_PARAMETERS`; VM continues |
| S5d bad entry | CPU_ON with entry IPA outside Guest RAM | `INVALID_ADDRESS`; VM continues |
| S5e self/conflicting on | CPU_ON of the calling (online) vCPU or an already-pending vCPU | `ALREADY_ON`/`INVALID_PARAMETERS` per §0.2; VM continues |
| S5f SYSTEM_OFF abuse | SYSTEM_OFF from any vCPU of a multi-vCPU VM | VM-scoped powerdown only (03 §5); Host and other VMs unaffected |

## 6. Open items owned elsewhere

- Frozen values (reported version, subset membership, conduit) — W02 gate.
- DTB `arm,psci` node contents — W04.
- Hypercall/PSCI namespace collision — prerequisite failure boundary (§1
  note); escalate to `ADR Required` only if it forces an ABI change.
- Linux-side PSCI driver behavior — cross-reference only (ADR §20); no
  Linux modification is authorized.
