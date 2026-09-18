# P0-W12 Crash Information, Identity Association, and Consumer Constraints

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W12 detailed design](README.md).

## 1. Minimum fatal-information contract

The baseline document must fix the following minimums. They are semantic
requirements on the P1 crash/panic designs (P1-W06 early console logging and
P1-W07 fatal crash diagnostics, per the P1 stage plan index)
subjects), not implementations; the owning design decides representations,
registers, and formats within them.

### 1.1 Panic message minimum

Every panic/fatal-exit message must permit attributing:

- **Failure classification:** which failure class the exit belongs to, per
  [P0-W14](../p0-w14-panic-failure-classification/README.md)'s taxonomy (by
  subject; until that taxonomy is delivered, the classification statement is
  written in W14-agnostic terms and the cross-review binds it later).
- **Site identification:** where the exit was raised — source location or an
  equivalent site identity the owning design defines.
- **Build identity:** a pointer to the identity minimum of §2 — inline where
  the owning design can, otherwise by a stated artifact association.

### 1.2 Crash dump minimum

A crash dump must additionally preserve, as applicable to the site and named
by the owning design:

- the machine state needed to diagnose the exit class (for an EL2 exception
  site, the architectural state the P1-W07 design names; stated here as a
  requirement, never as a register list);
- the identity set of §2;
- enough of the last diagnostic context (channel, level, or trace identities)
  to correlate the exit with preceding behavior, when the logging design
  makes that correlable.

### 1.3 Untrusted-data rule

Guest, device, firmware, or management-domain values must never appear in a
panic message or crash dump as trusted facts. They are labeled by source and
included only as bounded, validated displays; a value that cannot be
validated at dump time is shown as unvalidated. Reason: the dump is consumed
as ground truth for post-mortem work (ADR-007's posture applied to
diagnostics).

## 2. Identity association contract

- **Property:** every diagnostic record must be associable with the build/
  version identity minimum owned by
  [P0-W16](../p0-w16-version-build-metadata-baseline/README.md) (by
  reference; W12 restates no field).
- **Inline versus associable:** fatal/crash output must carry identity
  inline. Other channels must be associable via the artifact or session that
  produced them; the mechanism (embedding, sidecar, session record) is owned
  by the producing design, consistent with W16's deferral of association
  mechanics.
- **Two-way cross-review:** W12's identity sections are not closed until
  cross-reviewed against W16's delivered contract (or recorded as
  blocked-by-prerequisite with the conflict surface named). The same
  obligation exists on W16's side by its design; a mismatch between W12's
  association property and W16's minimum set is a cross-package design
  conflict, raised rather than absorbed.
- **Scope boundary:** W12 requires associability of diagnostics; it does not
  define artifact naming ([W17](../p0-w17-artifact-naming-baseline/README.md)
  subject) or any metadata field.

## 3. Constraints on P1+ diagnostic designs

The baseline document must contain the following binding lists. A P1+ design
that cannot satisfy an item raises the conflict at design review instead of
deviating silently.

### 3.1 Must

- M1: Define the concrete logging/trace/metrics surface (names, macros, or
  items) within the level set and visibility classes of
  [contract 01](01-diagnostic-channels-and-levels.md) §3–§4; the surface's
  namespace follows the consuming design's naming governance.
- M2: Bind the P1 early console as the first transport for the human log and
  fatal channels; the transport is the P1-W06 (early console logging)
  subject.
- M3: Implement runtime filtering for the compiled-out-by-default class
  wherever that class exists in the build.
- M4: Keep channel boundaries: machine-consumed facts to trace/metrics,
  narrative to the log, fatal exits only through the fatal channel.
- M5: Satisfy the fatal minimums of §1 for every fatal exit path the design
  introduces.
- M6: State, per introduced diagnostic, its channel, level, and visibility
  class.

### 3.2 Must not

- N1: Create a parallel logging path (direct console writes as a competing
  channel) after the logging design's adoption point.
- N2: Exit fatally outside [P0-W14](../p0-w14-panic-failure-classification/README.md)'s
  rules — in particular never on guest-caused input.
- N3: Encode machine-parsed fields in the human log, or prose in trace/
  metrics.
- N4: Route around trimming with unclassified switches or call-site
  conditional compilation (violates [W04](../p0-w04-build-profile-feature-governance/README.md)
  governance too).
- N5: Log unvalidated guest-controlled data as trusted fact (violates §1.3).

### 3.3 Transitional early-console rule

Before a logging subsystem exists (P1 start), bring-up output via the early
console is permitted as the boot-time transport, provided it is designed as
the M2 transport binding — not as a competing channel. The P1-W06 design owns
the transition point at which early-console output becomes the logging
surface's transport; from that point N1 applies. This rule gives P1
governance from day one without pre-deciding its logging design.

## 4. Boundaries with W13, W14, and W16

| Subject | Owner | W12's statement |
|---|---|---|
| Trace-event names, domain registry, compatibility | [W13](../p0-w13-trace-event-namespace-baseline/README.md) | channels reference canonical names; W12 defines no name |
| Failure classes, fatal vs recoverable, containment | [W14](../p0-w14-panic-failure-classification/README.md) | fatal channel is reserved for W14's invariant-fatal exits; other classes get VM-scoped or metric treatments |
| Identity fields, formats, timestamp/dirty policy | [W16](../p0-w16-version-build-metadata-baseline/README.md) | association property consumes W16's minimum set by reference |
| Channel semantics, levels, visibility classes, fatal minimums | W12 (this contract) | others reference; none restate |

A conflict between delivered sibling contracts and this one is reconciled in
the same change where possible; otherwise raised as a cross-package design
conflict, and where an ADR-level property is touched, labelled `ADR
Required`.
