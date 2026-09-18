# P1-W02 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W02 detailed design](README.md).

## 1. Logical module map

W02's logical modules are boot-scope units inside the boot-path module tree.
Their physical crate/file placement is owned by the P0 workspace baseline and
the module-tree decisions of the designs that follow; this design owns the
logical boundaries and every contract that crosses them.

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Boot entry (assembly) | W01 pre-transfer tier, establishment sequence, transfer to Rust | boot stack region (static), BSS (zeroed once) | firmware-delivered machine state, image load state | a running Rust entry with E1/E7 validated and the establishment discipline applied | DTB content, control-register policy (W04), console, panic handling |
| Rust entry and establishment | record `entry`/`runtime` events, build `BootContext`, order the runtime readiness steps | `BOOT_CONTEXT` (static, immutable after establishment) | W01 transfer guarantee (x0–x3), W09 tracker | the sequencer seam call with the runtime ready | lifecycle position (W09), sequencer content, capability/baseline mechanisms |
| Boot context retention | keep the boot parameters addressable for W03–W09 and P2 | `BOOT_CONTEXT` fields | x0–x3 at transfer | typed accessors | interpretation of the DTB, discovery, any P2 semantics |
| Early panic route | bounded, non-recursive panic behavior for the whole stage | single-entry guard flag | `PanicInfo` | one bounded report line set through the early writer; a terminal stop | crash-report content (W07), channel format (W06), recovery |
| Build identity | answer "what am I running?" for every diagnostic | static `BuildIdentity` data | P0-W16 metadata contract | `&'static BuildIdentity` accessor | wire format, versioning policy (P0-W16), telemetry |
| Early diagnostic writer | raw polling reference-UART byte output for the panic route | none (stateless) | byte slices | UART writes | channel abstraction, markers, formatting (W06/W07), init/probing |
| Sequencer seam glue | the fixed call arrangement between runtime readiness, W09's sequencer, `stable`, and idle | none | `run_init_sequence()` result regime | the `stable` record; entry into idle | sequencer internals, phase order, marker tokens |
| Controlled idle | terminal regime of the integrated boot path | none | entry after `stable` | none (never returns) | wake-up consumers, power management, scheduler idle |

## 2. Establishment order (the only sequence W02 owns)

The establishment order is a fixed, straight-line requirement. It is not a
state machine: boot position lives in W09's tracker (parent README decision
2), and each stage below is an invariant established once, never re-entered.

```text
image entry (firmware handoff, machine state per W01 contract §2)
  1. W01 pre-transfer tier          [W01 contracts; reject otherwise]
  2. SPSel = 1; SP_EL2 = stack top; DAIF all-masked
  3. BSS zeroed; static data valid by load (no relocation processing)
  4. transfer: el2_rust_entry(x0, x1, x2, x3)          [assembly ends]
  5. record Entry.enter, Entry.complete   (W01 transfer guarantee; W09 tracker)
  6. record Runtime.enter
  7. BootContext built and published; identity linkable by diagnostics
  8. panic route ready (handler linked; early writer reachable; guard clear)
  9. record Runtime.complete
 10. run_init_sequence()             [W09; phases capabilities..stage1]
 11. record stable                    (glue)
 12. controlled_idle()                [never returns]
```

Rules:

- O1 Every stage 1–8 is idempotent-free: executed exactly once per boot, on
  the boot CPU, with no re-entry path.
- O2 Stages 1–4 are assembly and must not depend on Rust data being ready;
  stage 3 completes before any Rust instruction executes.
- O3 Stages 5–9 record observable events at their boundaries (W09 M1), so
  P1-V03's "established in order" is reviewable from the tracker contract.
- O4 Stages 10–12 exist as contracts from the start but compile only when
  W09's items exist (parent README decision 6) — the wiring consequence is
  recorded, never stubbed.
- O5 The panic route (stage 8) is "ready" as soon as it is linked, but the
  order places its readiness assertion before `Runtime.complete` so that any
  later-stage failure is attributable to a routed, reporting runtime.

## 3. Boot-state ownership register

Every piece of mutable boot state has exactly one owner:

| State | Owner | Written when | Read by | Never written by |
|---|---|---|---|---|
| Lifecycle position | W09 tracker | W02-owned records; sequencer | W07 diagnostics, W10 evidence | W02 internals (only via the record API) |
| Boot stack | W02 (region); the executing code (contents) | establishment; runtime use | — | any later package (no re-sizing, no reallocation) |
| BSS/static data | W02 (zeroing/readiness); owning modules (contents) | stage 3; load | everyone after stage 3 | — |
| `BOOT_CONTEXT` | W02 | stage 7, once | W03–W09 consumers, P2 (via handoff) | anyone after publication |
| Panic-entry guard | early panic route | first panic entry | the route itself | everyone else |
| UART data register | early diagnostic writer (post-transfer); W01 rejection reporter (pre-transfer) | on each emit | — | anything else in P1 (single-consumer rule) |

The single-consumer rule on the UART is temporal: before transfer only the
rejection reporter may write; after transfer only the early writer (and later,
through W06's channel, its transport). Two writers at once is a review
failure in either design.

## 4. Concurrency model

P1 has one executing CPU and no enabled interrupt delivery:

- `DAIF` is all-masked from stage 2 onward; W04's baseline keeps it masked;
  no P1 mechanism unmasks it. No lock, no atomic ordering beyond the two
  explicitly named boundaries (W09's compare-exchange tracker; this design's
  panic-entry guard), both justified for the single-core context.
- Exception context exists only after W05 installs vectors; before that, an
  exception is outside P1's owned failure surface (the W09 "unowned window"
  limitation, restated in W09 §4 — not re-owned here).
- No allocation exists anywhere in the runtime: no heap, no `alloc`, no
  dynamically sized data; every collection is a fixed-size array or static.
- Re-entrancy: only the panic route can execute while the boot path is
  mid-establishment, and the guard plus the route's design make that
  termination-only (§4 of
  [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md)).

## 5. Hidden-dependency rules (P1-V04 review core)

- H1 Self-establishment: the runtime must not execute any code path whose
  correctness depends on a machine state it did not establish and that is
  not declared in [W01's entry-state table](../p1-w01-reference-boot-contract/01-boot-contract.md)
  §3. The review walks every register read and memory access in stages 1–9
  against that table.
- H2 No firmware-residue policy: control-register content is neither read
  nor relied upon by W02; policy registers belong to W04, vectors to W05,
  the MMU to W08. A W02 item that reads a control register for a decision is
  a design violation.
- H3 No future-stage reach: no allocation, no GIC, no discovery, no
  Stage-2, no secondaries (W09 H3 alignment).
- H4 Reserved-boundary respect: W02 consumes W01's tier, W09's tracker, and
  the P0 baselines; it must not re-declare or modify any of them. A conflict
  is raised, never absorbed.

## 6. Assumed contracts and failure boundaries

| Seam | Supplied by | Used for | Failure boundary if it delivers differently |
|---|---|---|---|
| Entry-state table; tier + reporter contracts; transfer guarantee | [W01](../p1-w01-reference-boot-contract/README.md) (accepted design) | stages 1, 5 | the tier is implemented verbatim; any deviation is a recorded W01/W02 coordination issue |
| AArch64 bare-metal target; `no_std` semantics; panic/link extension points; ASM co-build | P0-W03 (planned) | everything executable | upstream defect recorded per
[05-implementation-and-review.md](05-implementation-and-review.md) §1; W02 does not invent a target |
| Linker/layout extension points; BSS bounds; image form | P0-W03 reserved layout extensions (planned) | stages 2–3; linker symbols | if the baseline provides no extension point, record the blocker; do not fork a private linker script |
| Failure-class semantics | P0-W14 (planned) | panic route's classification posture | route stays minimal and fatal-only; classification refinement is a recorded follow-up, not a local taxonomy |
| Build-metadata content and embedding mechanism | P0-W16 (planned) | `BuildIdentity` fields | identity degrades to a recorded "unavailable" literal per §4 of the identity contract; never fabricated |
| `BootPhaseTracker`, `phase_enter/complete`, `run_init_sequence`, phase vocabulary | [W09](../p1-w09-initialization-sequencing/README.md) (accepted design) | stages 5–6, 9–11 | seam mismatch stops the seam step; the conflict is raised to both owners (W09 §1 rule) |
| Marker/channel availability; marker format | [W06](../p1-w06-early-console-logging/README.md) (parallel) | supersession of the early writer for markers | none required by W02 — the early writer stays panic-only; W06's channel is additive |
| Crash-report fields and token classes | [W07](../p1-w07-fatal-crash-diagnostics/README.md) (parallel) | panic-body extension seam | the minimal body remains until W07's design lands; the seam is the recorded trigger |

Produced for consumers: the running runtime and stable execution context, the
published `BootContext`, the panic route and its marker class, the
`BuildIdentity` accessor, the region inventory for W08, and the seam wiring
for W09.
