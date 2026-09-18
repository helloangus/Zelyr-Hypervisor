# P1-W08 Mapping-Class Assignment and Transition Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W08 detailed design](README.md).

Pseudocode is an outline, not runnable production code. Exact `TCR_EL2`/
`SCTLR_EL2`/descriptor field values are recorded against the architecture
revision in the implementation record (the W04 discipline); this file
fixes the *semantics* that the recorded values must realize.

## 1. Class-to-region assignment (work seq 2)

The complete assignment; any region outside this list is unmapped by
design, and any unmapped access from P1 code is a fault (the containment
property W11's NC5 depends on).

| Region (inventory row) | Class | Required attributes (semantics) |
|---|---|---|
| 1 Executable code | `CodeRx` | Normal, inner WB read-allocate; execute permitted; writes fault |
| 2 Read-only data | `RoData` | Normal, inner WB read-allocate; execute never; writes fault |
| 3 Writable/zero data (incl. `STAGE1_TABLES`) | `DataRw` | Normal, inner WB; execute never; read/write |
| 4 Boot stack | `BootStack` | Normal, inner WB; execute never; read/write (distinct review unit from row 3 — architecture file K1) |
| 5 Vector table | `Vectors` | Normal, inner WB read-allocate; execute permitted; writes fault |
| 6 Early-console MMIO | `ConsoleMmio` | Device, non-cacheable (nGnRE semantics); execute never; read/write |

The firmware DTB and all free RAM: unmapped (architecture file §2 notes).

## 2. Prohibited ambiguous attributes (work seq 2)

Each rule is a review gate (W08-DV02/DV05) and, by §5 of the address-type
contracts, a runtime-checked property of the tables:

- P1 **No RWX anywhere**: no page is both writable and executable — the
  class model cannot express it and verification rejects it.
- P2 **No executable Device memory, no cached MMIO**: Device pages are
  execute-never and non-cacheable by construction.
- P3 **No writable code or vector pages, no executable data or stack**:
  the code/vectors are read-only; data/stack are execute-never (the W05
  vector-table immutability becomes hardware-enforced).
- P4 **No uncached code/data, no unspecified shareability**: Normal
  classes are WB with the recorded shareability; "whatever the firmware
  left" is not an attribute value (the W04 residue principle applied to
  mappings).
- P5 **No block descriptors, no contiguity hints, no huge pages** (parent
  README decision 2; Reserved for later stages with their own designs).
- P6 **No mapping outside the inventory**: the verification's closure
  check (set equality both ways) is the mechanical form.

## 3. The transition — `enable_host_stage1()` (work seqs 3–4)

```text
Name and stability: fn enable_host_stage1(); internal; stable within P1.
  (Naming rule: W09 owns `stage1_step`; the supplying mechanism does not
  reuse it — W03 §6 precedent.)
Purpose and caller: the `stage1` phase's entire mechanism — assert,
  build, verify, enable, verify. Caller: run_init_sequence via the W09
  `stage1_step` adapter (W09 §8: "Stage-1 transition mechanism;
  post-MMU continuity").
Inputs / outputs: none; on normal return the post-MMU environment of §7
  holds; on failure it does not return.
Preconditions / postconditions: W09 phase prerequisites (fatal-path
  complete; console complete). Postcondition: MMU on at EL2 with the
  recorded TCR/TTBR0/SCTLR values; the tables verified; the environment
  of §7.
State and ownership change: STAGE1_TABLES (built once); TTBR0_EL2,
  TCR_EL2 (W08-owned from here); SCTLR_EL2 M/C/I (supersedes W04's C8a
  through this recorded design); architectural TLB/I-cache state;
  nothing else.
Concurrency/allocation context: boot context; DAIF masked; no
  allocation; single-shot.
Errors and failure guarantee: any premise mismatch, build/verify error,
  read-back mismatch, or post-MMU verification failure -> fail_stage1 ->
  fail_phase(Stage1, reason) -> W07's phase-failure report (W09 matrix
  `stage1` row). A fault during the sequence -> the W05 exception path ->
  the armed full report. No retry, no rollback, no partial enable.
Security/authorization checks: the premise assertions and table
  verification are the containment gates (no-RWX etc. reach the hardware
  only through them).
Logic:
  # 1. premises (all recorded-value assertions, W04 decision 8 posture)
  baseline_value(SCTLR_EL2).M == 0  else fail_stage1(Premise, "mmu-on")
  baseline_value(HCR_EL2).VM == 0   else fail_stage1(Premise, "vm-on")
  baseline_status(C8) == Established else fail_stage1(Premise, ...)
  vector_status() == Established    else fail_stage1(Premise, ...)
  fatal_path_ready()                else fail_stage1(Premise, ...)
  granule/fact queries published    else fail_stage1(Premise, ...)
  # 2. facts (W03 direct query; parent README decision 3)
  pa = CAPABILITIES.query(PaRange); g4 = CAPABILITIES.query(Granule4k)
  # 3. build + verify (pure table work; no hardware touched yet)
  build_stage1_tables(MAPPED_REGIONS)?     # Err -> fail_stage1(Build, ..)
  verify_stage1_tables(MAPPED_REGIONS)?    # Err -> fail_stage1(Verify, ..)
  # 4. barriers and invalidation (recorded reasoning, decision 6)
  barrier_dsb_ish(); tlb_invalidate_alle2(); barrier_dsb_ish()
  ic_invalidate_all(); barrier_dsb_ish()
  # 5. program translation controls (values recorded per revision)
  ttbr0_write(L1 base PA); tcr_write(tcr_value(pa, g4)); barrier_isb()
  # 6. enable (the only SCTLR mutation in P1; supersedes W04 C8a)
  sctlr_write(sctlr_read() | M | C | I); barrier_isb()
  # 7. post-MMU verification (§5 below)
  post_mmu_checks()?                       # Err -> fail_stage1(PostVerify, ..)
Validation: W08-DV03 (sequence/order), W08-DV04 (continuity rows), W08-DV05
  (constraint review).
```

Step order is normative: tables are built and verified before any
register changes; the enable step is last and single; the barrier
placements implement the recorded cache/TLB reasoning (parent README
decision 6: tables written with caches off need no clean; `ic iallu`
before enabling `I`; `tlbi alle2` ensures no stale translation survives).

## 4. Recorded translation values (semantics)

| Register | Recorded semantics |
|---|---|
| `TCR_EL2` | start level and `T0SZ` derived so the identity windows are reachable (derivation recorded); granule 4 KiB (from the `Granule4k` fact — a Required fact, asserted); PS/IPS from the `PaRange` fact; Normal windows inner WB; Device window via the descriptor attributes; shareability per K3 (non-shareable, single CPU) |
| `TTBR0_EL2` | the L1 table's physical base, within the documented bits; no ASID semantics at EL2 |
| `SCTLR_EL2` | `M=1, C=1, I=1` over the pre-MMU recorded value (read-modify-write on the read value so W04's other recorded bits are preserved); the post-MMU recorded value is this design's, superseding C8a per the ownership matrix |

Break-before-make: **not applicable in P1** — no entry is ever modified or
removed after verification; the rule enters with the first remap-owning
stage (P2+), recorded here so its absence is a decision, not an omission.

## 5. Post-MMU verification (work seq 3)

```text
Name and stability: fn post_mmu_checks() -> Result<(), Stage1Error>;
  internal; stable within P1.
Purpose and caller: the dynamic half of parent README decision 5. Caller:
  the transition body's last step.
Inputs / outputs: none; ok or the first failure.
Preconditions / postconditions: executing with the MMU enabled (this
  function itself is mapped code — execution reaching it is the first
  dynamic proof); pure observation apart from the one data sentinel
  write.
State and ownership change: one word in a .data sentinel static.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: any failure routes (PostVerify) — a
  half-working map is never reported as success.
Security/authorization checks: none.
Checks (each with its recorded reason):
  sctlr_read(): M/C/I observed set (the write took effect)
  read a known .rodata constant and compare (RoData mapped, readable)
  write/read the .data sentinel (DataRw mapped, writable)
  # stack correctness is implicit: this call chain runs on it
  # vectors and console are NOT probed here — their liveness is proven
  #   by W09's post-stage1 marker (console) and NC3/NC6 (vectors), per
  #   the architecture file §6 ownership split
Validation: W08-DV03; NC5's insertion point depends on this function's
  normal return (§6).
```

## 6. The NC5 continuation point (handoff to W11)

```text
Name and stability: the post-MMU continuation point; a defined location,
  not a function: the first sequencer action after enable_host_stage1()
  returns normally (all §5 checks passed), i.e. before W09 records
  stage1 completion.
Purpose: the insertion contract [W11's trigger
  containment](../p1-w11-negative-fault-validation/01-fault-scenario-matrix.md)
  §3 names ("inserted after the MMU-enabled continuation point W08
  defines"). Owner: this design defines it; W11 inserts per its
  selection mechanism.
Guarantees at the point: MMU on; all six classes mapped and verified;
  fatal path armed; vectors live and VBAR unchanged; console window
  mapped (its liveness proof is the post-stage1 marker, one step later).
  An access here to any address outside the inventory faults into the
  armed full report with `stage1`-or-later attribution — NC5's expected
  class and outcome.
Validation: W08-DV06 consumability; NC5 execution (W11, deferred).
```

## 7. The post-MMU environment statement (handoff to W09–W12 and P2)

After `enable_host_stage1()` returns and W09 records `stage1` completion,
the stable environment is: EL2 execution with the MMU, caches, and
instruction cache enabled under the recorded values; every mapped byte in
one of the six recorded classes; the identity window and fixed console
window recorded as temporary assumptions (W12's contract content); the
fatal path, vectors, and console all live on mapped memory; no map/unmap
service, no allocator, and no further mapping change exists in P1. P2
supersedes the assumptions through its own designs (relocation, new
regions, discovered memory) and inherits no P1 page-table code as a
precedent.
