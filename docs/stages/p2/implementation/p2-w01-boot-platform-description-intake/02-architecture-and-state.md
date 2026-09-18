# P2-W01 Architecture, State, and Lifecycle Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W01 detailed design](README.md).

## 1. Logical modules

W01 is one mechanism decomposed into five logical modules. Names are
stage-local design freedom owned by this design (README, Decision 2 rationale);
their physical placement is inside the platform/discovery layer of the P0-W03
workspace when it exists (working name `hv-platform` per ADR §13; the crate
naming question is pending under ADR-054 and does not belong to this design).

| Module (logical) | Responsibility | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|
| `fdt_access` | The single `unsafe` boundary: fabricate checked read-only byte views of physical memory through the A2 window | A2 window function, validated span | `&[u8]` views | No validation, no interpretation; the A2 contract itself (P1) |
| `fdt_header` | Header field decoding and validation | Byte view + placement facts | `ValidatedHeader` (all spans pre-checked) | Structure walking; reservation entries |
| `fdt_structure` | Token-stream validation and the bounds-guaranteed cursor over nodes/properties | `ValidatedHeader` + bytes | `ValidatedStructure`, `StructureCursor`, node/property accessors | Semantic interpretation of any property |
| `fdt_reservation` | Reservation-list validation | Bytes + validated blob span | `ValidatedReservations` (entry list + anomaly counter) | What reservations mean for memory (W03) |
| `intake` | Orchestration: placement checks, module sequencing, diagnostic funnel, handle publication | A1 location/length, A3 image range, A2 window, config limits | `ValidatedBootDtb` or one fatal `IntakeDiagnostic` | Any platform fact; any range claim |

Layering note: none of these modules may reference an architecture register, a
board name, or a QEMU constant (ADR-043/052). The only hardware-facing
dependency is the injected A2 window function, so the entire stack above
`fdt_access` is host-testable with synthesized byte fixtures.

## 2. Core objects and ownership

There is exactly one mutable state transition in W01 — the move from
"unvalidated boot input" to "published handle" — and it is owned by the
`intake` orchestrator. Everything else is immutable after construction.

| Object | Owner | Mutable state | Lifetime |
|---|---|---|---|
| Raw blob bytes (physical memory) | Firmware; after W03, the protected map | None (read-only) | Boot phase |
| `ValidatedBootDtb` | The boot sequence (single publication site) | None after publication | Until hypervisor restart |
| `ValidatedHeader` / `ValidatedStructure` / `ValidatedReservations` | Borrowed by / composed into the handle | None after validation | Borrow of handle |
| `StructureCursor` | Borrower (W02) | Cursor position only, `&mut` to its own instance | Borrow of handle |
| Config limits (`max_dtb_size`, caps) | Boot configuration, read-only | None | Static |

No registry, no global, no singleton: the handle is a value handed to W02 by
the boot sequence. A convenience global holding the handle is prohibited
(Coding Guidelines) — ownership stays with the boot sequence, which is the
single authority that decides when intake happened.

## 3. Lifecycle and state model

```text
BootInput (A1 pair + A3 range + A2 window)
   |
   v  intake::validate()
[Placement] --fail--> IntakeDiagnostic (fatal, one class) --> boot stop
   |
   v
[Header]     --fail--> IntakeDiagnostic --> boot stop
   |
   v
[Structure]  --fail--> IntakeDiagnostic --> boot stop
   |
   v
[Reservation]--fail--> IntakeDiagnostic --> boot stop
   |
   v
ValidatedBootDtb published  (immutable; blob now effectively frozen:
                             W03 will protect the range; no writer exists)
```

Properties that each contract in [03](03-code-contracts-intake.md) must
preserve:

- **All-or-nothing publication.** The handle exists only after all four
  stages pass; there is no "provisionally valid" state (§1 outcome).
- **Fail-fast ordering with locality.** Stages run in dependency order
  (placement → header → structure → reservation) so the reported diagnostic
  is the earliest independent failure; later-stage diagnostics must not
  trigger on inputs an earlier stage would have rejected.
- **No undo, no retry.** A failed intake is a boot stop (README Decision 4);
  there is no re-validation path in P2. W07's offline checker re-runs the
  same validators as fresh instances on host; it does not share state with a
  boot instance.
- **Frozen-after-validation.** Because the handle borrows the blob bytes and
  no writer exists in P2, re-validation on read is unnecessary; if a later
  stage ever gains a DTB-release or copy mechanism (Reserved), the lifecycle
  must be redesigned, not extended ad hoc.

## 4. Structural caps (DoS bounds)

Untrusted input must not convert CPU time into an attack surface. The
following stage-local caps are fixed by this design (rationale: orders of
magnitude above every reference/fixture DTB observed for QEMU `virt` and the
RK3566 fixture, while bounding worst-case validation work; each is a
configuration constant, reviewable, not hidden):

| Cap | Value | Rationale |
|---|---|---|
| `max_dtb_size` | 8 MiB | Bounds blob reads and all span arithmetic; fixture DTBs are < 1 MiB |
| `max_depth` | 32 | DT sources are machine-generated; real trees are < 10 deep |
| `max_nodes` | 4096 | Reference trees have hundreds; bounds structure walk |
| `max_properties_total` | 16384 | Bounds name resolution work |
| `max_reservations` | 1024 | QEMU emits 0–2; real firmware similar |
| `max_name_len` | 256 | Per node/property name; spec-conformant names are far shorter |

A cap violation is a distinct diagnostic (`DtbStructureInvalid` with cap
detail, or `DtbReservationInvalid` for the reservation list), not a silent
truncation. Caps are distinct from the P2-W02 **semantic** capacity limits
(platform fact list sizes), which are owned by W02's design — the two must
not be conflated.

## 5. Concurrency, allocation, and interrupt context

- **Single-core boot phase.** W01 runs on the boot CPU after P1's stable
  runtime, before any SMP mechanism exists (P3 scope). All state is owned by
  the boot sequence; there are no locks, no atomics, and no IRQ interaction.
  This is a stated stage boundary, not an accident to fix later: P3
  (p3-w06-concurrency-synchronization) owns any future synchronization design,
  and because the handle is immutable after publication, concurrent *readers*
  need no additional mechanism even in later stages.
- **No allocation.** Intake runs before the W04/W05 allocators exist. All
  validation is allocation-free over borrowed bytes; the validated
  reservation entry list uses a fixed-capacity boot-time array inside the
  handle (capacity `max_reservations`), not the heap. This is a temporary
  pre-heap model by design; the permanent dynamic model is W05's heap, and
  this boundary is reviewed there (W05, plan work-sequence item 4).
- **No blocking, no long work.** Validation is O(blob) with the caps above;
  worst case is linear in 8 MiB, acceptable at boot.

## 6. Failure model

- Every failure is one `IntakeDiagnostic` value with a stable class
  ([01 §3](01-intake-boundary.md)) and structured detail; the funnel is the
  orchestrator's single return point, so a caller cannot observe a
  half-validated state.
- Failure guarantee on error: no handle exists, no byte of the blob has been
  written (only reads occurred), no global state changed, and the diagnostic
  identifies the stage. Re-running intake is not defined in P2 (boot stops);
  host tests re-run validators freely on fresh inputs.
- The `unsafe` boundary failure mode is different in kind: if the A2 window
  contract is violated by P1 (e.g., coverage lies), no in-module check can
  save the read. That is why the window is an assumed contract with a
  failure boundary ([01 §2](01-intake-boundary.md)) and why `fdt_access`
  must contain the *only* address arithmetic that trusts it, small enough to
  audit in one sitting (P0-W10).

## 7. Security model summary

Untrusted source: firmware-supplied blob (and, transitively, the A1 location
pair). Trust anchors: the A2 window contract, the A3 image range, and the
caps. Invariants enforced by construction: bounds before every read; checked
arithmetic everywhere; one `unsafe` boundary; no allocation; no
interpretation below the structural layer. Residual risk accepted for P2:
a *correct-looking* DTB with hostile semantics (e.g., a `reg` pointing at the
hypervisor) is invisible to W01 by design — semantic containment is W02/W03's
contract, and the W10 handoff must carry that division of responsibility
forward.
