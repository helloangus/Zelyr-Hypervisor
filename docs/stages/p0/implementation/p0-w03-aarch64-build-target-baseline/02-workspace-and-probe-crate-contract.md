# P0-W03 Workspace and Probe-Crate Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W03 detailed design](README.md).

## 1. Scope of this file

This file contracts the only code-bearing artifacts of W03: the root workspace
manifest, the member manifest, and the two build-chain placeholder symbols. It
contains contract outlines and pseudocode, not runnable production code; the
member exists to prove the target build chain and owns no runtime behavior.

## 2. Manifest contracts

### 2.1 Workspace manifest (root `Cargo.toml`)

```toml
[workspace]
resolver = "3"             # the resolver matching the pinned toolchain's current
                           # stable edition default; if cargo emits a resolver
                           # recommendation at build time, resolve it at
                           # implementation time and record the final value
members = [
    "hypervisor",
]
# no [workspace.package]: introduced when a second member needs shared keys
# no [workspace.dependencies]: the workspace has zero dependencies (W18 owns
#   the first dependency decision)
# no [profile.*]: profile governance is W04's; built-in dev/release only
```

Field rules:

- The manifest is a virtual workspace (no root package). This keeps crate
  boundaries open, matching ADR-046's workspace-without-over-splitting rule and
  the task book's requirement that P0 not assert final crate boundaries.
- Membership changes are made only through an approved design; W03 records the
  single-member baseline and nothing more.
- The manifest must not contain features, patches, registry settings, profile
  overrides, or per-member build configuration.

### 2.2 Member manifest (`hypervisor/Cargo.toml`)

```toml
[package]
name = "hypervisor"        # matches the tracked directory; see the ADR-054 note
version = "0.1.0"          # placeholder value; identity semantics are W16/W17 scope
edition = "2024"           # confirmed against the pinned toolchain at implementation
# no dependencies, no build-dependencies, no build script, no features
```

Field rules:

- `name` follows the existing tracked `hypervisor/` directory from the W01
  baseline. The pending project-name/crate-prefix decision (ADR-054, register
  待定) is not resolved by this choice; a future formal-name freeze makes this
  rename routine maintenance recorded in the implementation record.
- `version` must exist for Cargo but asserts no identity contract; W16 and W17
  own artifact identity and naming semantics.
- `edition` uses the current stable edition supported by the pinned toolchain;
  the implementing agent confirms the pinned `rustc` accepts it and records
  the confirmed value.
- No `[lib]`, `[[bin]]`, or path overrides: the default `src/main.rs` binary is
  the baseline artifact source.

## 3. Placeholder-symbol contracts

The member contains exactly the crate-level attributes and the two symbols
below. Everything else is out of scope.

### 3.1 Crate-level contract

```text
Name and stability: hypervisor member root source (src/main.rs); internal; expected to be
    restructured by P1's entry design.
Purpose and caller: proves the freestanding no_std build; not called by anyone.
Inputs / outputs: none.
Preconditions / postconditions: none beyond compilation.
Attributes: #![no_std] and #![no_main] only. No #![feature]; no lint allows or
    denies (W07 owns warning policy; the member must be warning-free under
    defaults). A custom panic handler is defined per §3.3.
State and ownership change: none.
Concurrency/allocation context: no allocation anywhere; the member must not
    link an allocator.
Errors and failure guarantee: none — the member has no inputs and no I/O.
Security/authorization checks: none — no untrusted input exists in P0.
Logic: attribute declarations plus the two symbols in §3.2/§3.3.
Validation: target build (W03-DV02); scope review (W03-DV04).
```

### 3.2 Entry stub `_start` (assembly)

```text
Name and stability: `_start`, #[no_mangle], extern "C"; internal build placeholder;
    replaced by the P1 EL2-entry design.
Purpose and caller: satisfies the freestanding link requirement for an entry
    symbol; "called" only by the machine reset/firmware dispatch, never by Rust code.
Inputs / outputs: none; diverges (never returns).
Preconditions: none may be assumed or established — no CPU mode, exception
    level, MMU state, stack validity, or memory state is asserted, and the stub
    must not touch memory or require a stack.
Postconditions: the executing core parks in a low-power wait loop; no memory
    is written; no register state is interpreted.
State and ownership change: none.
Concurrency/allocation context: none; single-core parking, no synchronization.
Errors and failure guarantee: none — the stub cannot fail; it performs no I/O.
Security/authorization checks: none.
Logic (pseudocode, assembly):
    global_asm!(start of module-assembly input)
    _start:
      loop { wfi }        // wait-for-interrupt park; no sysreg read or write,
                          // no vector table, no MMU/cache work, no console
    global_asm!(end of input)
Validation: a successful member build (the assembly input is assembled as part
    of it); optional informative disassembly check.
```

### 3.3 Panic handler placeholder

```text
Name and stability: the member's #[panic_handler] function; internal placeholder;
    semantics owned by W14 (failure classification) and W12 (crash information).
Purpose and caller: satisfies the no_std requirement that exactly one panic
    handler exists; invoked by the core library on panic.
Inputs / outputs: &core::panic::PanicInfo -> never returns (diverges).
Preconditions: none.
Postconditions: never returns; parks the core (infinite loop).
State and ownership change: none; must not allocate, must not unwind, must not
    touch memory-mapped devices or format the PanicInfo (no output channel
    exists in P0, and emitting one would pre-empt W12).
Concurrency/allocation context: no allocation; no locking; reachable from any
    context by definition, hence minimal.
Errors and failure guarantee: the handler is the last-resort park; it cannot
    itself fail.
Security/authorization checks: none; the handler must not inspect or trust the
    payload.
Logic (pseudocode):
    #[panic_handler]
    fn baseline_panic(_info: &PanicInfo) -> ! { loop {} }
Validation: target build; review that no formatting, I/O, or policy decision
    is present (W03-DV04).
```

Non-responsibility of both symbols: they define no EL2 entry contract, no
exception behavior, no vector capture, no diagnostic output, and no failure
classification. Any extension of their bodies during P0 is a scope conflict
against the P0-W03 plan's out-of-scope list and is stopped at review.

## 4. Build invocation and artifact boundary

Suggested observation (not a gate; W07 owns gate spelling):

```text
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
file target/aarch64-unknown-none-softfloat/<profile>/hypervisor
```

Contract: the invocation succeeds from a clean provisioned environment
(toolchain and target restored through the W02 manifest), the artifact exists
under `target/aarch64-unknown-none-softfloat/<profile>/`, and `file` (or an
equivalent local tool; availability is machine-local convenience, not a project
prerequisite) reports an AArch64 ELF image. The artifact is generated output:
untracked, not executed on the development host, and not consumed by host-class
builds. A second invocation with no source change must be an incremental no-op
success (W03-DV03).

A bare host invocation (`cargo build` with no `--target`) is expected to fail
while the workspace contains only this freestanding member; that is the
documented single-class boundary of
[the target-baseline contract](01-target-baseline-contract.md) §7, resolved
when W08 adds the first host-class member. It must not be "fixed" by making the
member host-buildable — that would reintroduce a std panic handler conflict and
blur the class separation.
