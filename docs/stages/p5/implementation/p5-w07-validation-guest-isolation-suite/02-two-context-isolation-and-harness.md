# P5-W07 Two-Context Isolation and Harness Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W07 detailed design](README.md).

## 1. Logical artifact groups

W07 is a validation package: its logical modules are authoritative artifact
groups, not runtime subsystems.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| P5 scenario set in the Validation Guest asset | W07 (this design) + the asset's implementing agent | this matrix, W02–W06 delivered contracts | Guest code emitting the declared markers; it does not define hypervisor behavior or new ABI |
| Two-context boot composition | W07 | P4 boot path, W05 bootstrap-grant contract | two contexts with disjoint authority; it does not create a multi-VM management facility |
| Marker grammar and expectations | W07 | W06 category table | the machine-matchable expectation set; it does not carry telemetry counters (W09) |
| Harness expectation file and run procedure | W07, extending the P4-W08/P0-W09 entry point | marker grammar, scenario order | per-scenario verdicts; it does not change CI policy or the QEMU runner's architecture |
| Run records | verification record (created when evidence exists) | actual commands and output | run/not-run evidence; not part of the design |

## 2. Two-context isolation setup (P5VG-100–103)

### 2.1 Context model

- **Context A** and **Context B** are two independent bootable instances of
  the Validation Guest on the evidenced P4 boot path, single vCPU each,
  booted in the same declared QEMU session with identical images except the
  per-instance scenario program selected by a build-time constant.
- Each context is bound, by the Hypervisor-side test bootstrap using W05's
  explicit initial-grant contract, to exactly one capability over its own
  test object: A over A0, B over B0. The grants are explicit and
  Hypervisor-authored; nothing about either Guest's identity, order, or VM
  ID confers authority by itself.
- Contexts are otherwise equal: no ordering, no "first VM" status, no
  role difference. This equality is what makes the isolation evidence
  meaningful under INV-P5-05/INV-P5-10.

### 2.2 How B knows A's raw value (deterministic test constants)

The P5 test bootstrap assigns **deterministic test-bootstrap slot values**
documented in the asset: for a fresh boot of the declared composition, the
handle value of A0 is a fixed build-time constant of the scenario set (the
W04 handle representation plus the fixed test slot and the initial
generation). B's scenario program embeds that constant. This is a test
convention owned by this design — explicitly not ABI, not a stability
promise, and re-derived if the delivered W04 representation changes.

If the delivered W04 representation makes the value non-derivable at build
time, the fallback is harness-mediated value passing: A prints its handle
value (opaque value disclosure to the harness is acceptable — it is
Guest-known data, never a Host pointer), the harness injects it into B's
input registers at B's load step, and the run record documents the passing.
The deterministic-constant form is preferred because it keeps scenarios
self-contained.

### 2.3 The isolation scenarios

| ID | Input / precondition | Expected marker | Pass condition | Proves / does not prove |
|---|---|---|---|---|
| P5VG-100 | A queries A0 (own authority) | `Completed` | A's own use succeeds in the same boot | baseline sanity; nothing about isolation yet |
| P5VG-101 | B queries A0 using A's raw handle value (constant per §2.2), B holds no capability for A0 | `NoAuthority` | the valid, live, known value is denied across contexts | caller-associated authority (INV-P5-05): knowing a value confers nothing; the core P5-V12 proof |
| P5VG-102 | Negative control: B queries B0 (own authority) in the same boot | `Completed` | B's denial in P5VG-101 is authority-based, not value- or context-broken | the denial is attributable to missing authority, not to B being misconfigured |
| P5VG-103 | Repeat of P5VG-100–102 across the declared repeat boots (see §4) | same classes each boot | isolation result is stable across declared repeats | repeatability of the isolation property; not behavior under concurrent multi-VM load |

The set deliberately proves the strongest useful statement available in P5:
a second, equally ordinary context cannot exercise another's authority even
with full knowledge of its raw reference value. It does not prove
memory-isolation mechanisms (P4 evidence), scheduling isolation (P7), or any
policy about many-VM systems.

**Failure boundary:** if the delivered P4 foundation cannot boot a second
instance, or W05's delivered grant contract cannot produce two disjoint
authorities, the affected rows are **blocked prerequisites** and an
`Architecture Change Request` is recorded. A within-one-VM second context,
or any simulated substitute, must not be presented as P5-V12 evidence.

## 3. Marker grammar and marker discipline

- **Grammar:** `P5VG/<scenario-id>/<outcome-class>` on the P4-established
  Guest debug channel, one marker per scenario conclusion, emitted in
  scenario-ID order within a context. Scenario-specific fields, where needed
  (P5VG-002's attribute echo), are fixed-format, bounded, and declared in
  the matrix row — never free text, never addresses.
- **No disclosure:** markers must not contain Host virtual or physical
  addresses, Host object pointers, Guest buffer contents, or capability
  internal state. Raw handle values are Guest-known data and may appear only
  where a row explicitly requires them (the §2.2 fallback). The discipline
  is reviewed per W07-DV07 and preserved by W09's redaction rules.
- **No ABI promise:** markers, scenario constants, and scenario order are
  test conventions of this asset. No document, marker, or constant may be
  presented as a machine ABI, management ABI, or Guest SDK surface.
- **Determinism:** a scenario's expected class is fixed before its first
  run; a run never renegotiates an expectation. If the delivered mechanism
  makes an expectation wrong, the matrix is amended through this design's
  record with rationale, and the amendment is visible in review — not
  silently at match time.

## 4. Harness contract

The harness extends the automation entry point delivered by
[P4-W08](../../../p4/plans/p4-w08-qemu-integration-regression.md) on the
[P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md) QEMU
basis; it does not redesign that runner.

```text
Interface: P5 scenario run (host-side procedure; exact command spelling is
implementation-record material, not contract).
Inputs: built hypervisor + Validation Guest assets; the expectation list
        derived from 01 and this file (scenario order, expected class,
        repeat count); declared environment (QEMU version, CPU count, boot
        composition).
Behavior: boot the declared composition; A runs its scenario sequence, then
        B runs theirs (or the declared interleaving for P5VG-103 repeats);
        capture the debug channel; match markers against expectations in
        order.
Outputs: one verdict per scenario — passed / failed / not-run — plus a
        suite verdict; recorded with command, environment, timestamps.
Timeouts: every boot and every scenario waits under a declared timeout;
        expiry is a diagnosable non-success, never a hang presented as a
        pass.
```

Non-success classes (each is a first-class recorded outcome, mirroring the
regression rule W09 inherits):

| Class | Meaning | Example trigger |
|---|---|---|
| `marker-mismatch` | a marker arrived with a different class than expected | scenario produced `Malformed` where `NoAuthority` was expected |
| `marker-missing` | scenario sequence ended without the expected marker | Guest stopped early |
| `unexpected-output` | debug-channel content outside the marker grammar | disclosure or debug noise — triggers the marker-discipline review |
| `guest-fault-unexpected` | Guest faulted where the scenario expected a contained marker | hypervisor panic adjacent behavior |
| `hypervisor-failure` | Hypervisor panic, hang, or invariant record during the suite | containment defect — highest-priority failure |
| `timeout` | declared wait expired | environment or defect |
| `environment-unsupported` | declared environment unavailable | recorded; never converted to a pass |

**Pass condition for the suite:** every in-scope scenario passed; every
non-passed scenario is recorded with its class and diagnosis; no
`hypervisor-failure` occurred in any boot.

## 5. Test-asset maintenance ownership

- The Validation Guest remains a maintained test asset per the P4-W05
  handoff; W07 owns the P5 scenario extension: adding, amending, or retiring
  a scenario requires a matrix row, marker, expectation, and a record
  rationale in the same change.
- Scenario additions for future object classes are Reserved; they follow the
  same rule and must not turn the asset into a general Guest program.
- The asset must not accumulate production features (protocols, loaders,
  configuration parsing); the review gate is the plan's out-of-scope list.
- W09 consumes markers read-only; it must not alter expectations to make a
  regression pass. W10 links the inventory as evidence; it must not freeze
  the conventions as contracts.
