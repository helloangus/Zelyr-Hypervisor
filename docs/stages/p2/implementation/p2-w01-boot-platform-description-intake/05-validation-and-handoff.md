# P2-W01 Validation, Error Model, and Handoff Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W01 detailed design](README.md).

## 1. Scope of validation for this package

W01's own evidence is host-side: the entire intake stack above the access
boundary is pure logic over byte fixtures (README Decision 8). QEMU
observation of a real firmware DTB passing intake is W09 evidence; offline
re-use of the validators across fixtures is W07 evidence. Planning those here
does not supply them, and no row below may be reported as proving more than
its states.

## 2. Error, security, and observability model

- **Error model.** One fatal outcome class (boot stop) with nine diagnostic
  classes ([01 §3](01-intake-boundary.md)); all-or-nothing publication
  ([02 §3](02-architecture-and-state.md)); no partial state; no retry in P2.
  Guest-caused fault handling does not exist at this stage (no guest), and no
  path here may escalate a malformed *input* into an invariant-violation
  panic — malformed input is a named platform-failure diagnostic
  (P0-W14 classification).
- **Security model.** The DTB is untrusted; enforcement is by construction
  (bounds before reads, checked arithmetic, caps, no allocation, single
  `unsafe` boundary with a `SAFETY` argument, diagnostics that never echo
  blob content). Residual accepted risk: semantically hostile but
  structurally valid blobs pass intake by design; containment responsibility
  sits with W02/W03 and must travel through the W10 handoff record.
- **Observability.** Each diagnostic formats to one boot-log line via the
  P0-W12 channel (class + structured detail: failing field or offset class
  and stage). The anomaly counter is part of the handle so W06 inspection
  and W09 boot logs can surface structural anomalies without W01 printing
  blob data. Success observability: a single "DTB intake validated" marker
  with blob size, version, node/property/reservation counts — no content.

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W01-DV01 → P2-V01 (P2-A01) | Presence/size outcomes | Host unit tests over placement fixtures | Absent, zero-length, undersized, oversized inputs | Each yields `DtbAbsent`/`DtbSizeInvalid`, no reads occur | Placement gate works for those inputs; not firmware behavior |
| W01-DV02 → P2-V01 (P2-A01) | Reachability/alignment | Host unit tests | Misaligned base; span outside fake-window coverage | `DtbMisaligned`/`DtbUnreachable`; no reads | Window-coverage gate logic; not the real P1 mapping |
| W01-DV03 → P2-V01 (P2-A01) | Image overlap | Host unit tests | Overlapping/non-overlapping A3 ranges | Intersect → `DtbImageOverlap`; disjoint → proceed | Overlap arithmetic; not P1's range authority |
| W01-DV04 → P2-V02 (P2-A02) | Header validation | Host unit tests per field | Magic, version, last_comp, alignment, span, mismatch fixtures | Each failure mode yields its class; valid v17 passes | Header gate; not QEMU blob fidelity (W09) |
| W01-DV05 → P2-V02 (P2-A02/A03) | Structure validation | Host unit + generated mutations | Truncation, bad tokens, unbalanced nesting, missing/late `FDT_END`, bad string offsets, bad prop lengths, missing NUL | Only diagnostics; valid trees certify | Token-stream gate; not semantic correctness of any node |
| W01-DV06 → P2-V02 (P2-A02) | Cap enforcement | Host tests with configured caps deliberately lowered | Trees/list beyond caps | `DtbStructureInvalid`/`DtbReservationInvalid` with cap detail | Iteration bounds are real; not production cap values' adequacy |
| W01-DV07 → P2-V02 (P2-A02/A03) | Cursor safety + reservations | Host property-style run over many mutated blobs | Random mutations; cursor accessors exercised on certified results; reservation round-trips incl. zero-address entries | No panic; no out-of-bounds accessor result; verbatim entry preservation | In-bounds-by-construction property; not exhaustive adversarial coverage |
| W01-DV08 → P2-V02 (P2-A04) | Unknown-node handling | Host tests | Foreign nodes/properties in valid trees; anomaly shapes (deep chain, empty nodes) | Silently ignorable or `StructureAnomaly` counters; never fatal | Intake-level ignorance policy; not W02's semantic ignore policy |
| W01-DV09 → P2-V01/P2-V02 | Diagnostic locality and formatting | Host tests + review | Earliest-failure ordering; one-line formatting per class | First independent failure reported; no blob content in output | Diagnosability; not final boot-log rendering |
| W01-DV10 → W01 closure | Consumer walkthrough | Design review | Read contracts as W02 (can I walk nodes?), W07 (can I re-validate offline?), W08 (can I assert diagnostics?), W09 (can I observe boot intake?) | Each consumer acts without new W01 work or raw byte access | Handoff readiness; not consumer implementation |

Evidence statuses are passed / failed / blocked / not run, recorded with
command, input, environment, and timestamp in the verification record.
Host-side validation does not prove QEMU boot behavior, real firmware blob
shapes, or the A2 window contract; those are explicitly W07/W09 evidence and
must appear as not-run entries in this package's record.

## 4. Handoff checklist

Before handing W01 to review, provide:

- the changed-file/module list and the single `unsafe` inventory entry with
  its `SAFETY` argument;
- W01-DV01–DV10 evidence paths and statuses, including not-run entries for
  QEMU, real-firmware DTBs, and the A2 window integration;
- confirmation that no platform-fact types, memory-map types, allocator
  calls, dependencies, board names, or toolchain changes were introduced;
- open items recorded, not resolved: physical module placement pending
  P0-W03 workspace; A1–A3 assumed contracts pending P1 implementation; DTB
  copy/release Reserved;
- named consumer readiness: W02 (cursor + handle + taxonomy),
  W03 (validated DTB range + reservation entries),
  W07 (host-runnable validators), W08 (diagnostic fixtures),
  W09 (boot-marker expectations) — with pointers, not duplicated content.
