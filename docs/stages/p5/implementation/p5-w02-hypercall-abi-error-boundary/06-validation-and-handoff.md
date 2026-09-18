# P5-W02 Validation, Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W02 detailed design](README.md).

## 1. Preconditions for judging acceptance

Evidence exists only in the verification record; design text never proves
behavior. Rows below distinguish what W02 can evidence alone (host-side,
after P0 gates exist) from what requires upstream or sibling prerequisites
(QEMU, Guest scenarios, integrated dispatch), which is recorded **blocked**
until the prerequisite is evidenced rather than silently skipped or faked.

## 2. Validation matrix

| ID | Requirement → source | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W02-DV01 | Envelope and status constants match the approved design → P5-T01 | constants review | diff implementation constants against [03 §1](03-code-contracts-abi-surface.md) and [04 §1](04-code-contracts-error-boundary.md) | value-for-value equality; single source of truth; no extra Guest-visible vocabulary | the designed surface is what was built; not that a Guest can use it |
| W02-DV02 | Discovery/version/compatibility behave as specified → P5-T01, P5-V02 | host unit tests + QEMU discovery scenario | call matrix: compatible, lower minor, higher minor, cross major, zero word; QEMU Guest reads version (with [P5-W07](../p5-w07-validation-guest-isolation-suite/README.md)) | every matrix cell returns the designed code; discovery result equals the constants from EL1 | version semantics; not public compatibility of any kind (major 0) |
| W02-DV03 | Unknown/unsupported/malformed are distinguishable; decode is total → P5-V02, V06, V13 | host unit + property tests on the pure seam | exhaustive envelope cases; random `u64` register sweeps through decode and classification (fuzz seam for W08) | no panic on any input; every outcome is a §1 status; unknown ≠ unsupported ≠ malformed as designed | parser robustness and class separability; not integrated-dispatch behavior (W06) |
| W02-DV04 | Guest-caused vs invariant separation → P5-T14, P5-V09/V10 basis | type-level review + forced-invariant host test | attempt (compile-time) to convert `InvariantViolation` to a status; stub handler raising an invariant in a host harness | no conversion exists; the forced invariant never composes a status; escalation path is the only exit | structural separation; not the fatal-path runtime behavior (P0-W14 evidence) |
| W02-DV05 | No Host information exposure; no identity authority → P5-V15 basis | security review | walk both value paths out of the boundary (composition, telemetry vocabulary) for Host pointers, table locations, or identity inputs; confirm mapping ignores identity structurally | neither path can carry Host addresses or internal locations; no identity parameter exists in decode/route/mapping | boundary-level non-disclosure; not full-system redaction (W09) or two-context isolation (W07) |
| W02-DV06 | Factual ABI artifact route and compatibility-analysis readiness → P5-V15 (with W09/W10) | documentation review | check the `docs/abi/` route, the publication preconditions, and the skeleton's completeness against [01 §4](01-scope-and-foundations.md) | skeleton names every Guest-visible value and its change policy; no factual artifact published early | route readiness; not that the artifact exists or is correct yet (W10) |
| W02-DV07 | Consumer consumability → W02 closure | consumer walkthrough | read the implementation record as W05 (error classes usable?), W06 (integration contract complete?), W07 (markers assertable?), W08 (seam usable?), W09/W10 (vocabulary + route clear?) | each consumer can act without inventing values this design owns | handoff readiness; not that consumer packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run**, with command, environment, timestamp, and reason. QEMU-dependent rows
are blocked until the P4 execution boundary and W07 assets exist; that block
is evidence state, not failure. No row proves P5-V11–V14, and none may be
reported as doing so.

## 3. ABI change and compatibility route

While the major version is 0: any change to a Guest-visible value (registers,
call numbers, status codes, version word, discovery layout) is a reviewed
design revision plus an implementation-record entry naming the change; the
factual `docs/abi/` artifact, once published, records the compatibility
analysis (what is stable inside the P5 stage, what is explicitly not stable
externally). After any future public freeze (a superseding decision, not this
design): additions via feature bits and new call numbers only; redefinition
of an established supported call or code is a breaking event requiring a new
major version and a superseding design. This route implements the task
book's requirement that additions never silently redefine an established
supported call (P5-V02 wording).

## 4. Error, security, and observability model (summary)

- **Error model:** every request ends in exactly one of: `OK`, one of ten
  rejection classes, or invariant escalation to the fatal path. Rejections
  are side-effect-free (validation before effect), uniform across callers,
  and lossy only at sub-cause granularity (kept internally for telemetry).
- **Security model:** all frame values untrusted; envelope checks narrow
  forgery before any semantics; the guest-visible vocabulary is closed and
  identity-blind; no Host pointer, object-table location, or internal detail
  can cross the boundary through either value path; authority is absent from
  this boundary by construction, so no identity shortcut can be introduced
  here without a type-visible change (review stop).
- **Observability model:** per-request category events with caller-VM
  attribution, call number, outcome class, internal cause, and coarse
  duration class; no Guest data, no Host addresses. Transport, redaction
  policy, and regression integration are
  [P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md); the
  vocabulary is fixed here so W07 markers and W08 fuzz verdicts can cite it.
- **Known limitations carried forward:** timing side channels between status
  classes (recorded, out of scope); no nested/re-entrant call policy (none
  possible in v0); no public compatibility promise at major 0.

## 5. Handoff checklist

Before handing W02 to a reviewer, provide:

- the exact changed-file list, including any new `unsafe` with its SAFETY
  comments and P0-W10 inventory entry (expected only, if at all, in the M1
  trap adapter and M6 result view);
- ABI/public-API statement: which Guest-visible values were implemented,
  whether any differs from this design, and the major-0 change route
  acknowledgment;
- DV01–DV07 evidence paths with run status, including explicit blocked/not-run
  entries (QEMU rows, fatal-path runtime row, host-gate execution if P0
  evidence was absent);
- confirmation that no handle, capability, rights, Guest-data, dispatch
  ordering, telemetry transport, or management-ABI semantics were implemented
  inside W02, and that the `CALL_MINIMAL` handler slot is unreachable from
  Guest input in this package;
- handoff artifacts per consumer: W05 (status classes 7/8 semantics), W06
  (pipeline contract and S6 slot), W07 (assertable Guest-visible values), W08
  (pure seam description), W09/W10 (event vocabulary and ABI route); and
- open items: ADR-056 relationship recorded, timing side channels carried to
  the security deliverable, any `Contract Conflict` findings against P1/P4
  with their recorded status — none resolved inside W02.
