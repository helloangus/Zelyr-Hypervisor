# P8-W18 Isolation Scenario Matrix

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W18 detailed design](README.md).

## 1. Logical artifact groups

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Security-property taxonomy (this file §2) | W18 (this design) | ADR-007, ADR §12, ADR §19 | registration scheme for rows; it does not redefine any ADR property |
| Scenario matrix (this file §3) | W18 (row set and oracles); expected-result *content* from W05/W06/W13 classifications (assumed contracts) | task book P8-V24 wording; plan scope | the declared abuse set; it does not implement triggers or containment |
| Host/cross-VM observations (this file §4) | W18 (boundary); P5-W07 dual-context suite via W19 (mechanism evidence) | W16 harness oracles | what "unaffected" observably means in P8 scope |
| Oracle definitions (this file §5) | W18 | W13 fault ledger; W16 fatal-class rules | when a row fails; it does not judge non-security qualities |
| Execution, classification, escalation | [02-workflow-oracles-and-handoff.md](02-workflow-oracles-and-handoff.md) | this file | run and evidence rules |

## 2. Security properties

Every row registers against exactly one primary property:

- **P-1 Host memory isolation** — a Guest access can never read or write
  memory outside its assigned Guest physical address space (ADR §12/§19;
  Stage-2 is the enforcement point).
- **P-2 Guest-visible surface integrity** — every operation outside the
  approved Guest-visible contract (unsupported sysreg, unassigned MMIO,
  malformed PSCI, topology tampering) produces the classified controlled
  outcome, never silent corruption or an implicit allow.
- **P-3 VM-scoped failure** — a Guest crash or fault is contained to its VM
  context: the Hypervisor stays alive and diagnosable, and later Guests are
  unaffected (the plan's "a Linux panic is never equivalent to a Hypervisor
  panic").
- **P-4 Resource liveness under abuse** — Hypervisor scheduling and
  interrupt service retain responsiveness and state integrity while a Guest
  monopolizes CPU or floods interrupts, within the declared P8 scheduling
  policy limits.

## 3. Scenario matrix

Column conventions: `Track` = LG (Linux), VG (Validation Guest), or LG+VG.
`Trigger` is owned by the W15 fixture and W13/VG declared triggers. Expected
results cite the owning classification (W05 behavior classes, W06 PSCI
boundary, W13 diagnostic classes). Repetition is a recorded run parameter
(no contract-fixed count); `seed` defaults to `none`. Evidence destination
for every row: `../../verification/assets/p8-w18/<run-id>/` transcripts plus
the per-row run record; the verification record holds run/not-run status.

| Row | Property | Track | Attack / fault vector | Input and precondition | Controlled expected result (content owner) | Containment expectation | Proves | Does not prove |
|---|---|---|---|---|---|---|---|---|
| ISO-01 | P-1 | LG+VG | Guest reads/writes IPA outside assigned Guest RAM: unmapped hole, reserved region, region above RAM top | trigger active in fixture; approved machine map | Stage-2 fault classified per W13; VM-scoped diagnostic with VM/vCPU/PC/syndrome/address context; Guest fault handling per W05 class | no Host memory access; EL2 continues; no fatal class | the P8 integration retains Stage-2 containment of illegal IPA under a real OS | not hardware Stage-2 behavior; not every illegal-address pattern |
| ISO-02 | P-2 | LG+VG | Access to unassigned MMIO: outside the declared console window, non-existent device slots, reserved device regions | trigger active; approved device map | controlled rejection/abort path per W05 classification; VM diagnostic; on VG track, the P4/P5 declared controlled-exit marker | no device-state corruption outside the VM; EL2 continues | unassigned device space is rejected, not passed through | not device-assignment security (Reserved, P14) |
| ISO-03 | P-2 | LG+VG | Trapped system-register abuse: accesses classified Unsupported or EL2-reserved; attempt to alter virtualization controls from EL1 | trigger active | W05 Direct/Emulate/Reject/Hidden/Unsupported classification produces its declared outcome; controlled diagnostic; no silent value change | no EL2 state leak or corruption; Guest sees only contract-visible effects | unsupported operations stay classified and controlled | not that the classification is complete for every register (W05 scope) |
| ISO-04 | P-2 | LG+VG | Malformed PSCI: invalid function ID, invalid target CPU ID, invalid entry address, wrong conduit | trigger active | standard PSCI error return or controlled rejection per the W06 boundary; vCPU lifecycle unchanged | no vCPU state corruption; no host CPU fact disclosure in the response | malformed firmware calls are rejected within the declared boundary | not PSCI implementation correctness generally (W06) |
| ISO-05 | P-2 | LG+VG | Topology tampering: PSCI CPU_ON of a non-enumerated CPU ID; post-boot write to the Guest DTB region; GIC/register access outside the Guest view | trigger active; approved DTB and topology | rejection with VM diagnostic per W06/W07 boundaries; DTB region protected per approved reserved-region rules | enumerated topology unchanged; no host topology disclosure | the approved Guest-visible topology is not Guest-mutable | not that all Guest-visible state is immutable (contract-scoped) |
| ISO-06 | P-3 | LG+VG | Guest crash: declared Linux kernel-panic injection; VG deliberate fatal fault (P4 controlled illegal behavior) | trigger active; clean baseline first | Guest-scoped failure; W13 diagnostic context (PC/PSTATE, syndrome, exit reason, recent trace); EL2 alive; console diagnostic usable | EL2 never panics; no fatal class; subsequent clean boot succeeds in-session | a crashing Guest cannot take the Hypervisor down | not crash *analysis* completeness (W13 limit); not Host kernel resilience |
| ISO-07 | P-4 | LG+VG | Infinite loop: vCPU executes a never-yielding CPU-bound loop | trigger active; declared scheduling configuration | loop continues until declared termination; other declared entities and Host-side control path remain responsive per the W10/W11 scheduler expectations | scheduler retains control within the declared P8 policy; no lockup class in the W13 ledger | CPU monopolization does not remove Hypervisor control | not real-time bounds, not fairness, not behavior beyond declared M:N limits |
| ISO-08 | P-4 | VG (mechanism) + LG where the fixture declares a storm workload | Interrupt storm: tight self-IPI/SGI loop; repeated timer re-arm; declared storm profile | trigger active; storm duration declared and recorded | storm proceeds for the declared duration; core state preserved (P6 exit-criterion inheritance); IRQ telemetry remains producible; diagnosable via W11/W13 | host IRQ path stays live; no lost core state; storm ends by declared termination | interrupt flooding does not corrupt or deadlock the interrupt subsystem | not unlimited-duration storms; not hardware interrupt-controller behavior |

Rules: a row must not combine vectors; combined-attack exploration is
Reserved. Any new row follows this matrix's column discipline and is added
only by reviewed edit of this design. Blocked rows (missing trigger, missing
approved map) stay listed with their blocker — deletion is prohibited
(same rule as W16 §5).

## 4. Host and other-context observations

### 4.1 Host-protection oracle set

Applied to every row, evaluated by the harness from the W16 fatal-class rules
plus these W18-specific observations:

1. **EL2 liveness:** Hypervisor diagnostics and telemetry remain producible
   during and after the row; no EL2 panic/fatal-class output occurs.
2. **Control-path responsiveness:** the harness control channel (including
   the W09 console ownership boundary) remains serviceable; the row must not
   be able to capture or silence Host serial ownership (P8-V12 containment
   inheritance).
3. **No Host disclosure:** no Host physical address, Host IRQ, board/SoC
   identity, or Host firmware fact appears in any Guest-visible output
   (aligns with W04 host-leak and W13 diagnostic-content rules).
4. **No residual authority:** after the row, the Guest holds no capability,
   mapping, or handle it did not legitimately hold before (per the evidenced
   P5 authority model); observed via the retained P5 suite invariants and
   the post-row state check.

Any observation failing is a row failure regardless of the Guest-visible
outcome.

### 4.2 Other-VM and other-vCPU containment in P8 scope

P8's approved Linux scope is a single Linux VM; P8 therefore **cannot and
does not claim** multi-VM co-existence containment. The P8-V24 "other VM"
clause is discharged, within P8 scope, by:

1. **Other-vCPU observations** (rows ISO-07/ISO-08 on multi-vCPU
   configurations): vCPUs other than the abusing one continue their declared
   workloads per the W10/W11 expectations; no cross-vCPU state corruption
   class appears in the W13 ledger.
2. **In-session recovery observation:** after each abuse row, a subsequent
   clean Guest boot succeeds in the same harness session (ISO-06's
   containment expectation, applied as a general post-row check for
   state-leaking rows), demonstrating no persistent Host/Hypervisor damage.
3. **Mechanism-level dual-context evidence:** the retained P5 dual-context
   suite (one context cannot use another's authority), executed via
   [W19](../p8-w19-validation-guest-dual-track/README.md), remains the
   authority-isolation evidence; P8 records it as inherited mechanism
   evidence, not as new P8 observation.

Full multi-VM containment on one Hypervisor instance is **Reserved**
(trigger: P10+ management-domain work) and must not be implied by any P8
artifact.

## 5. Crash/failure oracles

A row **fails** when any of the following occurs; each is a distinct, named
oracle recorded in the run record:

- **ORA-PANIC:** Hypervisor panic, fatal EL2 exception, or any fatal-class
  entry in the W13 fault ledger.
- **ORA-HANG:** harness timeout without the row's declared controlled
  outcome (distinguishes "contained" from "stopped by the clock").
- **ORA-CLASS:** a fault class appears that the W13 ledger does not classify
  for this vector, or the declared expected class does not appear.
- **ORA-LEAK:** any §4.1 observation fails (liveness, responsiveness, Host
  disclosure, residual authority).
- **ORA-CROSS:** a later scenario in the same session fails in a manner the
  record attributes to state left by this row (cross-scenario
  contamination).
- **ORA-ESCALATE:** the outcome required Guest-scoped failure but instead
  silently continued (a rejected operation taking effect is worse than a
  crash and is an explicit failure).

A Guest crash inside the attacked VM is an expected controlled outcome for
ISO-06 (and permitted for ISO-01/02/03/05 per their W05 classifications);
it is never by itself a row failure. Row repetition follows the declared
per-row parameters; re-running a failed row to obtain a pass is prohibited
(identical rule to W16 §5).
