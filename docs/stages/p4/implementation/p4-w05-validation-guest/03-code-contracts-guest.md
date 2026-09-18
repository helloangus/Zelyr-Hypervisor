# P4-W05 Code Contracts — Validation Guest

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W05 detailed design](README.md).  
**Companion:** module map and scenario table in
[02-guest-architecture-and-scenarios.md](02-guest-architecture-and-scenarios.md).

Guest-side contracts are Guest-crate APIs (internal to the Guest binary);
the host-side scenario-validation contract is cited to
[P4-W04](../p4-w04-vcpu-entry-exit/README.md). All names are P4-internal and
unstable-by-declaration. Pseudocode is an outline; the Coding Guidelines
govern final Rust shape. The Guest is `no_std` with no allocator.

## 1. `_start` entry contract (module `entry`)

- **Name and stability:** `_start` (linker entry symbol at image base;
  naked function). Internal.
- **Purpose and caller:** first Guest instruction after `ERET`; caller is
  the W04 entry path (W04 §3 contract).
- **Inputs:** x0 = validated scenario id; SP = stack top; PC = image base
  (hardware); PSTATE = EL1h, DAIF masked.
- **Preconditions:** W04's construction guarantees all of the above; the
  function re-asserts SP defensively (set SP to the boot-info-provided
  value as its first actions — defensive, and it makes the stack provenance
  explicit).
- **Postconditions:** runtime initialized; boot-info validated or
  `VG-FAIL:BOOTINFO` halt; scenario dispatched.
- **Errors:** boot-info failure and non-EL1 detection emit `VG-FAIL:*` and
  halt; nothing else can fail before dispatch.
- **Security posture:** the Guest trusts nothing not covered by the entry
  convention; it validates even host-authored input (D3 of
  [01 §4](01-scope-and-foundations.md)).
- **Logic:**

```text
_start:                          // naked
    sp = STACK_TOP_FROM_LAYOUT   // from bootinfo location convention
    scenario_id = x0
    if CurrentEL() != EL1: emit VG-FAIL:EXCEPTION-LEVEL; halt()
    boot = bootinfo::validate() else { emit VG-FAIL:BOOTINFO; halt() }
    console::banner(PROTOCOL_VERSION, boot.layout_version, scenario_id)
    scenario::dispatch(scenario_id, boot)        // never returns
```

- **Validation:** on-target VG-001 (P4-V05); boot-info corruption injection
  (host-side test of the validator logic where host-testable, else
  on-target failure-marker evidence).

## 2. `bootinfo::validate` contract

- **Name and stability:** `fn validate() -> Result<BootInfoView, BootInfoError>`.
  Guest-internal.
- **Purpose and caller:** defensive validation of the W03-written block;
  called once by `_start`.
- **Inputs/outputs:** block bytes at the fixed boot-info IPA → checked
  accessor view or error.
- **Checks, in order:** magic word; protocol/layout version acceptance rule
  (exact match for P4); declared block size within bounds and matching the
  IPA window; checksum over the declared span; per-field bounds (RAM size,
  image size, addresses within declared RAM; probe/window addresses within
  RAM or explicitly outside it as designed).
- **Postconditions:** `BootInfoView` accessors return only field values that
  passed bounds checks; no accessor performs arithmetic that can overflow
  (checked conversions at validation time).
- **Errors:** `BadMagic`, `BadVersion`, `BadSize`, `BadChecksum`,
  `FieldOutOfBounds` — each mapped by the caller to `VG-FAIL:BOOTINFO` (one
  marker; the detailed reason goes into the reason field).
- **Security:** this is the Guest's only untrusted-input boundary in P4;
  the failure mode is fail-closed (halt), never best-effort continuation.
- **Logic:**

```text
validate():
    raw = bytes_at(BOOT_INFO_IPA)                // the built-in constant
    check magic(raw) else BadMagic
    check version(raw) == ACCEPTED else BadVersion
    check size(raw) <= WINDOW and size(raw) >= MIN else BadSize
    check checksum(raw[..size]) == raw.checksum else BadChecksum
    fields = decode(raw, checked conversions) else FieldOutOfBounds
    check fields.ram_window_bounds() else FieldOutOfBounds
    return BootInfoView(fields)
```

- **Validation:** host-side unit tests (validator compiled for host in a
  test harness where feasible) covering every error class; truncation and
  bit-flip property checks; on-target corrupted-block injection via W08
  tooling if available.

## 3. Console contracts (module `console`)

### 3.1 `write_line`

- **Name and stability:** `fn write_line(bytes: &[u8])`. Guest-internal.
- **Purpose and caller:** line-oriented console output; called by banner,
  markers, failure paths.
- **Preconditions:** console page mapped Device RW (W03 D5); called at EL1
  during Guest execution (no output after a trigger —
  [02 §5](02-guest-architecture-and-scenarios.md) discipline).
- **Postconditions:** bytes plus CRLF visible on the reference console;
  polled transmit (no buffering beyond one byte; no interrupts — Guest
  DAIF is masked anyway).
- **Errors:** none (polled MMIO; a stuck console is a host-environment
  failure that automation detects by timeout, W08's domain).
- **Security:** output is data to the Host; the Guest cannot influence Host
  control flow through it.
- **Logic:** for each byte: wait transmit-ready; store byte. Append CRLF.
- **Validation:** VG-001 on-target; marker grammar conformance review.

### 3.2 Marker helpers

- **Name and stability:** `begin(id)`, `ok(id)`, `fault(id, kind)`,
  `fail(id, reason)`, `panic_marker(reason)` — thin wrappers fixing the
  grammar (`VG-<id>:<EVENT>`) so scenario bodies cannot misspell it.
  Guest-internal.
- **Validation:** grammar conformance as part of W08's expected-sequence
  matching; review that every terminal path emits before halting (D7).

## 4. Scenario contracts (module `scenario`)

### 4.1 `dispatch`

- **Name and stability:** `fn dispatch(id: ScenarioId, boot: &BootInfoView)
  -> !`. Guest-internal.
- **Purpose and caller:** route to the scenario body; caller `_start`.
- **Preconditions:** id validated host-side (W04 §2.1) and echoed in
  boot-info; the Guest re-checks id against its own table (defense in depth)
  and treats mismatch as `VG-FAIL:SCENARIO`.
- **Postconditions:** never returns; every path ends in a marker + halt or a
  marked trigger + hardware exit.
- **Errors:** none beyond the fail-marker path.
- **Logic:**

```text
dispatch(id, boot):
    match TABLE.get(id):
      None => { fail(id, "SCENARIO"); halt() }
      Some(body) => body(boot)
```

- **Validation:** table/id agreement test (Guest table vs W04 validation
  table — cross-checked in review; both derive from
  [02 §5](02-guest-architecture-and-scenarios.md)).

### 4.2 Scenario bodies (representative contracts)

Each body is a total function `fn(&BootInfoView)` with the shape of
[02 §4](02-guest-architecture-and-scenarios.md). Contracts for the fault
triggers (P4-D05/D07) in detail:

- **`vg004_unmapped_read(boot)`** — reads the boot-info probe address
  (guaranteed unmapped by the W02 mapping plan). Contract: emits
  `VG-004:FAULT:LOAD-UNMAPPED`, executes exactly one load from the probe
  address, nothing else afterward (control reaches EL2). Expected outcome
  per table: `Stage2Translation` exit.
- **`vg005_ro_write(boot)`** — single store to the read-only window base +
  fixed small offset (within the window). Expected: `Stage2Permission`
  (write) exit.
- **`vg006_xn_exec(boot)`** — forms a call target inside the XN data
  window and performs one indirect branch. Expected: `Stage2Permission`
  (execute) exit, distinguishable from VG-005 by syndrome access type.
- **`vg010_illegal(boot)`** — executes one architecture-guaranteed-undefined
  instruction word. Expected: `IllegalExecution` exit.
- **`vg011_svc(boot)`** — executes `SVC #0`. Expected: `UnknownSync`
  exit with full ESR retained.
- **`vg012_completion(boot)`** — banner + benign action + `VG-012:OK` +
  WFI. Expected: `Wfi` exit → `Stop(Controlled)`; teardown clean.

All bodies: no loops without progress, no memory access outside
boot-info-provided windows, exactly one trigger instruction for fault
scenarios (D5).

- **Validation:** per-scenario on-target evidence (P4-V05/V06/V08/V09 via
  W08); frame cross-checks (W04 DV09): captured Guest PC equals the marked
  trigger site for each fault scenario.

## 5. Halt and panic contracts

### 5.1 `halt` (module `halt`)

- **Name and stability:** `fn halt() -> !`. Guest-internal.
- **Purpose:** terminal Guest action for completion and failure paths;
  executes WFI (trap → W04 classifies → policy stop per W04 D7).
- **Preconditions:** markers already emitted (order discipline).
- **Postconditions:** control leaves the Guest via the classified WFI exit;
  no further Guest execution in P4.
- **Logic:** `loop { wfi() }` — architecturally the first WFI traps under
  W04's routing intent; the loop is defensive only.
- **Validation:** VG-007/VG-012 evidence.

### 5.2 `panic` handler

- **Name and stability:** `#[panic_handler] fn panic(info:
  &PanicInfo) -> !`. Guest-internal.
- **Purpose and caller:** Rust-level panic containment for the test asset;
  called by the runtime on panic paths (for example internal invariant
  failures in Guest code).
- **Postconditions:** `VG-PANIC:<SHORT-REASON>` emitted (best effort, no
  allocation/formatting beyond `fmt`), then `halt()`.
- **Security:** panic must not depend on possibly-broken state (no unwinding,
  no locks — none exist); it is the Guest's fail-closed path.
- **Validation:** deliberate-panic test scenario path in host-side Guest
  logic tests (where feasible); review that all Guest `unwrap`-class
  operations are absent per Coding Guidelines (the Guest avoids
  `unwrap`/`expect` entirely).

## 6. Host-side scenario-validation contract (cited, not implemented here)

- **Owner:** [P4-W04](../p4-w04-vcpu-entry-exit/README.md) §2.1
  (`construct` validates the scenario id against this design's table).
- **Contract:** the table of valid scenario ids and their meanings is
  owned by W05 ([02 §5](02-guest-architecture-and-scenarios.md), version
  `VG-T1`); W04 consumes it as data; both sides derive from the same table
  version recorded in the implementation records. A table change without a
  W04 consumer update is a review failure in whichever side changed first.
- **Validation:** cross-consistency review row in both packages' workflows.
