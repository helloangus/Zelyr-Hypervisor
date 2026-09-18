# P2-W08 Scenario Matrices — Allocator, Stress, Determinism

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W08 detailed design](README.md).  
Column semantics as in
[02 §1](02-matrices-input-and-map.md). Page-allocator scenarios run over
host-buffer-backed metadata and synthetic sealed maps (W04's host mode).

## 1. W08-S3xx — allocator lifecycle and protected-page safety (P2-J03)

| ID | Input / precondition | Expected observable | Pass condition | Technique | Proves / does not prove |
|---|---|---|---|---|---|
| W08-S301 | Exhaust a single-region fixture with order allocations | `OutOfFrames{order, region_stats}` with stats snapshot | Err; `stats()` unchanged vs pre-call | Drain loop | Explicit OOM; stateless failure |
| W08-S302 | `allocate(order > MAX_ORDER)`; `allocate_contiguous(0)` | `OrderTooLarge` / typed error | Err, no state change | Boundary calls | Discipline enforcement |
| W08-S303 | Free each allocation in FIFO and LIFO orders; re-allocate after full release | Ok; freed frames reusable; final `stats()` equals initial baseline | Conservation + equality vs snapshot | Cycle loops | Reuse-after-release (P2-E02) |
| W08-S304 | Free variants: unmanaged range, double free, bad order/alignment, partial overlap of a live block, metadata-range free | `UnmanagedFree` / `DoubleFree` / `BadFreeRange` per class | Each Err with its class; state unchanged | Error-fixture matrix | Debug detection (P2-E06); documented order-mismatch limit applies (W04 residual) |
| W08-S305 | Exact-count allocations at span boundaries (count = span, span±1, 2^k ± 1) | `AllocatedFrames` with correct `usable`; surplus held then freed exactly | Post-free state equals pre-alloc baseline | Boundary counts | Surplus-holding contract (W04 Decision 2) |
| W08-S306 | Multi-region fixtures (1/2/8 spans) | No cross-region block or coalesce; per-region isolation | Region membership asserted per allocation | Span matrices | Multi-region support (P2-E04) |
| W08-S307 | Init audits: tampered seal (plan ≠ sealed ranges), domain mismatch, conservation break | `AllocatorInitError` variants | Each class fired by injected inconsistency; no allocator published | Init-fault injection | Structural hard-gate audit (W04 I-audit) |
| W08-S308 | **Protected-page soak (hard gate, host strength)**: seeded randomized sequence (≥ 10⁴ ops) of valid allocate/free/contiguous plus interleaved invalid frees over a map rich in protected classes; audit after every op | Zero protected frames in any returned range, ever; invariants hold after every op | Audit-clean at every step; seed and op count recorded | Seeded soak + per-op audit | The P2 gate at allocator logic level; not QEMU integration (W09) or hardware |
| W08-S309 | Heap release semantics: double dealloc, unknown handle, cross-kind mismatch | `InvalidFree{reason}` per class | Err, state unchanged | Error-fixture matrix | W05 exactness (P2-F03) |
| W08-S310 | Heap budget: force exhaustion; release; retry | Typed `OutOfMemory`; after release, retry succeeds; budget never exceeded | `pages_used <= budget` at every checkpoint | Budget stress | Containment + recovery (P2-F02) |

## 2. W08-S4xx — allocation stress (P2-J04)

All stress scenarios: fixed seeds, op counts and seeds recorded; checkpoint
invariant audit every K ops (K fixed per scenario); no timing measurement.

| ID | Input / precondition | Expected observable | Pass condition | Technique | Proves / does not prove |
|---|---|---|---|---|---|
| W08-S401 | Page-allocator stress: seeded random orders/sizes with frees in random order, N ≥ 10⁵ ops | Conservation at every checkpoint; final free-frame count equals baseline after full release | Audit-clean; baseline equality | Seeded driver | Sustained correctness; not performance or fragmentation quality |
| W08-S402 | Heap stress: seeded random class alloc/dealloc across all 8 classes, slab churn with return-empty policy toggled per configuration | Slab/class invariants (W05 I1–I6) at checkpoints; `audit()` clean | Audit-clean; budget respected | Seeded driver | Class machinery under churn; not latency |
| W08-S403 | Combined stress: heap-driven page allocation (heap acquires/releases pages while independent direct allocations run) | Both accounting systems consistent; no cross-system drift | Joint audit; C3/C4-style checks hold at checkpoints | Two-driver interleaving (seeded) | Ownership-chain integrity (W05 over W04); not SMP behavior (single-owner only) |
| W08-S404 | Exhaustion-and-recovery cycles: repeatedly drain then fully release (pages and heap), C cycles | Every cycle: explicit OOM then full recovery; no monotonic drift of free counts | Baseline equality at each cycle end | Cyclic drain | Lifecycle repeatability (P2-V07's "recovery" wording) |
| W08-S405 | Determinism of stress: re-run S401–S403 with identical seeds | Identical operation traces and final states | Bit-identical replay | Double-run diff | Seeded reproducibility of the suite itself |

## 3. W08-S5xx — deterministic repeated discovery (P2-J05)

| ID | Input / precondition | Expected observable | Pass condition | Technique | Proves / does not prove |
|---|---|---|---|---|---|
| W08-S501 | Identical fixture blob through W02 `normalize`, R repeated runs (R ≥ 2, recorded) | Identical `PlatformInfo` bits each run | Byte-level comparison passes | Double-run diff | Discovery determinism (W02 §7) |
| W08-S502 | Identical fact inputs through W03 draft+seal | Identical map entries, summary, anomalies | Byte-level comparison | Double-run diff | Map determinism |
| W08-S503 | Identical full chain (blob → intake → facts → map → allocators → W05 heap) then W06 `compose` + `render` | Identical report and identical rendered bytes | Byte-level comparison | Chain double-run | End-to-end active-state render determinism |
| W08-S504 | Chain re-run in a fresh process with identical inputs | Identical outputs | Cross-process comparison | Two invocations | No process/environment leakage (addresses, ordering) |
| W08-S505 | W06 divergence injection: build divergent section sets per target (platform RAM total ≠ map total; map equation broken; allocator domain/conservation broken; heap pages unaccounted) and compose | `ConsistencyBroken{C1|C2|C3|C4}` per tamper target; nothing published | Correct check ID per injected case; first-failure order holds | Fault injection | Consistency detection (with W06-DV05; included here so the regression suite owns the running version) |
| W08-S506 | QEMU-supplied `virt` DTB (from W07's fixture, if acquired) through S501/S503 | Same determinism on a real-platform-shaped blob | Comparison passes | Fixture reuse | Determinism generalizes beyond synthetic blobs; not QEMU runtime behavior (W09) |
