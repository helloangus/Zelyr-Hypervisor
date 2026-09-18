# P5-W03 Validation, Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W03 detailed design](README.md).

## 1. Preconditions for judging acceptance

Evidence lives only in the verification record. Rows distinguish
package-local evidence (host-side, given P0 gates) from evidence requiring
upstream or sibling prerequisites (real Stage-2, QEMU Guest scenarios), which
is recorded **blocked** until prerequisites are evidenced.

## 2. Validation matrix

| ID | Requirement → source | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W03-DV01 | Checked length arithmetic; zero/maximum length → P5-T03, P5-V03 | host unit + property tests | limit boundary, limit+1, `u64::MAX` overflow, base near `u64::MAX`, zero length vacuous pass | construction never wraps; zero succeeds with no access; over-limit → `LengthExceeded` before any query | range-construction correctness; not live Stage-2 behavior |
| W03-DV02 | Required negative and normal cases → P5-V03 | host case suite over the fake space | unmapped (first/middle/last page), RO-write, RW-read, device-type, cross-page, partial first/last page, discontiguous pages, `QueryUnavailable` escalation, protected-range guard | each case yields its designed cause or invariant; Ok tiling is exact; Err ⇒ zero accesses | validator and accessor semantics; not a real VM's Stage-2 |
| W03-DV03 | Forbidden-boundary and no-Host-exposure → P5-V03/V09/V10 | security review + guard test | forced protected-range translation in the fake space; grep/review of value paths for Host pointers into composed values or logs | guard escalates as invariant; no Host address exists in any Guest-visible value or telemetry payload | boundary-level containment; not full-system isolation (W07/W12 evidence) |
| W03-DV04 | Live Stage-2 integration behaves as the fake → P5-V03 (QEMU basis) | QEMU Guest negative scenarios (with [P5-W07](../p5-w07-validation-guest-isolation-suite/README.md)) | Guest issues buffer calls over unmapped/RO/device-ish ranges; compare outcomes to the designed classes | real-Stage-2 causes match the fake-space classes for the same scenario | the port binding is faithful for tested scenarios; not hardware behavior or untested granules |
| W03-DV05 | QEMU ≠ Core rule; Specification-Investigation duty → task book §8 | documentation review | check each architectural claim in the code/docs cites its AArch64 basis; search Core for QEMU-name or QEMU-layout dependencies | no board/QEMU branch; claims carry cited bases | documentation discipline; not the correctness of the cited architecture (locked revision is a later task-book duty) |
| W03-DV06 | Consumer consumability → W03 closure | consumer walkthrough | read the record as W06 (two-phase contract usable per call?), W07 (cause vocabulary sufficient for markers?), W08 (seam + invariants stated?), W09 (cause events redactable?) | each consumer can act without redefining this boundary | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, environment, timestamp, and reason. W03-DV04 is blocked
until the P4 execution boundary and W07 assets exist. No row proves P5-V04
through P5-V08, and none may be reported as doing so.

## 3. Error, security, and observability model

- **Error model:** the boundary has two failure classes — Guest-caused
  (`GuestDataFault` → W02 `GUEST_MEMORY_FAULT`, recoverable, VM-facing, zero
  side effects) and Hypervisor-internal (`QueryUnavailable`, protected-range
  guard, post-validation Host fault → W02 invariant escalation, no Guest
  resumption). Nothing in between exists; every fault carries exactly one
  cause.
- **Security model:** Guest numbers never bind to Host pointer types
  (decision 1); every page is queried — no trusted fast path; the forbidden
  boundary fails closed; device memory is unreachable by policy; all-or-
  nothing means denied requests leave Guest state untouched; identical
  ranges validate identically for all callers (no identity input).
- **Observability model:** per-fault cause events (cause class, Guest-supplied
  base/len, access kind, VM attribution) with no Host addresses and no Guest
  buffer contents; transport and redaction are
  [P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md). Segment
  internals are never logged.
- **Known limitations carried forward:** no cross-CPU concurrent-mutation
  guarantee (decision 8 of [01](01-scope-and-foundations.md)); type checking
  only where the P4 query expresses type (AC-03.1 gap handling); byte-granular
  copy performance (Reserved); limit values are stage facts, not ABI
  promises.

## 4. Handoff checklist

Before handing W03 to a reviewer, provide:

- the exact changed-file list, including any new `unsafe` with SAFETY
  comments and the P0-W10 inventory entry (expected only, if at all, in the
  G2 Arch binding and G4 volatile access);
- the fixed per-call limit values with their recorded rationale, or the
  explicit statement that none were fixed because no consuming call landed
  in this package;
- DV01–DV06 evidence paths with run status, including explicit blocked/not-run
  entries (live-Stage-2 row, host-gate execution if P0 evidence was absent);
- confirmation that no dispatch ordering, handle/capability, envelope, or
  telemetry-transport semantics were implemented inside W03, that no Stage-2
  mutation API was added, and that no QEMU-specific constant entered Core;
- handoff artifacts per consumer: W06 (two-phase contract and per-call limit
  declaration duty), W07 (cause-to-marker vocabulary), W08 (fake-space seam
  and invariants), W09/W10 (fault-cause events); and
- open items: AC-03.1 type-expressibility finding, AC-03.2/P2-ACR-01
  visibility, any `Contract Conflict` against P1/P4 with recorded status —
  none resolved inside W03.
