# P2-W03 runtime storage and handoff correction

**Status:** Detailed design authorized by the user's request to finish W03's
own storage, lifetime and boot integration; implementation evidence is separate.
**Scope:** W03 foundations required before its W04 handoff.
**Version:** v0.1
**Owner/change context:** W03 completion correction, 2026-09-27.
**Supersedes:** The earlier deferral of W03 storage fit and boot adaptation to
W04/W09 in this package's design and delivery records.

## 1. Foundation audit and boundary

Parent: [W03 plan](../../plans/p2-w03-boot-memory-map-ownership.md).
Starting baseline: `main@df72bfb` (PR #70). W03 currently runs only in host
tests; its owning draft/sealed objects contain about 40 KiB of fixed arrays.
By-value constructors/returns can multiply this on P1's 64 KiB boot stack.
W02 discovery already uses substantial temporary stack storage. Object size
alone is not a call-chain stack budget, and compilation is not runtime proof.

| Required outcome | Missing foundation | Owner and acceptance |
|---|---|---|
| Map usable by W04 in the actual boot environment | Bounded storage, construction without large moves, lifetime | W03: caller-owned in-place API, target frame and execution evidence |
| Live facts and image/DTB protection | Boot wiring after W02, authoritative linker image extent | W03 adapter consuming W01's existing image extent and DTB handle |
| Metadata-before-allocation sealing | Target-usable seal API, not only owning host values | W03: borrowed draft/sealed transition; W04 still chooses metadata |
| Terminal map failures | Real rejection path exercised with hostile DTBs | W03: bounded reason/source diagnostics, no success marker after rejection |
| Reviewable completion | Correct previous overstatement and preserve prior evidence | W03 records separate original host evidence from runtime closure |

**Required:** all rows above, bounded clip observability and unchanged map
classification/accounting semantics. **Reserved:** later static/heap backing,
DTB release, SMP publication. **Out of scope:** allocator implementation,
metadata placement/layout/mapping, stack-size increase, new linker layout,
P4 objects and full W09 configuration/stress coverage. P2-ACR-01 stays open.

## 2. Storage ownership and representation

`MapStorage` owns existing bank, source and partition arrays plus lifecycle
state. Remove the redundant 672-element clip cache: each clip is the checked
intersection of one retained source and one normalized RAM bank. `clips()`
returns owned `ClipRecord` values lazily in source-major/bank-major order.
At most `8 * 84 = 672` candidate intersections exist; no allocation, truncation
or lost provenance occurs. Count warnings once while building the partition.
This removes cache storage without weakening the observable clip contract.

`UnsealedMemoryMap<S = MapStorage>` and `BootMemoryMap<S = MapStorage>` own a
backing value `S`. Queries borrow `MapStorage`; seal needs exclusive access.
Only constructors owned by W03 produce map handles. Existing owning `draft`
remains useful for host callers; the target **must use**
`draft_in(&mut MapStorage, platform, active_dtb, image)` and retain its small
borrowed handle. No `Clone`, reset, re-seal, unprotect or mutation of sealed
state is introduced. Generic backing does not grant a public constructor.

Storage lifecycle is explicit:

```text
Fresh -> Building -> Draft -> Sealing -> Sealed
               \-> Failed          \-> Failed
```

Any second attempt using non-Fresh storage returns `StorageUnavailable`.
Dropping a handle does not reset storage. Failed build/seal publishes no
allocation authority. Only constructing a different storage object starts a
new independent lifecycle. The sealed handle retains the exclusive backing
borrow internally and exposes shared queries only.

`MapStorage::new()` is const-capable. The boot adapter declares its single
storage with inline const initialization, after W02 has returned, in a
separate non-inlined, non-returning function. No full-storage assignment,
copy or by-value result is permitted on that target call chain. This is
bounded automatic storage, not a global map registry or a new allocation.
The existing P1 image range includes the entire boot stack; hence this storage
is protected as HypervisorImage, not allocator metadata. Verify containment
using the established P1 identity-address conversion before construction.

## 3. Function and lifetime contracts

- `BootMapBuilder::draft_in<'a>(&'a mut MapStorage, &PlatformInfo, Span, Span)`
  returns a draft borrowing only storage. Facts/ranges are copied into bounded
  records, not borrowed. It is boot-only, non-blocking, heap-free and safe Rust.
  Validate Fresh, enter Building, use the existing checked builder, then enter
  Draft or Failed. No partial map escapes.
- `UnsealedMemoryMap<S>::seal(self, &[PhysFrameRange])` preserves existing plan
  acceptance rules. It consumes the draft, enters Sealing, records metadata,
  independently audits the partition, then returns `BootMemoryMap<S>` or marks
  backing Failed. The full array never moves when S is a mutable borrow.
- `DtbWindow::image_range()` returns the already checked linker-owned extent
  retained by W01. No new linker symbol, raw read, physical access or unsafe
  segment is required; the W03 adapter must not invent image constants.
- `boot::p2::memory_map(...) -> !` owns the local storage and borrowed map for
  the rest of this boot. It borrows facts, DTB and its window, retains them in
  its idle loop and never remaps/releases the DTB. W04 will insert its planner
  **between draft and seal**, not attempt to reopen a sealed map.

The current binary has no allocator and therefore seals an explicit empty
metadata extent list. Emit `metadata=0 allocator=absent`; this is a complete
map of currently existing protected ranges, not a fabricated W04 plan or an
allocation-availability claim. Future W04 must replace that call site with
its actual metadata projection before introducing any allocation.

All map errors stop boot using one bounded P2 rejection line with typed reason
and source/ordinal details. Storage outside the image is an invariant stop.
No retry, fallback map or success marker follows a failure. Diagnostic storage
is bounded by the existing fatal-line builder.

## 4. Stack and validation contract

No new unsafe, synchronization, dependency, target, feature or assembly is
needed. Keep P1's 64 KiB stack and mappings unchanged. Supply both:

1. Pinned debug/release target compilation with emitted stack-frame evidence
   for the real entry, W02 caller and W03 in-place construction/seal chain;
   verify the target has no owning-map construction call or large map copy.
2. QEMU execution with single-instruction register trace covering stack use,
   plus successful terminal markers, on normal and negative W03 inputs.
   A QEMU debugger breakpoint at the W03 entry enables CPU logging only after
   earlier boot phases. The test controller changes no target memory/register;
   it removes the breakpoint before continuing. Trace all instructions and
   indirect callees after entry, not a sampling of selected functions.
   Record exact tracing granularity, minimum observed SP and image identity.
   Execution evidence is fixture-bounded and does not replace the static
   bound or prove hardware/IRQ/SMP stack usage.

If either evidence shows insufficient headroom, further reduce W03 storage or
construction temporaries within this design; do not increase P1's stack or
introduce static/raw storage as an undocumented workaround. A changed compiler,
capacity or call chain requires rechecking this budget before integration.

## 5. Ordered work and acceptance

1. Add explicit storage lifecycle and borrowed builder/seal; preserve existing
   owned host callers and all range/conflict tests. Test repeated storage use,
   failed build/seal, equivalent query/clip outputs and borrow-lifetime misuse.
2. Wire the target adapter with authoritative image/DTB inputs and lifetime
   retention. Acceptance: target emits draft then sealed summary with exact
   conservation and image/DTB/storage protection.
3. Run host/debug/release checks and compile probes. Probe small borrowed
   handle sizes and rejection of early backing destruction/mutation.
4. Run bounded QEMU normal, multibank/reservation, malformed-fact/ownership
   conflict and repeated-boot fixtures through the existing runner owner.
   Collect stack-frame/register-trace evidence and image hashes.
5. Correct the W03 records/index, then deliver through required-check PR merge.

QEMU's ordinary `-dtb` path rewrites RAM nodes. For adversarial/multibank
fixtures only, a test-loader trampoline passes an unchanged FDT at
`0x43000000`, zeros x1–x3 and branches to the normal Image at `0x40080000`.
Its page at `0x43200000` is itself a declared reservation. Both artifacts lie
inside the existing W01 trusted envelope; no access-window extension is made.
The canonical/repeated cases retain QEMU's ordinary kernel/DTB loader. This
fixture distinction is recorded in evidence, not hidden as real topology.

| Validation | Required passing condition |
|---|---|
| W03-RV01 storage | In-place equivalence, one-shot lifecycle, no authority on failure, borrowed handle lifetimes enforced |
| W03-RV02 boot | Actual W01/W02 inputs reach map, image includes map storage, sealed totals conserve RAM |
| W03-RV03 errors | Ownership overlap, non-usable fact and RAM conflict terminate without sealed marker |
| W03-RV04 stack | Actual pinned debug/release path fits unchanged stack with reported headroom; no large map movement |
| W03-RV05 handoff | W04 can consume the live draft/borrowed sealed API; remaining allocator layout/mapping work is W04-owned |

The existing W03-DV01–DV10 matrix remains mandatory. W09 still owns its full
configuration/stress matrix, but no W03-owned runtime foundation is deferred
to it. Actual results belong in the runtime verification record.
