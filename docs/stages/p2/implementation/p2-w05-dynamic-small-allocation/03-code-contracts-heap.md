# P2-W05 Code Contracts — Dynamic Small Allocation

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W05 detailed design](README.md).  
**Contract notation:** implementation-design checklist §3. Pseudocode is an
outline, not production code. Names are stage-local design freedom owned by
this design. The backing frame API is W04's
([page-alloc contracts](../p2-w04-physical-page-allocation/03-code-contracts-pagealloc.md)).

## 1. `Heap::init`

```text
Name and stability: heap::core::Heap::init(page_alloc: &mut PageAllocator,
  budget: HeapBudget) -> Result<Heap, HeapError>. Internal; called once
  per boot after W04 is ready.
Purpose and caller: acquire directory storage through the page path,
  initialize class states, publish the heap.
Inputs / outputs: W04 allocator handle; budget. Output: ready Heap.
Preconditions / postconditions: pre — W04 initialized (its own audit
  passed); post — directory live, zero allocations, budget accounting
  includes the directory pages; invariants 1–6 of
  [02 §5](02-architecture-and-state.md) hold trivially.
Concurrency/allocation context: single-core boot; uses W04 only; no
  other allocation exists yet.
Errors and failure guarantee: HeapError::OutOfMemory (directory storage
  itself) — fatal to boot (no heap); state unchanged on failure.
Security/authorization checks: directory storage is a W04
  `AllocatedFrames` value — the ownership chain starts inside this call.
Logic (pseudocode):
    dir_block = page_alloc.allocate_contiguous(DIR_PAGES)?   # typed above
    directory.init_over(dir_block)              # full block incl. held surplus
    classes[].init(); pages_used = dir_block.range.count   # budget counts
                                # actual held frames (invariant 3, [02 §5])
    return Ok(Heap{ ... })
Validation: W05-DV01 (backing chain), DV02 (first allocations).
```

## 2. `heap::layout::route`

```text
Name and stability: heap::layout::route(layout: Layout) -> Result<Route,
  HeapError>. Internal; pure.
Purpose and caller: map (size, align) to Class(class_idx) or Pages{
  page_count } per the constants of
  [02 §2](02-architecture-and-state.md).
Preconditions / postconditions: size == 0 treated as class 0 (16 B — the
  `alloc` contract's zero-size rule satisfied with a unique address);
  align ≤ MAX_SMALL_ALIGN routes to the smallest class with class_size ≥
  max(size, align); align ≤ MAX_PAGE_ALIGN and size > MAX_SMALL route to
  ceil(size / 4096) pages; anything else → UnsupportedAlignment /
  RequestTooLarge (typed).
Concurrency/allocation context: none; total function.
Errors: UnsupportedAlignment, RequestTooLarge.
Security checks: no caller value can select a route that ignores
  alignment.
Logic: comparison ladder; no arithmetic beyond checked ceil-div.
Validation: W05-DV02 (route table fixtures).
```

## 3. Checked allocation — `Heap::alloc`

```text
Name and stability: heap::core::Heap::alloc(layout: Layout) -> Result<
  AllocHandle, HeapError>. The P2 dynamic-allocation contract; callers in
  P2: later consumers post-boot; validated via host tests; wrapped by the
  adapter.
Purpose and caller: allocate memory per layout, explicitly.
Inputs / outputs: layout; output: AllocHandle { ptr_class: route info,
  raw address } — a typed handle so dealloc cannot be called with a
  fabricated pointer+layout pair.
Preconditions / postconditions: pre — heap initialized; post — bytes
  [ptr, ptr+size) uniquely owned, alignment satisfied; invariants 1–4
  hold; on Err state unchanged.
Concurrency/allocation context: &mut self; single-core; no locks.
Errors and failure guarantee: OutOfMemory{which} per the
  [01 §7](01-scope-and-foundations.md) table; stateless failure.
Security/authorization checks: every slot address derives from a
  directory-owned page; bitmap marks uniqueness; alignment enforced by
  route.
Logic (pseudocode):
    route = layout::route(layout)?
    match route:
      Class(i):
        if let Some(slab) = classes[i].partial_slabs.pop_front():
            slot = slab.alloc_slot()?                 # bitmap op
            if slab.full(): move slab to full set
            return Ok(handle(slot, class=i))
        slab_page = acquire_pages(1)?                 # budget + W04 + directory
        slab = SlabHeader::init(slab_page, class=i)
        slot = slab.alloc_slot()
        classes[i].partial_slabs.push(slab)           # not full (>=1 free)
        return Ok(handle(slot, class=i))
      Pages{n}:
        frames = acquire_pages(n)?
        return Ok(handle(frames, kind=Large))
  acquire_pages(n): budget check (on requested n; the full block counts
                    while held) -> page_alloc.allocate_contiguous(n)
                    -> directory.insert(full AllocatedFrames block, kind)
                    -> page base
Validation: W05-DV02/DV03.
```

## 4. Checked release — `Heap::dealloc`

```text
Name and stability: heap::core::Heap::dealloc(handle: AllocHandle) ->
  Result<(), HeapError>. Takes the typed handle, not a raw pointer
  (README Decision 6 rationale: ownership is exact).
Purpose and caller: release a live allocation exactly once.
Preconditions / postconditions: pre — handle is live and unmodified; post
  — slot bitmap cleared and free_count incremented (slab path) or
  directory entry removed and pages freed to W04 (large path); invariants
  hold; on Err state unchanged.
Concurrency/allocation context: &mut self; no locks.
Errors and failure guarantee: InvalidFree{reason} for double release,
  unknown handle, cross-kind mismatch; stateless failure.
Security/authorization checks: directory/slab header validation (magic,
  class match, bitmap state) before any mutation — the exactness that
  makes unknown pointers detectable.
Logic (pseudocode):
    match handle.kind:
      Class(i):
        slab = directory.slab_for(handle.page) else Err(InvalidFree{unknown})
        if slab.class != i or !slab.bitmap.test_and_clear(handle.slot):
            Err(InvalidFree{double_or_cross})
        slab.free_count += 1
        if slab was full: return slab to partial list
        if slab is entirely free and POLICY.return_empty:
            classes[i].remove(slab); release_pages(slab.page, 1)
      Large:
        entry = directory.remove_large(handle.frames) else Err(...)
        page_alloc.free_contiguous(entry.block)     # full block incl. held surplus
    return Ok(())
Validation: W05-DV05.
```

## 5. `HeapStats` and audit

```text
Name and stability: stats() -> HeapStats { pages_used, budget_max,
  per_class: [ClassStats { slabs, free_slots, used_slots }; NUM_CLASSES],
  large_allocations }, and
  audit() -> Result<(), HeapError> (recomputes invariants 1–4, 6 from
  directory + headers; used by tests and diagnostics).
Purpose and caller: live-derived observability for W06; invariant checks
  for stress tests.
Preconditions / postconditions: stats values satisfy conservation vs
  directory; audit walks everything once.
Concurrency/allocation context: &mut self for audit; &self for stats (no
  concurrent mutation exists in P2).
Errors: audit reports the violated invariant class as
  HeapError::InvariantBroken{which} — boot policy: fatal.
Validation: W05-DV06/DV07.
```

## 6. `GlobalAlloc` adapter (conditional module)

```text
Name and stability: heap::global::HeapAdapter, `unsafe impl GlobalAlloc`
  over a registered &Heap; compiles only when the target supplies `alloc`
  ([01 §4](01-scope-and-foundations.md)); P2-local; the alloc-error
  handler for the boot phase lives beside it.
Purpose and caller: make `alloc`-based dynamic allocation available to
  later P2 consumers and P3/P4 designs.
Inputs / outputs: Layout in; ptr or null out (alloc contract); dealloc
  per contract.
Preconditions / postconditions: the adapter adds no logic — it routes to
  core and maps outcomes: Ok -> ptr; core Err(OutOfMemory|...) -> null
  plus the registered fatal allocation-error path (README Decision 5);
  dealloc -> core dealloc with InvalidFree -> fatal invariant stop
  (README Decision 6).
Concurrency/allocation context: `GlobalAlloc` is &self; P2's single-core
  ownership makes this sound; P3 revisits (its design owns the locked
  wrapper).
Errors and failure guarantee: per [01 §7](01-scope-and-foundations.md)
  table — the adapter never invents a recovery.
Security/authorization checks: none beyond core (adapter is a thin map);
  `SAFETY` argument: single-threaded boot ownership; Layout values come
  from the compiler's call sites.
Logic: two thin functions + handler registration; no caching, no retries.
Validation: W05-DV04 — only when `alloc` is available on the P0 target;
  otherwise recorded as blocked-by-upstream with the inner API validated
  regardless (W05-DV02/03/05/06/07).
```

## 7. Explicitly unauthorized interfaces

No `realloc`/grow-in-place (Reserved); no zeroing-on-alloc; no
alignment > `MAX_PAGE_ALIGN`; no per-class caches or CPU-local pools (P3);
no direct page allocation exposed to consumers (they use the heap or W04
explicitly); no telemetry hooks beyond `HeapStats`; no Drop side effects on
`AllocHandle` (release is explicit).
