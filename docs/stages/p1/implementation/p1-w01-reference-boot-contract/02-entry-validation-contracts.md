# P1-W01 Entry Validation Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W01 detailed design](README.md).

All names below are boot-scope items. The pre-transfer tier (§1–§3) is
assembly executed before any Rust code; its physical home is the boot entry
module that [P1-W02](../p1-w02-minimal-rust-el2-runtime/README.md) delivers,
implementing these contracts verbatim. Pseudocode is an outline of the
algorithm, not runnable production code. No allocation exists anywhere in
this design (no stack, no heap, no Rust runtime in the pre-transfer tier).

## 1. Tier model

```text
firmware handoff
  -> Tier A (assembly, pre-transfer): T1 exception-level check, T2 DTB-pointer
     check, rejection reporter, bounded stop          [this design]
  -> transfer: established discipline (stack, SPSel, DAIF, BSS) and Rust entry
     execution                                        [P1-W02]
  -> Tier B (Rust, post-transfer): no checks; only retention of the boot
     parameters per [01-boot-contract.md](01-boot-contract.md) §4 and the
     W09 entry-event records on the transfer guarantee
```

There is deliberately no post-transfer "Tier B validation". Conditions that
cannot be checked before the runtime exists are assumptions with evidence
obligations (contract §3), not checks; adding Rust-side validation would
relocate the boundary after transfer and violate R1 of the contract.

## 2. Tier A checks — pre-transfer validation block

```text
Name and stability: the pre-transfer validation block of the boot entry;
  assembly, internal, stable within P1. Contract owner: this design;
  physical home: the boot entry module (W02).
Purpose and caller: classify the delivered environment against the T1/T2
  classes before anything else executes; branch to the rejection reporter on
  failure, or fall through to runtime establishment. Caller: the image entry
  point, as its first instructions.
Inputs / outputs: reads CurrentEL and x0 only. No output other than the
  rejection branch.
Preconditions / postconditions: executes with the firmware-delivered machine
  state of contract §2; on fall-through, E1 and E7 of the entry-state table
  hold and execution continues at the runtime-establishment sequence. On
  rejection, R1–R5 hold and nothing downstream executes.
State and ownership change: none (reads only; no memory or register writes
  before the fall-through point).
Concurrency/allocation context: boot CPU only, no allocation, no stack, no
  assumption about DAIF beyond the canonical-path property (the block itself
  performs no masking — masking is W02 establishment work).
Errors and failure guarantee: a disallowed environment diverges to the
  rejection reporter and never returns to the fall-through path.
Security/authorization checks: this is the hypervisor's first trust decision:
  it refuses to execute privileged runtime code at an unexpected exception
  level.
Logic (per check, in order; both checks before any establishment work):
  T1: mrs x_tmp, CurrentEL; if (x_tmp >> 2) != 0b10 -> reject(reason=EL)
  T2: cbz x0 -> reject(reason=DTB)
Validation: W01-DV03 boundary review; W11 NC1 executes the EL class.
```

Check order is normative: T1 before T2, because the exception level is the
trust-relevant class and the reporter itself is only meaningful at EL2
(polling-store behavior at EL1 is an unowned environment).

## 3. Rejection reporter

```text
Name and stability: the rejection reporter; assembly, internal, stable within
  P1. Contract owner: this design; physical home: the boot entry module (W02).
Purpose and caller: emit the fixed rejection token line, then execute the
  bounded stop. Caller: the Tier A checks only; unreachable from anywhere
  else in the image.
Inputs / outputs: reason token ('EL' | 'DTB') from the caller; output is one
  ASCII line through the raw reference-UART polling write.
Preconditions / postconditions: preconditions are the caller's (running at
  the firmware-delivered state, pre-establishment). Postconditions: the line
  was emitted (on the reference platform, guaranteed; elsewhere
  best-effort-by-design per R4) and the boot CPU never executes another
  instruction of this image.
State and ownership change: the raw UART data register only; no other device,
  register, or memory access.
Concurrency/allocation context: no stack, no allocation, no loops except the
  transmit-poll and the terminal stop; no timer, no interrupt.
Errors and failure guarantee: a transmitter that never clears its busy
  condition would spin in the transmit poll — the bounded stop of R3/R4 is
  then the timeout-visible outcome; the reporter never continues normal
  execution in any case.
Security/authorization checks: none beyond the tier's own trust decision;
  the reporter emits fixed literals and never reflects register or memory
  content.
Output format (fixed, single line, ASCII):
  "ZELYR P1 BOOT REJECT reason=<token>\r\n"
Logic:
  for byte in literal_prefix: poll-until-writable; store byte to UART data
  register
  emit token byte(s); emit CRLF
  bounded stop: branch-to-self
Validation: W01-DV03/DV05; W11 NC1 observes the line; W10's matcher sees the
  forbidden-marker absence property of normal boots (a normal boot must
  contain zero rejection lines).
```

The reporter is not a console and is not shared with W06: it exists only on
the rejection path, uses no initialization, and defines no abstraction. The
raw UART base constant follows the single-source rule of §6.

## 4. Transfer-guarantee mechanics

```text
Name and stability: the transfer guarantee; a property, not a function.
  Owner: this design (guarantee); [P1-W02](../p1-w02-minimal-rust-el2-runtime/README.md)
  (recording on the guarantee's authority); W09 (record semantics).
Purpose: let downstream packages rely on the entry-state table (contract §3)
  without re-checking, and let W02-owned code record the W09 `entry` events.
Inputs / outputs: none. The observable is the W09 tracker state:
  Entry.enter and Entry.complete recorded at the start of `el2_rust_entry`.
Preconditions / postconditions: the guarantee is exactly "execution reached
  the Rust entry" implies T1/T2 passed. The records are written after the
  runtime can execute code but before any other lifecycle event; they are
  W02's first tracker operations, per W09 decision 6's delegation of the
  `entry` and `runtime` records to W02-owned code.
State and ownership change: W09 tracker position only; W01 owns no state.
Concurrency/allocation context: boot context; the tracker API is W09's
  single-writer compare-exchange discipline.
Errors and failure guarantee: if the tracker rejects the record (sequence
  misuse), the route is W09's misuse routing — not a W01 concern.
Security/authorization checks: none; recording is not authorization.
Tracker mechanics (W09 contract, restated for the reader): `Runtime.enter`'s
  required predecessor is `Completed(Entry)`, left by these records; the
  sequencer's first advance (`Capabilities.enter`) requires
  `Completed(Runtime)`, left by the runtime record.
Logic: none here — the authoritative sequence lives in
  [P1-W02's runtime contracts](../p1-w02-minimal-rust-el2-runtime/03-code-contracts-rust-runtime.md)
  §1, which this design consumes.
Validation: W01-DV05 review that the delegation wording matches W09 §8;
  W09-DV01 lifecycle review reads the same point from its side.
```

## 5. Post-transfer routing refinement (recorded W09 matrix extension)

The W09 failure-routing matrix has an `entry` row (pre-transfer rejection
route) and a `runtime` row (panic route). This design fixes the behavior of
the gap between them, and raises the refinement to
[P1-W09](../p1-w09-initialization-sequencing/README.md) for review per its
conflict rule rather than absorbing it silently:

```text
Refinement row — `entry` (post-transfer window)
  Window: after the Rust entry executes and before Entry.complete is
    recorded (W02-owned records happen early in the Rust entry, so the
    window is by construction small; see W02 §3).
  Failing condition: an invariant violation during the W02-owned entry
    recording itself (W09 SequenceError or similar) — no W01-owned check
    exists in this window by design (§1).
  Route: the panic route (established: the W02 runtime's panic handler is
    linked before any Rust instruction runs).
  Phase attribution: `entry` (the tracker is at Entry.enter or PreBoot;
    the W07 phase field / tracker read yields the accurate position per
    W09 rule H6).
  W09 H4 check: satisfied — the route (panic) is established before the
    window opens.
```

If W09's accepted design cannot accept this row as an amendment, the conflict
is a recorded coordination issue between W01 and W09 owners; no local
rewording of either matrix is authorized.

## 6. Shared boot-entry constant (single-source rule)

The raw reference-UART base address is defined exactly once in the boot entry
module and is consumed by both the rejection reporter (assembly) and — via a
Rust-visible constant in the same module — W02's early diagnostic writer.
Rules:

- the constant is documented in the implementation record as a
  reference-platform fact (contract §8 layering statement);
- W01 and W02 implementations must not duplicate the value; a second literal
  in any module is a review failure (W01-DV03 includes the single-source
  search);
- W06's console defines its own reference-console assumption per its plan and
  must not consume this constant; when P2 discovery replaces fixed addresses,
  both consumers migrate through their own designs.

## 7. Rejection-token vocabulary (evidence interface)

The tokens `EL` and `DTB`, the literal prefix, and the line format of §3 are
the complete machine-facing vocabulary of the boundary. They are P1-internal
boot diagnostics, not an ABI: W10 and W11 match them as evidence classes;
external compatibility promises are out of scope. Token changes invalidate
prior evidence and follow the W10 precedent (recorded before the first
verdict-bearing run, never adjusted afterwards).
