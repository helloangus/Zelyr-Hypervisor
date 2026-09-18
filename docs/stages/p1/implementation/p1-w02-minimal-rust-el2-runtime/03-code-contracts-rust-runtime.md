# P1-W02 Rust Runtime Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W02 detailed design](README.md).

All names are internal boot-scope items owned by this design; they are not
ABI and may be renamed only by a recorded design change. Pseudocode is an
outline, not runnable production code. No allocation exists anywhere; every
type is static or stack-local.

## 1. `el2_rust_entry` — Rust entry point

```text
Name and stability: el2_rust_entry; #[no_mangle] extern "C" fn(u64, u64, u64,
  u64) with a diverging body; internal; stable within P1.
Purpose and caller: the Rust entry establishment; the "Rust entry
  establishment" seam W09's §8 table names. Caller: the transfer of
  [02-code-contracts-entry-assembly.md](02-code-contracts-entry-assembly.md)
  §5 only.
Inputs / outputs: (x0, x1, x2, x3) forwarded by the assembly; x0 is the DTB
  physical pointer per W01 §4.1. No return (diverges into the sequencer and
  idle, or a terminal failure route).
Preconditions / postconditions: W01 tier passed (transfer guarantee);
  establishment stages 1–3 complete (stack, DAIF, BSS). On termination of
  normal execution the function has recorded `stable` and entered the
  controlled idle; it never returns to its caller.
State and ownership change: builds and publishes `BOOT_CONTEXT`; records the
  W09 `entry`/`runtime` events; performs no other machine-state change.
Concurrency/allocation context: boot CPU, DAIF masked, no allocation, no
  blocking.
Errors and failure guarantee: any failure inside the body routes through the
  early panic route with the phase attribution of the current tracker
  position (W09 H6); the function has no internal recovery path.
Security/authorization checks: none beyond the W01 tier's; the function
  consumes the guarantee rather than re-checking it.
Logic:
  el2_rust_entry(x0, x1, x2, x3):
    phase_enter(Entry); phase_complete(Entry)     # W01-guaranteed; W09 API
    phase_enter(Runtime)
    BOOT_CONTEXT.publish(BootContext::from_registers(x0, x1, x2, x3))
    # runtime readiness: identity linkable; panic route asserted ready (§3)
    phase_complete(Runtime)
    run_init_sequence()                            # W09; diverges or returns
    phase_enter(Stable)                            # W09 glue expectation
    controlled_idle()
Validation: W02-DV03 order review against
  [01-architecture-and-state.md](01-architecture-and-state.md) §2.
```

## 2. `BootContext` and `PhysAddr`

```text
Name and stability: PhysAddr(u64); internal newtype; Copy; stable within P1.
Purpose and caller: semantic wrapper for boot-scope physical addresses per
  the Coding Guidelines' newtype rule and the P0-W15 red lines. Callers:
  BootContext; later P1 designs that consume boot addresses.
Inputs / outputs: constructor from raw u64 is `const` and internal;
  accessors return the raw value only at audited boundaries (assembly
  handoff, MMIO constants) — none exist in P1 besides those already
  contracted.
Preconditions / postconditions: no arithmetic is defined on the type in P1
  (checked arithmetic enters with W08's mapping work; premature arithmetic
  would be unused surface).
State and ownership change: none (value type).
Concurrency/allocation context: no allocation; trivially copyable.
Errors and failure guarantee: cannot fail.
Security/authorization checks: the type exists precisely so addresses cannot
  silently mix with indices or lengths.
Logic: transparent u64 wrapper; explicit, documented conversion points only.
Validation: W02-DV05 review that no naked address integer is retained
  elsewhere.
```

```text
Name and stability: BootContext; internal struct; stable within P1;
  published once as static BOOT_CONTEXT.
Purpose and caller: retain the boot parameters for W03–W09 and the P2
  handoff. Callers: el2_rust_entry (publisher); downstream packages
  (readers).
Inputs / outputs: built from (x0, x1, x2, x3); fields:
  dtb: Option<PhysAddr>       — Some(x0) (x0 != 0 guaranteed by W01 T2);
                                None cannot be constructed from boot inputs
  reserved: [u64; 3]          — x1–x3, retained uninterpreted
  Provides read-only accessors; no mutators after publication.
Preconditions / postconditions: published exactly once, before
  Runtime.complete; every accessor's precondition is "published"
  (post-Runtime.complete), which downstream contracts inherit.
State and ownership change: publication is the only write; interior is
  immutable afterwards.
Concurrency/allocation context: single-writer boot context, no allocation;
  publication uses the audited once-publication boundary (same pattern as
  W03's report cell; SAFETY: single CPU, DAIF masked, one boot path).
Errors and failure guarantee: a second publication attempt is an invariant
  violation routed via the panic route (phase-attributed `runtime`).
Security/authorization checks: the retained values originate from the boot
  contract's trusted boundary (W01); no downstream consumer may treat them
  as guest-influenced data in P1.
Logic: from_registers maps x0 through the T2 guarantee; publish writes the
  static and sets the published flag inside the audited boundary.
Validation: W02-DV03; W03/W09 consumability reviews.
```

## 3. Runtime readiness steps

```text
Name and stability: the readiness assertions between Runtime.enter and
  Runtime.complete; internal functions or inline steps; stable within P1.
Purpose and caller: make stages 7–8 of the establishment order explicit and
  reviewable. Caller: el2_rust_entry only.
Inputs / outputs: none; they assert and return.
Preconditions / postconditions: identity data linkable (the static
  BuildIdentity resolves to a printable form — see
  [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md));
  panic route ready (handler linked — true by construction once the crate
  links — and the early writer's UART constant resolves). Assertions never
  mutate machine state.
State and ownership change: none.
Concurrency/allocation context: no allocation, no I/O.
Errors and failure guarantee: a failed readiness assertion is an invariant
  violation routed via the panic route before Runtime.complete, so W09's
  matrix attributes it to phase `runtime` with the panic route — the
  route-exists-before-it-is-needed property of W09 H4.
Security/authorization checks: none.
Logic: check BuildIdentity resolves; check the early-writer constant is
  linked; panic otherwise.
Validation: W02-DV03/DV04.
```

## 4. Sequencer seam

```text
Name and stability: the seam arrangement: run_init_sequence() (W09-owned),
  followed by the W02-owned glue (phase_enter(Stable); controlled_idle()).
  Internal; stable within P1.
Purpose and caller: hand the established runtime to the W09 lifecycle and
  reach the terminal regime only through it. Caller: el2_rust_entry.
Inputs / outputs: none on the W02 side; on normal return from the sequencer,
  the W09 contract guarantees Completed(Stage1).
Preconditions / postconditions: called exactly once, immediately after
  Runtime.complete (W09 decision 6's expectation — this design is the
  "seam its design fixes" that decision names). After the glue, position is
  `stable` and execution is in idle.
State and ownership change: lifecycle position (via the W09 API) only.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: sequencer failures never return here (W09's
  routes are terminal); the glue adds no handling of its own.
Security/authorization checks: none.
Wiring consequence (parent README decision 6): until W09's items exist this
  call site does not compile; W02's implementation delivers everything except
  the final link of stages 10–12 and records the deferred link explicitly.
  No stub sequencer, temporary direct-to-idle wiring, or placeholder phase is
  authorized.
Logic: as in §1's tail.
Validation: W02-DV03 seam review; W09-DV01 reads the same point.
```

## 5. `controlled_idle`

```text
Name and stability: controlled_idle() -> !; internal; stable within P1.
Purpose and caller: the controlled stable state the plan names as the
  package outcome and later packages use as the normal-stage consumer state.
  Caller: the seam glue only — entry from any other point is misuse.
Inputs / outputs: none; never returns.
Preconditions / postconditions: position is `stable` (asserted by the caller
  arrangement, not re-checked — W09's tracker discipline is authoritative);
  DAIF masked (establishment stage 2 invariant). Executes the wfi loop
  forever.
State and ownership change: none.
Concurrency/allocation context: no allocation; no synchronization. WFI may
  complete while interrupts are masked (architectural behavior: pending
  masked interrupts can cause WFI exit); the loop re-enters, so any wake is
  benign and no wake-up consumer exists in P1.
Errors and failure guarantee: cannot fail; a fault taken here belongs to
  W05/W07's post-`stable` route, not to idle.
Security/authorization checks: none.
Logic:
  loop { wfi }
Validation: W02-DV03; P1-V04 execution evidence belongs to W10 (integrated
  path).
```

The stable-state marker emission point is W09's (`stable` entry, marker M3)
with token selection owned by W10; W02 owns neither the token nor the
emission and must not print anything at idle entry.
