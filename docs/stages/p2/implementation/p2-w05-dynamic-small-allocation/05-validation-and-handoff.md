# P2-W05 Validation, Error Model, and Handoff Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W05 detailed design](README.md).

## 1. Scope of validation for this package

W05's own evidence is host-side: the entire heap is hardware-independent
(buffer-backed pages) by design ([01 §8](01-scope-and-foundations.md)).
QEMU evidence (heap live on the reference platform, stats across repeated
boots) is W09; wider stress/negative harnesses are W08. All validation
here is correctness/invariant evidence; nothing in this package supports a
performance claim (plan work sequence 5), and none may be recorded as one.

## 2. Error, security, and observability model

- **Error model.** Typed at the checked layer
  ([01 §7](01-scope-and-foundations.md)): `OutOfMemory{which, stats}`,
  `InvalidFree{reason}`, `UnsupportedAlignment`, `RequestTooLarge`,
  `BudgetExhausted`, `InvariantBroken{which}` — all stateless on failure.
  Fatal at the adapter layer per the two policies (allocation-failure →
  resource-exhaustion stop; invalid-free → invariant-violation stop),
  classified per P0-W14. Guests do not exist in P2, so no failure here is
  guest-caused; the guest-facing boundary is a later stage's design.
- **Security model.** Containment is inherited transitively (heap → W04 →
  W03 sealed map), asserted by construction review and the W05-DV01
  chain check; ownership is exact (typed handles, directory, bitmap);
  budget bounds blast radius; unknown pointers are detectable, not
  guessed at. Residual limits recorded for W10: directory capacity bounds
  are containment (typed exhaustion), not unbounded growth; no
  hardening against a *hypervisor bug* that forges handles beyond the
  checks — that class is a coding-guideline audit matter, not a runtime
  property.
- **Observability.** `HeapStats` is live-derived (per-class used/free,
  pages vs budget, large allocations); boot emits one "heap ready" marker
  with totals; errors render with `which`/`reason` detail. W06 renders
  stats; W09 compares boot-to-boot stats traces (determinism invariant 6).

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W05-DV01 → P2-V07 | Backing chain | Host joint test with W04 fixtures + construction review | Every heap page traces to an `AllocatedFrames` value; W03 seal precedes | No heap byte exists outside a W04 allocation | Transitive protection inheritance; not W04's internals |
| W05-DV02 → P2-V07 (P2-F01) | Small-object needs | Host unit + property tests | Alloc/free across all classes, header-boundary slots, zero size | Unique aligned handles; invariants hold per op | Class machinery correctness; not later-stage object designs |
| W05-DV03 → P2-V07 (P2-F02) | Explicit failure | Host tests | Exhaust class, slabs, directory, budget, backing (small pools) | Each `OutOfMemory{which}` fires with correct `which`; stats unchanged | Typed exhaustion; not performance under pressure |
| W05-DV04 → P2-V07 | `GlobalAlloc` adapter | Host integration (only if `alloc` available) | `Box`/`Vec`-shaped corpus through the adapter; failure-path hooks | Null-on-failure + fatal policy observed; dealloc breach halts | `alloc` surface works; skipped ⇒ blocked-by-upstream entry |
| W05-DV05 → P2-V07 (P2-F03) | Release | Host tests | Correct release; double free; unknown handle; cross-kind | `Ok` exactly once; each misuse typed `InvalidFree`; state unchanged | Release exactness; not Drop semantics (none designed) |
| W05-DV06 → P2-V07 (P2-F04) | Stress invariants | Host property/soak test | Long randomized interleavings incl. empty-slab returns and exhaustion probes | Invariants 1–6 of [02 §5](02-architecture-and-state.md) hold after every operation; leak-freedom at cycle ends | Invariant preservation under stress; no performance claim |
| W05-DV07 → P2-V07/P2-V10 | Determinism | Host property test | Identical operation sequences from identical init | Identical stats traces | Repeatability; not hardware-idempotence |
| W05-DV08 → W05 closure | Static-array boundary review | Design review | Check [01 §6](01-scope-and-foundations.md) position against the implemented structure | Pre-heap arrays remain boot-phase; heap is the post-boot model; directory bounds documented as containment | Boundary honesty; not future migration designs |
| W05-DV09 → W05 closure | Consumer walkthrough | Design review | Read as W06 (stats), W08 (fixtures), W09 (traces), P3/P4-via-W10 (boundaries) | Each consumer can proceed without new W05 work | Handoff readiness; not consumer implementations |

Evidence statuses are passed / failed / blocked / not run with command,
input, environment, timestamp. A deferred adapter is *blocked*, with the
inner-API evidence standing alone. Host validation does not prove QEMU
integration or real-memory behavior (W09), SMP safety (P3), or any P4
object design.

## 4. Handoff checklist

Before handing W05 to review, provide:

- changed-module list; confirmation of zero locks/atomics, zero board
  names, zero W04 API changes, zero performance instrumentation, and
  `unsafe` confined to the adapter trait impls and page-view fabrication
  with `SAFETY` arguments;
- W05-DV01–DV09 evidence statuses, including the adapter's
  blocked-by-upstream entry if A2 is unresolved, and not-run entries for
  QEMU (W09) and stress harness (W08);
- confirmed consumer readiness: W06 (`HeapStats`), W08 (typed errors and
  invariant list as fixtures), W09 (stats traces), W10 (dynamic-allocation
  contract, failure policy, containment bounds for P3/P4);
- open items recorded, not resolved: P0 target/`alloc` decision (A2),
  physical placement (P0-W03), Reserved items (realloc, zeroing, class
  tuning, CPU-local pools), and the static-array boundary review
  statement;
- explicit statement that this package designs no VM/vCPU, scheduler, or
  capability objects and adds no public API beyond
  [03](03-code-contracts-heap.md).
