# P4-W06 Code Contracts — Isolation Expectations, Report, and Dump

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W06 detailed design](README.md).  
**Companion:** IS-series matrix and containment model in
[02-architecture-and-state.md](02-architecture-and-state.md).

All names are P4-internal and unstable-by-declaration; P4-W09 records them as
implemented facts only. Pseudocode is an outline; the Coding Guidelines govern
the final Rust shape. Buffer types are fixed-capacity, stack-allocated value
types with checked appends; no heap allocation exists in this design.

## 1. `IsolationExpectation` (module `isolation-expect`)

- **Name and stability:** `IsolationExpectation { id: ExpectationId, vg_ref:
  ScenarioId, probe_class: ProbeClass, expected_class: ExitClass,
  expected_access: Option<FaultAccess>, ipa_predicate: IpaPredicate,
  expected_mapping: Option<ExpectedMapping>, el2_live_required: bool }` with
  `ProbeClass — UnmappedGap | RamBoundary | HypervisorOwned | RoWindow |
  XnWindow | NonMemory`, `IpaPredicate — Equal(GuestPhysAddr) | Within(
  GuestIpaRange) | None`, and `ExpectedMapping — Unmapped | MappedWith(
  S2Flags)`. Internal.
- **Purpose and caller:** the machine-checkable form of one IS-series row;
  consumed by `check_expectation`, by W07's episode driver (which declares
  which expectation applies to the running scenario), and by W08 (expected
  evidence in the automation manifest).
- **Preconditions/construction:** constructed only from the static `IS-T1`
  table; ad-hoc construction at run time is not part of the contract (the
  matrix is versioned data, not a runtime API).
- **Contract notes:** the table version bumps jointly with the VG table
  ([02 §6](02-architecture-and-state.md); open item O2). Probe address values
  arrive through the W03/W05 boot-info channel (open item O1); the expectation
  stores the *predicate*, and the probe values it references are resolved from
  the validated boot-info view by the episode setup, never invented here.
- **Validation:** table review (every IS row has a well-formed expectation);
  cross-consistency review against W05's `VG-T<n>` expected outcomes
  (DV11).

## 2. Probe-class membership (module `isolation-expect`)

- **Name and stability:** `fn probe_membership(probe_class: ProbeClass,
  probe: GuestPhysAddr, layout: &GuestLayout, platform: &PlatformFacts) ->
  Result<MembershipOk, ProbeError>`. Internal.
- **Purpose and caller:** assert that a probe address really lies where its
  class claims (gap, RAM boundary, Hypervisor-owned range), so IS-01–IS-03
  prove what they name; called by episode setup before the Guest runs.
- **Inputs/outputs:** probe address, the W03 layout record, and P2-derived
  normalized platform facts (protected/Hypervisor ranges, W01 row R08) →
  membership confirmation or a named error.
- **Preconditions:** layout already validated (W03); platform facts already
  normalized (P2, assumed contract).
- **Postconditions:** on success, the probe is a page-aligned IPA outside all
  mapped grants for `UnmappedGap`; exactly one page past the RAM end for
  `RamBoundary`; inside a protected/Hypervisor-owned range and outside all
  grants for `HypervisorOwned`.
- **Errors:** `ProbeInsideMappedRegion` (would not fault — a setup-contract
  bug, fatal class for the episode, not run), `ProbeMisaligned`,
  `FactsUnavailable` (platform facts absent — blocked prerequisite per M1/P2,
  recorded, never guessed around).
- **Security/authorization checks:** this function is the guard that keeps
  isolation evidence honest — a probe that is accidentally mapped would
  otherwise turn a negative test into a silent positive run. All inputs are
  host-authored; the Guest cannot influence any of them.
- **Logic:**

```text
probe_membership(class, probe, layout, platform):
    if !probe.page_aligned(): return Err(ProbeMisaligned)
    match class:
      UnmappedGap:
          if layout.ram_span().contains(probe): check not in any special region
          // gap probes live in the declared IPA space outside granted ranges
      RamBoundary:
          require probe == layout.ram_end()                 // exact boundary
      HypervisorOwned:
          require platform.protected_ranges().any(|r| r.contains(probe))
          require !layout.ram_span().contains(probe)
    if grants_or_special_regions_contain(probe): return Err(ProbeInsideMappedRegion)
    return Ok(MembershipOk)
```

- **Validation:** unit tests per class incl. the rejection cases; on-target:
  IS-01–IS-03 passing implies membership held (the fault occurred as
  expected).

## 3. Expectation matching (module `isolation-expect`)

- **Name and stability:** `fn check_expectation(diag: &ExitDiagnostic, exp:
  &IsolationExpectation) -> MatchVerdict` with `MatchVerdict — Match |
  Mismatch(BoundedList<MatchReason>)`. Internal.
- **Purpose and caller:** turn one diagnosis into a determinate verdict for
  the run record; called by W07's episode driver after `diagnose`.
- **Inputs/outputs:** the diagnostic and the declared expectation → verdict
  with bounded, named reasons.
- **Preconditions:** the expectation is the one declared for the running
  scenario (caller duty; a scenario/expectation mismatch is a setup error,
  not a `Mismatch` verdict).
- **Postconditions:** `Match` iff every applicable field agrees: class,
  access (when declared), IPA predicate (when declared), mapping state
  (when declared), `mapping_agreement == Consistent` (when a comparison
  applied), and `el2_live_required` is satisfiable (the run continues —
  observed by W07, not by this function). `Mismatch` lists every failed
  dimension; nothing is partial or heuristic.
- **State/ownership change:** none.
- **Concurrency/allocation:** no allocation (reason list is a fixed-capacity
  value); constant time.
- **Errors:** none (mismatch is a value, not an error).
- **Security/authorization checks:** consumes the diagnostic's Guest-derived
  fields as data; a mismatching Guest cannot forge a `Match` because every
  dimension is compared against host-authored expectation data.
- **Logic:**

```text
check_expectation(diag, exp):
    reasons = []
    if diag.class != exp.expected_class: reasons.push(ClassMismatch)
    if let Some(acc) = exp.expected_access, diag.access != Some(acc):
        reasons.push(AccessMismatch)
    match exp.ipa_predicate:
      Equal(a)    if diag.ipa != Available(a): reasons.push(IpaMismatch)
      Within(r)   if !r.contains(diag.ipa):    reasons.push(IpaMismatch)
      None        -> pass
    match (exp.expected_mapping, diag.mapping):
      (Some(Unmapped),   Some(Unmapped))            -> pass
      (Some(MappedWith(f)), Some(Mapped(m))) if m.flags == f -> pass
      (Some(_), _)       -> reasons.push(MappingMismatch)
      (None, _)          -> pass
    if diag.mapping_agreement == Mismatch: reasons.push(StaleTranslation)
    if reasons.empty() -> Match else Mismatch(reasons)
```

- **Validation:** exhaustive unit tests per reason dimension and per IS row
  (positive and mutated-diagnostic negatives); on-target verdicts travel in
  the W07 run record and W08 evidence (DV05–DV09).

## 4. Bounded diagnostic report (module `diag-report`)

- **Name and stability:** `fn format_diagnostic(diag: &ExitDiagnostic, out:
  &mut ReportBuf) -> Result<(), ReportError>` with `ReportError —
  BufferTooSmall`. Internal.
- **Purpose and caller:** the single human-readable rendering of a
  diagnosis; called by the exit handler's post-stop step; re-renderable from
  the retained record.
- **Inputs/outputs:** diagnostic + fixed-capacity buffer → formatted text
  (line-oriented, one fact per line) or `BufferTooSmall` with the buffer
  untouched beyond a truncation marker (D9: truncation is annotated, never
  silent).
- **Preconditions:** called after the stop (containment-first, D7); buffer
  provided by the caller (episode-owned static buffer).
- **Postconditions:** the rendered text is a deterministic function of the
  record; unavailable fields render as explicit markers (for example
  `ipa=unavailable`), never blanks; output includes build identity per the
  P0 diagnostic-metadata rule (assumed M5).
- **Concurrency/allocation:** no allocation; pure formatting; callable once
  per episode in P4.
- **Errors:** `BufferTooSmall` only; surfaced to the caller (which retries
  once with the oversized-buffer fallback defined in the workflow, then
  records the report-skipped fact — a report must never be silently absent).
- **Security/authorization checks:** renders Guest-derived values as data;
  no formatting-time interpretation; the fixed field set prevents
  format-string-class bugs (no dynamic format selection).
- **Logic:**

```text
format_diagnostic(diag, out):
    write(out, "HV-DIAG begin")                 // fixed vocabulary, one line
    line(out, "class", diag.class)
    line(out, "esr", hex(diag.raw_esr)); sub_fields(out, diag.view)
    line(out, "guest_pc", hex(diag.guest_pc))
    line(out, "ipa", diag.ipa)                  // hex or "unavailable"
    line(out, "mapping", diag.mapping)          // unmapped / flags+frame
    line(out, "agreement", diag.mapping_agreement)
    line(out, "stop", diag.stop_cause)
    line(out, "build", build_identity())        // P0 metadata rule
    write(out, "HV-DIAG end")
```

- **Validation:** golden-text unit tests (deterministic rendering, truncation
  marker, `unavailable` markers); grammar review with W08 (the report is
  human evidence; the machine evidence is the run record, not this text).

## 5. Ledger-derived mapping dump (module `diag-report`)

- **Name and stability:** `fn dump_mapping_state(space: &GuestAddressSpace,
  out: &mut ReportBuf, budget: EntryBudget) -> Result<DumpStats,
  ReportError>`. Internal.
- **Purpose and caller:** satisfy the task book's Stage-2 fault "page-table
  dump" intent with the authoritative mapping state (W02 D9: the ledger);
  called post-stop for translation/permission faults.
- **Inputs/outputs:** the (now quiescent) address space, buffer, and an entry
  budget → per-range dump text plus stats (ranges rendered, truncated flag).
- **Preconditions:** space is not `Active` (post-stop; P4's sequencing makes
  this automatic); space lock is free or taken internally by W02's ledger
  read path — the dump uses W02's read surface only, never descriptor
  memory.
- **Postconditions:** the dump covers the whole ledger or stops at the budget
  with an explicit `truncated=true` stat; entries render as IPA range →
  frame base + flags; no content beyond mapping metadata is rendered (never
  Guest data bytes).
- **State/ownership change:** none (read-only).
- **Concurrency/allocation:** no allocation; bounded by the budget; runs in
  setup context after stop (no time pressure).
- **Errors:** `BufferTooSmall` (annotated truncation, same rule as §4).
- **Security/authorization checks:** renders mapping metadata only; frame
  addresses are Host-owned data rendered for humans, never exposed to any
  Guest-readable path; no hardware walk exists to mislead (D8).
- **Logic:**

```text
dump_mapping_state(space, out, budget):
    for range in space.ledger_ranges():            // W02 read surface
        if budget.exhausted(): return Ok(DumpStats { truncated: true, .. })
        line(out, range.base, range.pages, range.flags, range.frame_base)
    return Ok(DumpStats { truncated: false, .. })
```

- **Validation:** unit tests with a stubbed ledger (empty, single, many,
  budget-truncated); on-target: dump present for every IS-01–IS-05 episode
  (DV07); consistency review that dump entries agree with the diagnostic's
  snapshot for the faulting IPA.

## 6. Error model recap

- `ReportError::BufferTooSmall` is the only W06 error value; it is always
  surfaced, never swallowed (a missing report or dump is recorded, not
  hidden).
- `ProbeError` values (`ProbeInsideMappedRegion`, `ProbeMisaligned`,
  `FactsUnavailable`) are setup-phase values: the first two are host-authored
  contract violations (fatal episode-setup class per W03's `LayoutMismatch`
  precedent), the third is a blocked-prerequisite condition recorded per M1.
- All Guest-caused conditions arrive as `ExitDiagnostic` data, never as
  errors — preserving the W01 A2 / W04 type-split boundary.
- `Mismatch` verdicts are values consumed by the run record; they fail the
  episode's evidence, not the Hypervisor.
