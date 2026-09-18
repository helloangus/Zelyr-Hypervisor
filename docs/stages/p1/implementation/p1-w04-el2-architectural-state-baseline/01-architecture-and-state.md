# P1-W04 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W04 detailed design](README.md).

## 1. Logical module map

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Control-write layer | perform each register write per its recorded specification | none (the registers themselves are machine state owned stage-wide per §3) | write specs, W03 facts for guards | written registers | verification, declaration, consumer policy |
| Read-back verification | masked re-read and compare of every write | none | the write log | per-write verification result | fixing or retrying a mismatch (fatal, no retry) |
| Baseline declaration | publish per-category establishment status for consumers | `EL2_BASELINE` static | verified write results | query API | re-reading live registers; consumer interpretation |
| Establishment body | the `el2-baseline` phase's single entry: assert preconditions → write (guarded) → verify → declare | orchestration only | W09 phase call | baseline established or fatal route | sequencing around the phase (W09), vector installation (W05) |

The establishment body is the mechanism entry W09's `el2_baseline_step`
adapter calls (W09 §8 seam table); its contract is
[02-code-contracts-control-writes.md](02-code-contracts-control-writes.md)
§1.

## 2. Baseline categories and their capability-fact mapping

The categories are exactly P1-V07's list plus execution state. Each maps to
the W03 facts it is guarded or justified by (work seq 1; ADR-044):

| Category | Controls (owner: W04 unless noted) | W03 facts consumed |
|---|---|---|
| C1 Execution state | `SPSel=1`, `DAIF` all-masked — **established by W02**, asserted here via read-back | (none; these are W02 invariants, not CPU facts) |
| C2 Exception routing | `DAIF` mask (assert); physical-interrupt routing posture via `HCR_EL2` IMO/FMO/AMO = 0; `SCR_EL3` explicitly **not owned** (firmware domain, ADR-008) | `ExecutionLevel` (the category is only meaningful at EL2) |
| C3 Trap policy | `HCR_EL2` = `RW` only (no trap groups enabled; TGE=0) | `Stage2Support` (context for the reserved VM field — read as fact, not acted on) |
| C4 FP/SIMD | `CPTR_EL2.TFP=1` (deny at EL2); `CPACR_EL1.FPEN=00` (deny at EL1/EL0) | (none; deny-by-default posture) |
| C5 Debug/performance | `MDCR_EL2` trap bits (TDA, TDOSA, TDE, TPM, TPMCR); `MDSCR_EL1 = 0` | (none) |
| C6 Timer | `CNTHCTL_EL2` EL1 gates = 0 (EL1 physical timer access denied); `CNTKCTL_EL1 = 0` (EL0 virtual-timer/event access denied); `CNTHP_CTL_EL2` disabled+masked; `CNTHV_CTL_EL2` disabled+masked **guarded** | `El2VirtualTimer` (guard), `CounterFrequency`, `El2PhysicalTimer` |
| C7 EL1/EL0 preparation | `HCR_EL2.RW=1` (EL1 AArch64); `CPACR_EL1` (C4); `CNTKCTL_EL1` (C6); `SCTLR_EL1` M/C/I cleared by mask (no EL1 MMU/cache residue) | `ArchProfile` (confirms an EL1 exists to prepare for) |
| C8 Translation controls | `SCTLR_EL2` M/C/I = 0 (MMU/cache off); `VTCR_EL2 = 0`; `VTTBR_EL2 = 0`; `HCR_EL2.VM = 0` (Stage-2 disabled) | `PaRange`, `Granule4k` (context recorded for W08; W04 does not act on them) |

`VBAR_EL2` is **absent by design**: vector installation is W05's mechanism,
and writing a vector base before vectors exist would fabricate an invariant.
The baseline records the category boundary "vectors unowned until W05" as
part of the known-state boundary (§5).

## 3. Register-ownership matrix (one owner, stage-wide)

| Register | Owner | Written when | Everyone else |
|---|---|---|---|
| `SPSel`, `DAIF` (PSTATE) | W02 establishment | image entry, once | W04: read-back assert only; never rewrite |
| `SCTLR_EL2` | W04 | establishment | W08 writes its post-MMU value **through its own design**, which supersedes the value via a recorded design change — not by local edit here |
| `HCR_EL2` | W04 | establishment | later stages (P4+) via superseding designs |
| `CPTR_EL2`, `CPACR_EL1` | W04 | establishment | — |
| `MDCR_EL2`, `MDSCR_EL1` | W04 | establishment | — |
| `CNTHCTL_EL2`, `CNTKCTL_EL1`, `CNTHP_CTL_EL2`, `CNTHV_CTL_EL2` | W04 | establishment (CNTHV guarded) | P6 timer virtualization supersedes via its own stage |
| `SCTLR_EL1` | W04 (mask-write) | establishment | later EL1-entry stage supersedes |
| `VTCR_EL2`, `VTTBR_EL2` | W04 (zeroing) | establishment | P4 Stage-2 supersedes |
| `VBAR_EL2` | W05 | its `exceptions` phase | nobody before; W04 records it unowned |
| UART/identity/other registers | their own packages | — | — |

A control with two writers is a review failure in whichever design adds the
second writer. Supersession is legitimate only as a recorded design change
in the superseding package.

## 4. Establishment lifecycle (write-once, monotone)

```text
UNESTABLISHED (firmware residue — the state W02 delivered)
  -> precondition assertion (phase-1 completed; execution state per C1)
  -> C2..C8 writes in the recorded order (guarded where a fact guard exists)
  -> read-back verification of every write (C1 assert included)
  -> ESTABLISHED (all categories Established) — declared via the API
any write whose verify mismatches, or a guard inconsistency:
  -> BaselineError routed via the panic route (phase-attributed el2-baseline)
  -> status of categories after the failure point remains NotEstablished;
     nothing is rolled back (W09 T2/T3: monotone, terminal)
```

Rules: establishment executes exactly once, on the boot CPU, inside the
`el2-baseline` phase; the write order is the category order C1→C8 (denial
posture lands before any optional/guarded write); no category is skippable
except its explicitly guarded optional elements.

## 5. Known-state boundary and continuation prerequisites

The baseline's exit condition — what W05/W08/W09 may rely on — is exactly:
every C1–C8 category `Established`, every recorded value per
[02-code-contracts-control-writes.md](02-code-contracts-control-writes.md),
and the two recorded boundary facts: (a) vectors are unowned until W05
declares them, so exceptions taken before W05's declaration are outside
P1's owned surface (the W09 unowned-window limitation, restated not
re-owned); (b) the values are P1's minimal baseline, not a guest or
future-stage policy — supersession happens only through the owning later
design. Consumers assert categories through the declaration API
([03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md)
§4) instead of assuming or re-reading.

## 6. Unsupported and unavailable elements

| Situation | Behavior | Rationale |
|---|---|---|
| A guarded optional element's fact is Absent (e.g. `El2VirtualTimer`) | skip the write; record the skip in the declaration (`SkippedAbsent`); continue | ADR-044: capability-driven; an absent element cannot be "established", and faking it would be residue |
| A required context fact is not `Established` (W03 phases) | invariant violation → panic route (cannot happen in a legal lifecycle; the precondition assert catches lifecycle misuse) | W09 H1: declared prerequisites only |
| A write's read-back mismatches | fatal `BaselineError { control, expected, observed }` via the panic route | decision 6; no retry — retrying would mask hardware/firmware divergence |
| A control is architecturally UNKNOWABLE at EL2 (none in P1's set) | — recorded boundary: the set was chosen so this case does not exist; adding one is a design change | honesty about detection limits (same posture as W01's E3 assumption) |

## 7. Concurrency model

Boot CPU only; `DAIF` masked (C1 invariant); no allocation; no locks. All
writes are context-synchronizing system-register writes in a straight-line
boot context; ordering between them needs no barriers beyond the
architecturally serializing nature of `msr` to these controls (recorded as
an implementation note: any write whose later stage depends on instruction
visibility — none in P1 — would require the barrier discipline of the
Coding Guidelines' MMIO/register section).

## 8. Assumed contracts and failure boundaries

| Seam | Supplied by | Used for | Failure boundary |
|---|---|---|---|
| `el2-baseline` phase placement; tracker attribution | [W09](../p1-w09-initialization-sequencing/README.md) (accepted design) | when establishment runs; how failures are attributed | seam mismatch raised per W09 §1 |
| Panic route | [W02](../p1-w02-minimal-rust-el2-runtime/README.md) (accepted design) | the fatal route of §6 | if the route cannot carry the error fields, W02/W04 coordination issue; no second route is built |
| Capability facts and query API | [W03](../p1-w03-aarch64-capability-inventory/README.md) (accepted design) | guards (C6), context (C3/C7/C8) | a missing fact is a W03/W04 coordination issue; W04 does not re-read identification registers |
| No-FP build guarantee | P0 target semantics (planned) | C4's validity (denying FP while the compiler emits it would fault on our own code) | recorded dependency; verification via review, with the toolchain-inspection reservation as the recorded escalation if deeper evidence is needed |
| Consumer needs beyond C1–C8 | W05/W08 (parallel designs) | Reserved extension triggers | their designs extend the baseline through their own recorded changes, not by editing W04 |

Produced for consumers: the verified baseline, the declaration API, the
boundary facts, and the baseline record for W12.
