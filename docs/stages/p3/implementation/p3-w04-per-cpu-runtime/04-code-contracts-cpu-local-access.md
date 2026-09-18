# P3-W04 Code Contracts — CPU-Local Access and Installation

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W04 detailed design](README.md).

Contracts follow the project function/type template. The register
convention (`TPIDR_EL2` holds the current CPU's `PerCpuArea` address) is
this design's stage-local architecture decision (README decision 2) with
its recorded prerequisite-compatibility boundary.

## 1. The locality rule (binding)

A CPU's own area is discovered **only** by reading its CPU-local register.
No code path may find "its own" area by searching the lookup table,
reading a global variable, or inferring from execution order — any such
path re-creates the implicit global-current-CPU contract the plan
prohibits. The lookup table exists for cross-CPU *read-only* access
(diagnostics, audits, tests) and for no other purpose.

## 2. `current()`

```text
Name and stability: current() -> &'static PerCpuArea — internal safe
    wrapper; the only sanctioned locality accessor.
Purpose and caller: gives any consumer its own area; callers: W02 entry
    tail (post-install), W05 signal, W07/W08/W09/W11 consumers, P4 (via
    the reserved capacity).
Inputs / outputs: none; returns a shared reference to the caller's area.
Preconditions / postconditions: precondition — the calling CPU completed
    installation (`install_state == Installed`); before installation the
    accessor must not be called (W02's entry path orders this). The
    `'static` lifetime is sound because areas are never freed or
    relocated (README decision 4).
State and ownership change: none.
Concurrency/allocation context: one register read; no allocation; no
    synchronization (the area is CPU-private by construction).
Errors and failure guarantee: no error path; a call before installation
    is a sequencing bug — debug-build validation (§4) makes it a loud
    failure, release builds trust the sequencing (documented trust
    decision, revisit trigger: any pre-install consumer appearing).
Security/authorization checks: n/a (not guest-reachable).
Logic (pseudocode):

    current():
        ptr = read_register(TPIDR_EL2)          # unsafe: arch boundary
        return &*(ptr as *const PerCpuArea)     # validity per §1 rule

    install_current(area):                       # §3; the only writer
        write_register(TPIDR_EL2, area)

Validation: W04-DV02 unit tests with a register-fake seam; W04-DV04
    review that no other locality source exists.
```

## 3. `install_local_state`

```text
Name and stability: install_local_state(area: &PerCpuArea) -> Result<(),
    InstallError> — internal; executed by a CPU for itself.
Purpose and caller: the exactly-once installation of a CPU's local
    environment; caller is W02's entry tail (secondaries) and the boot
    sequence (boot CPU, which passes its own area).
Inputs / outputs: the caller's pre-allocated area; Ok after the W05
    readiness signal is issued, Err(InstallError) naming the failed step.
Preconditions / postconditions: precondition — W03 eligibility for this
    CPU is `EligibleForLocalInstall` (the CPU's lifecycle state is
    Initializing; the boot CPU reaches that state via its own
    boot-variant transition before installing), checked first.
    Postconditions — header validated; register loaded; runtime stack
    active (secondary); `install_state == Installed`
    (release-stored); isolation diagnostic emitted; W05 signal delivered.
State and ownership change: area NotInstalled → Installing → Installed
    (CAS-guarded so a duplicate install attempt fails closed);
    provisional stack ownership per the W02 transfer contract.
Concurrency/allocation context: no allocation; no locks; the ordering of
    architecture §5 is binding (header before register; register before
    stack switch; release-store before signal).
Errors and failure guarantee: on any failure the CPU reports through the
    W02/W03 failure paths and never signals readiness; the area stays
    uninstalled and the CPU never becomes eligible.
Security/authorization checks: the eligibility gate is the authorization
    check (W03's boundary); a refused gate aborts installation.
Logic (pseudocode):

    install_local_state(area):
        if !lifecycle.eligibility(my_logical).allows_install():
            return Err(GateRefused)             # Initializing, boot CPU
                                                # included; my_logical is
                                                # topology.boot for it
        if area.install_state.swap(Installing, AcqRel) != NotInstalled:
            return Err(AlreadyInstallingOrInstalled)
        validate_header(area)?                  # magic, self_ptr, identity
        install_current(area)
        switch_to_runtime_stack(area)           # secondary only
        emit_isolation_diagnostic(area)         # identity + addresses
        area.install_state.store(Installed, Release)
        boot_gate.signal_local_init_complete(area.header.logical)  # W05
        return Ok(())

Validation: W04-DV02 (order and exactly-once), W04-DV04 (boundary).
```

## 4. Debug validation hook

```text
Name and stability: debug_validate_current() — validation-only helper;
    compiled out of release profiles per the P0 profile governance.
Purpose and caller: asserts the register points at a header whose magic,
    self_ptr, and install state are consistent; callers: tests, debug
    diagnostics, and the W09 diagnostic path may reuse the check.
Preconditions / postconditions: read-only.
Errors and failure guarantee: failed validation is a fatal diagnostic
    (corrupted locality is not recoverable).
Validation: W04-DV02.
```

## 5. `area_of` (cross-CPU read-only table access)

```text
Name and stability: area_of(logical: LogicalCpuId) -> Option<&'static
    PerCpuArea> — internal; cross-CPU diagnostic/audit access.
Purpose and caller: read-only inspection from another CPU (W12/W13
    tests, audits, fatal dumps).
Inputs / outputs: logical id; shared reference or None for an id outside
    the installed set.
Preconditions / postconditions: the table is published (post-global-init
    publication); the caller treats the area as unstable for anything
    except header/diagnostic reads (typed slots are CPU-private — reading
    them cross-CPU without the owning package's protocol is forbidden).
State and ownership change: none.
Concurrency/allocation context: acquire-load of the table entry; areas
    never freed, so the reference stays valid.
Errors and failure guarantee: None for unknown ids; never synthesizes.
Security/authorization checks: diagnostic-scope only; not a targeting
    mechanism (W07/W08 target through W03's online set and their own
    protocols).
Logic: table[logical.as_usize()] with a bounds check.
Validation: W04-DV04 (boundary: no self-lookup through this path in
    kernel code — enforced by review).
```
