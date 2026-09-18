# P1-W03 Classification and Report Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W03 detailed design](README.md).

Pseudocode is an outline, not runnable production code. No allocation; all
fixed-size data.

## 1. Classification

```text
Name and stability: Classification (Copy + PartialEq enum: Required |
  Optional | Future); internal; stable within P1.
Purpose and caller: the first taxonomy axis — what the fact means for P1
  continuation versus later consumers. Callers: FactRecord; the required
  check; rendering.
Inputs / outputs: plain data.
Preconditions / postconditions: every fact carries exactly one
  classification, fixed by the fact table
  ([01-architecture-and-state.md](01-architecture-and-state.md) §2) and not
  recomputed at runtime — classification is design authority, not data.
State and ownership change: none.
Concurrency/allocation context: none needed.
Errors and failure guarantee: cannot fail.
Security/authorization checks: none.
Validation: W03-DV02 taxonomy review.
```

## 2. `CapabilityRejection` — the fail-fast carrier

```text
Name and stability: CapabilityRejection { fact: FactId, reason: &'static str
  }; internal; stable within P1.
Purpose and caller: carry the named absence to the route. Callers: the
  required check produces it; the phase body routes it.
Inputs / outputs: fact identity plus a static reason string naming why the
  fact is required (the strings are the rejection vocabulary of §4).
Preconditions / postconditions: produced only by the required check; never
  constructed for Optional/Future facts.
State and ownership change: none.
Concurrency/allocation context: no allocation — two static strings, by
  construction renderable through W02's early panic writer (pre-console).
Errors and failure guarantee: cannot fail.
Security/authorization checks: none.
Logic: plain carrier.
Validation: W03-DV03 (NC2-observable vocabulary review).
```

## 3. Required-fact check

```text
Name and stability: fn verify_required(draft: &[FactRecord; N]) ->
  Result<(), CapabilityRejection>; internal; pure; stable within P1.
Purpose and caller: evaluate the Required set against the demanded
  observations. Caller: the phase body, immediately after draft
  construction and before publication.
Inputs / outputs: the draft records; ok, or the first rejection in
  `FactId::ALL` order (deterministic: one rejection per boot, never a
  random first-found).
Preconditions / postconditions: every fact's record present; pure; no
  mutation. On Err the draft is discarded — nothing partial is ever
  published (W09 H2: no partial-init continuation).
State and ownership change: none.
Concurrency/allocation context: no allocation.
Errors and failure guarantee: the Err value is the only failure
  representation; the check cannot panic.
Security/authorization checks: this is a continuation-policy gate, not a
  security gate (the CPU cannot lie about being an EL2 CPU in any way P1
  could detect anyway — honesty recorded, detection impossible is the
  documented bound).
Demanded observations (the whole Required policy):
  ExecutionLevel  == Present(EL2)
  Granule4k       == Present(supported)
  CounterFrequency== Present(freq) with freq != 0
Logic:
  for record in draft (ALL order):
    if record.classification == Required and not demanded(record.observation):
      return Err(CapabilityRejection { fact, reason: REQUIRED_REASONS[fact] })
  Ok(())
Validation: W03-DV03; NC2 consumes the vocabulary (W11).
```

## 4. Rejection vocabulary (fail-fast reason strings)

One static reason per Required fact, fixed before first verdict-bearing
evidence and never adjusted afterwards (W10 token precedent):

| Fact | Reason string (fixed literal) |
|---|---|
| `ExecutionLevel` | "required: Non-secure EL2 execution" |
| `Granule4k` | "required: 4 KiB granule support for P1 mapping work" |
| `CounterFrequency` | "required: non-zero system counter frequency" |

The route (phase body) renders `cap-reject fact=<label> <reason>` through
the panic route; the fact label comes from `FactId::label()`. The full line
format is recorded in the implementation record before W11 consumes it.

## 5. Published report and query API

```text
Name and stability: CapabilityReport { records: [FactRecord; N] } with
  static CAPABILITIES in a once-publication cell; internal; stable within
  P1.
Purpose and caller: the stage's single authority for capability knowledge.
  Callers: phase body (publish); W04, W08, W09 (query); the render step.
Inputs / outputs: publish(draft) — precondition: verify_required returned
  Ok; postcondition: records equal the draft; second publish is an
  invariant violation routed via the panic route.
  query(FactId) -> FactRecord — precondition: published (post-capabilities
  phase; consumers' contracts inherit this); postcondition: returns the
  published record, never a default.
State and ownership change: publication is the single write; interior
  immutable afterwards.
Concurrency/allocation context: the audited once-publication boundary
  (SAFETY: single boot CPU, DAIF masked, one boot path; readers exist only
  after publication — same pattern as W02's BootContext, one more audited
  cell in the P0 unsafe inventory).
Errors and failure guarantee: misuse is an invariant violation routed via
  the panic route with the tracker's phase attribution; the cell never
  returns a half-written report.
Security/authorization checks: none; knowledge, not authority.
Logic:
  publish: copy draft into the cell's storage; set the published flag
  query:   read the record for the id
Validation: W03-DV02/DV03; consumer reviews read the API, not the interior.
```

## 6. Phase body (the mechanism entry W09 calls)

```text
Name and stability: fn build_capability_report(); internal; stable within
  P1. (Naming rule: W09 owns the `<phase>_step` adapter names; the supplying
  mechanism deliberately does not reuse them.)
Purpose and caller: the `capabilities` phase's entire mechanism — extract,
  classify, check, publish. Caller: run_init_sequence via the W09
  `capabilities_step` adapter (W09 §8: "Capability mechanism +
  required/optional classification").
Inputs / outputs: none; on normal return the report is published and every
  Required fact conformed; on rejection it does not return (routes).
Preconditions / postconditions: phase prerequisites per W09 §1 (runtime
  complete); postcondition: CAPABILITIES published with the stage's facts.
State and ownership change: the publication of §5; nothing else.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: rejection routes via the panic route with the
  §4 vocabulary, phase-attributed `capabilities` by the tracker position
  (W09 matrix row); the function never returns mid-way.
Security/authorization checks: the required check (§3) is the only
  continuation gate in the body.
Logic:
  for fact in FactId::ALL: record = classify(extract(fact)); draft.push(record)
  verify_required(&draft)?           # Err -> route via panic route
  CAPABILITIES.publish(draft)
Validation: W03-DV03/DV04; W09-DV03 reads the body for hidden dependencies.
```

## 7. Render/emit contract

```text
Name and stability: fn render_report(emit: &mut dyn FnMut(&str)); internal;
  stable within P1.
Purpose and caller: deliver the report as bounded lines to the boot
  diagnostics. Caller: the W09 `console` phase wiring (after W06's channel
  is available), once per boot; not called during the capabilities phase.
Inputs / outputs: emit receives each complete line as a &str; lines follow
  the fixed form `cap <label>=<value> (<classification>)` with Absent/
  Unreadable rendered as their tokens; count and length are bounded by the
  fact table (no dynamic content beyond decoded field values).
Preconditions / postconditions: report published; emit callable; rendering
  is side-effect-free with respect to the report and performs no
  interpretation (W09 M4: markers identify phases; capability content is
  this output).
State and ownership change: none.
Concurrency/allocation context: no allocation (fixed stack buffer + the
  bounded formatter discipline of
  [W02's formatting contract](../p1-w02-minimal-rust-el2-runtime/04-code-contracts-panic-identity.md)
  §4); called once in boot context.
Errors and failure guarantee: a channel failure during rendering routes via
  the `console` row of W09's matrix; the report state is unaffected.
Security/authorization checks: none.
Logic:
  for record in CAPABILITIES.records (ALL order):
    format line into the bounded buffer; emit(line)
Validation: W03-DV03; observable in every executed boot after W06 lands
  (W10 evidence).
```

W03 owns the line content and order; W06 owns the channel and the marker
format. If W06's accepted design fixes a line-framing rule that conflicts,
the coordination issue is recorded — W03 does not adopt a channel-internal
format, and W06 does not re-render fact content.
