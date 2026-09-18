# P5-W04 Validation, Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W04 detailed design](README.md).

## 1. Preconditions for judging acceptance

Evidence lives only in the verification record. Package-local (host-side)
evidence is available once P0 gates exist; Guest-side and real-referent
evidence requires P4 and W07 and is recorded **blocked** until then.

## 2. Validation matrix

| ID | Requirement → source | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W04-DV01 | Existence, stale-after-reuse, repeated destroy, forged values → P5-T05, P5-V04 | host corpora + stress tests | random/zero/max u64; valid→destroy→stale; destroy→reuse→old-handle; generation off-by-one; create/lookup/destroy/recreate loops with invariant harness | no stale or forged handle is ever accepted; every rejection has one typed cause; invariants hold after every operation | table semantics; not that real Guests behave (that is W07) |
| W04-DV02 | Owner destruction cascade → P5-T05 | host cascade tests | destroy Vm with N occupied Vcpu slots; verify all stale in one transaction, ordering, and event emission | vcpu slots first, vm slot last, atomic under lock; subsequent lookups all reject | cascade semantics; not P4's actual stop behavior (AC-04.5 integration pending) |
| W04-DV03 | Opaqueness; no Host-address disclosure → P5-T04, P5-V04 basis | security review | audit handle layout and every value path to Guests/events/logs for Host addresses or referent ids | handle bits derive only from slot/gen/class; no Host pointer or object address in any output | non-disclosure by construction; not an empirical leak-finding claim |
| W04-DV04 | No authority in the identity boundary; uniform rejection → P5-T06, P5-V07 basis | structural review + cause-uniformity tests | confirm no table API carries rights/caller identity; confirm all invalid causes map to the single Guest-visible class | no rights-bearing type exists in the boundary; Guest observes INVALID_HANDLE regardless of cause | identity/authority separation at this boundary; behavioral authority proof arrives with W05/W07 (P5-V06–V08) |
| W04-DV05 | Wrong-type rejection; class extensibility → P5-T06, P5-V05 | host tests + review | cross-class presentation matrix (Vm handle where Vcpu required and vice versa); class-enum extension exercise in a test | every cross-class use rejected before referent access; extension path documented without v0 members | type safety; not future classes' semantics |
| W04-DV06 | Multi-pCPU readiness → P5-V14 basis | concurrency review/test | two-pCPU (or threaded host) exercise: concurrent with_object vs destroy on distinct and same-class handles per the declared shape | no torn states; linearized destroy vs access; matches only the declared guarantee | the designed concurrency boundary; not scalability or a final lock strategy (Reserved) |
| W04-DV07 | Consumer consumability → W04 closure | consumer walkthrough | read the record as W05 (identity + hook usable?), W06 (outcome classes clear?), W07 (markers assertable?), W08 (stress seam + invariants stated?), W09/W10 (event vocabulary + redaction inputs?) | each consumer can act without redefining this boundary | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, environment, timestamp, and reason. Guest-side rows and
real-referent rows are blocked until P4/W07 evidence exists. No row proves
P5-V06–V08 (authority behavior, W05/W06/W07 scope), and none may be reported
as doing so.

## 3. Error, security, and observability model

- **Error model:** four internal invalid causes and one capacity condition;
  Guest-visible outcome uniformly `INVALID_HANDLE` (or `RESOURCE_EXHAUSTED`
  for capacity); invariants escalate through the W02 invariant model; no
  operation leaves partial state.
- **Security model:** opaqueness (no Host structure derivable), single
  lifecycle authority (no contradictory truth), generation protection
  against reuse (retire-on-wrap closing the wrap window), type tags checked
  before referent access, identity without authority (rights absent from
  the types), uniform rejection (no probing oracle), fail-closed invariants.
- **Observability model:** lifecycle events (registered / destroyed /
  vm_destroyed) and rejection causes with class granularity for telemetry;
  no referent identities or handle values in events unless W09's redaction
  policy admits them; the invariant harness is test-only and never runs in
  production paths.
- **Known limitations carried forward:** fixed capacity (documented value
  with rationale; exhaustion is a defined outcome); table-wide lock (contention
  strategy Reserved with W08-evidence trigger); no Guest-visible creation or
  destruction in P5; handle bit patterns unstable at major 0.

## 4. Handoff checklist

Before handing W04 to a reviewer, provide:

- the exact changed-file list, including any new `unsafe` with SAFETY
  comments and P0-W10 inventory entry (expected: none — this boundary is
  safe Rust; any exception needs explicit justification);
- the fixed capacity value and rationale, and the layout constants, as
  implementation facts;
- DV01–DV07 evidence paths with run status, including explicit blocked/not-run
  entries (real-referent integration, two-pCPU execution if unsupported,
  Guest-side rows);
- confirmation that no rights, delegation, Guest-visible create/destroy, or
  VM/vCPU lifecycle semantics were implemented inside W04, and that no table
  API leaks references beyond closures;
- handoff artifacts per consumer: W05 (identity lookup semantics + destroyed
  hook), W06 (outcome conversion and closure discipline), W07 (observable
  reference behaviors), W08 (stress seam, invariants, two-pCPU shape),
  W09/W10 (event vocabulary); and
- open items: AC-04.1 integration status, AC-04.5 hook ownership,
  `Contract Conflict` items with the W02 sibling if any — none resolved
  inside W04.
