# P2-W06 Code Contracts — Inspection

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W06 detailed design](README.md).  
**Contract notation:** implementation-design checklist §3. Pseudocode is an
outline, not production code. Names are stage-local design freedom owned by
this design. Source contracts consumed here: W02
([facts](../p2-w02-platform-discovery-normalization/04-code-contracts-facts.md)),
W03
([bootmap](../p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md)),
W04
([pagealloc](../p2-w04-physical-page-allocation/03-code-contracts-pagealloc.md)),
W05
([heap](../p2-w05-dynamic-small-allocation/03-code-contracts-heap.md)).

## 1. `InspectionError` and diagnostic shape

```text
Name and stability: InspectionError (enum: ConsistencyBroken { check:
  CheckId, detail: CheckDetail }, CapacityInherited { which }), internal;
  CheckId ::= C1 | C2 | C3 | C4.
Purpose and caller: the only failure type of compose; asserted by W08 and
  rendered by boot diagnostics.
Preconditions / postconditions: carries source identities and counts, never
  content ([01 §3](01-scope-and-foundations.md)).
Errors: n/a (this is the error type).
Validation: W06-DV05.
```

## 2. `compose`

```text
Name and stability: inspect::compose(platform: &PlatformInfo,
  map: &BootMemoryMap, pages: &PageAllocator, heap: &Heap,
  bootctx: &BootContextRecord) -> Result<InspectionReport,
  InspectionError>. Internal; called at boot diagnostic points and by host
  tests; W09 observes its render output.
Purpose and caller: the single W06 entry point; build sections from the
  live sources, enforce C1–C4, publish all-or-nothing.
Inputs / outputs: shared references to the four authoritative sources plus
  the A5 boot-context record. Output: one immutable report or one
  diagnostic.
Preconditions / postconditions: pre — W04/W05 initialized and stats()
  callable (&self), map sealed; post — on Ok, every section field traces to
  one source field and C1–C4 held at composition time; on Err, no report
  exists (all-or-nothing, [02 §3](02-architecture-and-state.md)).
State and ownership: owns a fixed-capacity report under construction on the
  caller's stack; sources only borrowed; no allocation.
Concurrency/allocation context: single-core boot phase; no locks; no
  allocation ([01 §4](01-scope-and-foundations.md)).
Errors and failure guarantee: ConsistencyBroken (first failing check in
  fixed order C1→C4) or CapacityInherited (a source reported exhaustion at
  construction); stateless failure — nothing was published, sources
  untouched.
Security/authorization checks: read-only access only; no content rendering
  inputs are copied into the report; no platform names can exist in any
  rendered field (mechanically reviewable, W06-DV07).
Logic (pseudocode):
    boot = sections::boot_context(bootctx)
    plat = sections::platform(platform)?          # CapacityInherited only
    mm   = sections::memory_map(map)?
    pg   = sections::page_allocator(pages.stats())
    hp   = sections::heap(heap.stats())
    for check in [c1, c2, c3, c4]:                # fixed order
        check(plat, mm, pg, hp, sources)?         # ? -> ConsistencyBroken
    return Ok(InspectionReport { boot, plat, mm, pg, hp })
Validation: W06-DV01–DV05, DV08.
```

## 3. Section builders (`sections::*`)

```text
Name and stability: sections::boot_context / platform / memory_map /
  page_allocator / heap, internal pure functions; one per source.
Purpose and caller: map source records to section values per the rules of
  [01 §3](01-scope-and-foundations.md); called only by compose.
Inputs / outputs: source reference (or stats value) -> section value.
Preconditions / postconditions: fact states render verbatim — Usable with
  a summary payload, Unsupported{reason}, Unusable{diagnostic class},
  Absent, NotDiscovered all have distinct render marks; no state is
  collapsed or upgraded (README: no interpretation upgrades).
Concurrency/allocation context: none; pure.
Errors: CapacityInherited only if a source-side count already exceeds the
  section's fixed capacity (unreachable given capacity-matched constants;
  kept as a guarded diagnostic, not a panic).
Security checks: string-typed fact payloads (stdout_path, bootargs) render
  as presence + length + state, never content; ranges render as
  (first, count) typed pairs.
Logic (pseudocode, platform builder excerpt):
    cpu = { usable: count(facts.cpus where Usable),
            disabled: count(where status disabled), boot_matched: facts.boot_cpu_ok }
    gic = match facts.gic { Usable(g)  -> Mark::Usable{regions: 1 + g.redistributors.len()},
                            Unsupported{r} -> Mark::Unsupported{reason: r},
                            Unusable{d} -> Mark::Unusable{diag: d.class},
                            Absent -> Mark::Absent, NotDiscovered -> Mark::NotDiscovered }
    ... same shape for timer, psci, console, bootargs
    ram_total_frames = sum(bank.len) in frames      # C1 input
    counters = facts.counters verbatim
Validation: W06-DV01, DV08.
```

## 4. Consistency checks (`consistency::c1..c4`)

```text
Name and stability: consistency::{c1, c2, c3, c4}, internal pure
  functions; registry fixed in [02 §4](02-architecture-and-state.md).
Purpose and caller: detect divergence between rendered sections and their
  sources; called only by compose in fixed order.
Inputs / outputs: section values (+ source references where the check needs
  the authoritative field, e.g. c3 reads map.allocatable_spans() directly);
  output Result<(), (CheckId, CheckDetail)>.
Preconditions / postconditions: cheap — sums over bounded lists; checked
  arithmetic throughout; a check never mutates and never "fixes" a value.
Errors: the check's own failure is its output; internal overflow maps to
  ConsistencyBroken{check, detail: overflow}.
Security checks: checks compare derived-vs-derived and derived-vs-source
  only; they never read frame contents or DT bytes.
Logic (pseudocode):
    c1: plat.ram_total_frames == mm.summary.ram_frames or Err(C1{...})
    c2: mm.summary.ram_frames
        == mm.summary.allocatable_frames + mm.summary.protected_total
        or Err(C2)
    c3: pg.managed == sum(mm.allocatable_spans().counts) - pg.reserved_meta
        and pg.managed == pg.free + pg.used + pg.reserved_meta or Err(C3)
    c4: hp.pages_used <= pg.used
        and hp.pages_used == hp.slab_pages + hp.large_pages or Err(C4)
Validation: W06-DV05 (each check fired by injected divergence, each ID
  asserted distinctly).
```

## 5. `render`

```text
Name and stability: inspect::render(report: &InspectionReport,
  out: &mut dyn fmt::Write) -> Result<(), RenderError>. Internal; called by
  boot diagnostics, host tests, and (through W09's capture) QEMU evidence.
Purpose and caller: write the report as stable-order text; the evidence
  surface for P2-V08/P2-V11.
Inputs / outputs: report reference + sink; writes lines, returns sink
  failure if any.
Preconditions / postconditions: pre — report composed (C1–C4 held); post —
  sections appear in fixed order ([02 §2](02-architecture-and-state.md)),
  each section header exactly once, identical report => identical bytes.
Concurrency/allocation context: no allocation; one bounded line buffer;
  the sink is caller-supplied so no logger API is frozen (README
  Decision 2).
Errors and failure guarantee: RenderError on sink failure; the report is
  unchanged and may be re-rendered to another sink.
Security/authorization checks: render is a pure function of the report; no
  environment, no time; content-exclusion rules inherited from the section
  builders ([01 §3](01-scope-and-foundations.md) rule 4).
Logic (pseudocode):
    write("[boot-context]"); line(el, boot_cpu_id | "not recorded")
    write("[platform]");     lines(cpu..., gic, timer, psci, console,
                                  bootargs, ram_total, reserved_counts,
                                  counters..., capabilities...)
    write("[memory-map]");   lines(per-class protected totals,
                                  allocatable spans "first=.. count=..",
                                  summary totals, warnings, anomalies)
    write("[page-allocator]"); lines(managed/free/used/reserved total,
                                     per-region rows in region order)
    write("[heap]");         lines(budget, used, headroom,
                                   per-class rows in class order, large)
    # no [consistency] section: a published report already passed C1–C4
Validation: W06-DV06 (byte-stability), DV09 (bounded, allocation-free).
```

## 6. `BootContextRecord` (consumed, not owned)

```text
Name and stability: BootContextRecord — the A5 P1 record's renderable
  shape. W06 does not define P1's producer; if P1 names its own type, W06
  adapts to it at integration (a W06 revision, recorded).
Purpose: render exception level and boot-CPU hardware identity as recorded
  at P1 entry.
Fields (minimum): exception_level (El1|El2 marker as P1 recorded),
  boot_cpu_identity (opaque recorded value, e.g. MPIDR-class), source marker.
Failure boundary: absent record => sections::boot_context returns the
  NotRecorded marker; never a fabricated value ([01 §2](01-scope-and-foundations.md) A5).
Validation: W06-DV08 (marker renders; no panic).
```

## 7. Explicitly unauthorized interfaces

No `Display`/`Debug` impls that render fact payload content (log-content
hygiene, matching W02's rule); no `Into<String>`/allocation-based rendering;
no access to `PageAllocator`/`Heap` beyond `stats()` (calling `audit()` is
the boot caller's labeled diagnostics step, not composition's); no mutation
API on `InspectionReport` (constructed once, immutable); no comparison
against a previous boot (Reserved); no serialization. W09 and W08 consume
render output and check IDs as *expectations*, never as callable hooks.
