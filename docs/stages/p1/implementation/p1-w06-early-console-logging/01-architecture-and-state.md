# P1-W06 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W06 detailed design](README.md).

## 1. Logical module map

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Transport | polled byte/line write to the reference console; minimal init | hardware UART state (via init), the console-region constant | byte slices / strings | transmitted bytes | content production, framing policy of producers, discovery |
| Channel phase body | the `console` phase's single entry: init transport, emit start line, set availability | `CHANNEL_AVAILABLE` once-flag | W09 phase call (`console_step`) | availability signal; start line | replay content (W09), marker vocabulary (W09), failure routes (W09 matrix) |
| Marker formatting | render (phase, event) records into the fixed marker line format | none | W09 label/event vocabulary | complete marker lines to the transport | emission points (W09), token selection (W10) |
| Content transport | move producer lines (W03 capability content; W07 report lines) to the transport verbatim | none | complete line strings | transmitted lines | content, order, interpretation |
| Mapping-requirement interface | publish the console MMIO region identity and required class for W08 | none (static facts) | the reference-console assumption | region identity + attribute requirement | the mapping itself (W08), the transition sequence (W08/W09) |

## 2. Message-category model (work seq 2)

The channel carries exactly four categories. Anything else is a scope
failure; the category set is frozen at the producers' needs (growing it is
a design change, the W03 fact-set precedent).

| Category | Prefix (fixed literal, recorded before first verdict-bearing run) | Producer | Content owner | Example shape |
|---|---|---|---|---|
| Channel control | `ZELYR P1 CHANNEL` | W06 phase body (start line, once) | W06 | `ZELYR P1 CHANNEL ready ident=<identity>` |
| Phase marker | `ZELYR P1 PHASE` | W09 `record_marker` emissions (live, and inside the replay) | W09 (labels, event kinds); W06 (framing) | `ZELYR P1 PHASE <label> <enter\|complete>` |
| Capability content | none (line embedded as produced) | W03 `render_report` via its emit callback | W03 (content and order) | `cap <label>=<value> (<classification>)` |
| Report line | producer's class prefix (panic-class / fatal-class markers per W07's report model) | W07 renderer | W07 (fields, ordering) | the W07 report line set |

Rules:

- C1 The category set is closed; a new producer line class is a design
  change here, not a local prefix.
- C2 Prefix literals follow the W10 token precedent: fixed in the
  implementation record before the first verdict-bearing run, never
  adjusted afterwards; a change invalidates prior evidence comparisons and
  is recorded as such.
- C3 The marker format is W06's (W09 §8): `<prefix> <label> <event>` plus
  CRLF. The replay emitted by W09's `materialize_replay` consists of
  exactly these marker lines in recorded order — W06 supplies the framing,
  not the sequence.
- C4 Capability and report lines are transported verbatim with the
  channel's line termination; W06 adds no interpretation and no re-framing
  beyond termination (W09 M4; W03 §7).

## 3. State ownership register

| State | Owner | Written when | Read by | Never written by |
|---|---|---|---|---|
| Console MMIO registers | W06 transport (post-transfer; W01 reporter pre-transfer per its own window) | init; each transmit | — | any other P1 package |
| Reference-console constant | W06 (its own definition; never W01's) | link time | transport, mapping-requirement interface | — |
| `CHANNEL_AVAILABLE` | channel phase body | phase body success, once | W09 replay rule; W07 transport preference; W05/W06 integration checks | everyone else |
| Marker line buffer | marker formatting (stack-local per emission) | each emission | transport | — |
| Producer line buffers | the producers (W03/W07, per their bounded-formatter contracts) | each render | content transport | W06 |

Temporal single-consumer rule on the UART (extending the W02 register's
rule): pre-transfer, the W01 rejection reporter; post-transfer until
channel availability, W02's early writer (panic route only); from channel
availability, the W06 transport (markers, content, reports), with W02's
writer retained exclusively as the panic-route-era fallback transport per
its own contract. Two simultaneous writers are a review failure in
whichever design adds the second (W06-DV04 walks the inventory).

## 4. Channel lifecycle and the MMU transition (work seq 3)

```text
UNAVAILABLE (image entry .. console phase)
  -> console phase body: transport init -> start line (identity) ->
     CHANNEL_AVAILABLE = true          [the availability signal]
  -> materialized regime: replay (W09), then live markers, capability
     lines, report lines
  -> MMU transition (W08 `stage1` phase):
       transport code/data execute from mapped memory (mapping class per
       W08's table: Normal cacheable, same VA under the recorded P1
       identity-window decision);
       console MMIO accessed through the mapped Device window (same VA;
       attribute per the mapping requirement);
       the transition markers (`stage1` enter/complete, `stable`) are W09
       emissions through this channel
  -> STABLE: channel continues to serve reports and markers post-`stable`
     (post-stable fault reports are W05/W07 events rendered here)
```

Continuity obligation split: W06 guarantees its code touches no absolute
state that assumes MMU-off semantics (MMIO access is already volatile
polled I/O; caches-on changes visibility, not correctness, for device
memory — the Device attribute is non-cacheable by definition); W08
guarantees the two windows (transport code/data; console MMIO) are mapped
with the required attributes before `SCTLR_EL2.M` is set, so no instruction
of the channel is ever unmapped. Any change to the console VA is a W08
design change that must record the W06 coordination impact — never a local
constant edit (integration file §3).

## 5. Reference-console assumption and replacement boundary (work seq 4)

- The assumption, stated once and recorded in the implementation record:
  the reference platform presents a PL011 at the canonical QEMU `virt`
  address; P1 performs no discovery and treats the address as a
  reference-platform fact of the canonical boot path (W01 contract §2
  environment). The minimal init sequence (enable transmit; no baud/clock
  programming beyond what the reference platform's firmware state already
  provides) is selected at implementation against the recorded reference
  environment and recorded.
- Layering: the constant and the init/write code live in the reference
  bring-up scope (boot/platform side), selected by the canonical-path
  environment — never by a platform-name branch in generic Core (ADR-043 /
  ADR-044). Generic consumers (W09 markers, W03 content, W07 renderer)
  reference the channel API, never the address.
- Replacement boundary: P2 discovery supersedes the fixed address through
  its own design; the full console subsystem (framework, additional
  transports) is later-stage work. The channel API's consumers must
  therefore depend on the API, not on the transport's placement — the
  Reserved trigger names the boundary, nothing pre-builds it.

## 6. Concurrency model

Boot CPU only; `DAIF` masked; no allocation; no locks. The channel is
callable from boot context (markers, content, phase body) and, after W05's
vectors exist, from the guarded exception path (W07 post-arm reports).
Single-consumer discipline is temporal and guard-based: within any regime
there is exactly one caller (the boot path, or the guarded exception path),
so the transport needs no synchronization primitive; the guard (W05/W07)
serializes the exception case. Relaxed statics suffice for the
availability flag in this model (same justification family as W09's
tracker ordering note); the flag is written once and only read after
publication in every contracted consumer.

## 7. Assumed contracts and failure boundaries

| Seam | Supplied by | Used for | Failure boundary if it delivers differently |
|---|---|---|---|
| `console` phase placement; replay call; emission points | [W09](../p1-w09-initialization-sequencing/README.md) (accepted design) | when the channel comes up; what is printed when | seam mismatch raised per W09 §1; never adapted locally |
| Panic route; `early_write_bytes` fallback; `BuildIdentity` accessor | [W02](../p1-w02-minimal-rust-el2-runtime/README.md) (accepted design) | pre-channel panics; fallback transport; start-line identity | identity unavailable → recorded degradation literal (W02 §2); channel start line renders the literal and continues |
| Capability content lines | [W03](../p1-w03-aarch64-capability-inventory/README.md) (accepted design) | the `console`-phase render wiring | a line-framing conflict with §2 C4 is the recorded W03/W06 coordination issue; W03 keeps content ownership |
| Exception-context callability premise; guard discipline | [W05](../p1-w05-el2-exception-entry-baseline/README.md) (parallel design) | post-arm reports emitted from the exception path | a guard/transport conflict is a W05/W06/W07 coordination issue raised, not absorbed |
| Report lines; class prefixes | [W07](../p1-w07-fatal-crash-diagnostics/README.md) (parallel design) | report transport and the report marker classes W10 matches | prefix vocabulary conflict is a W06/W07 coordination issue; both own their own literals per §2 C2 |
| MMIO mapping class; VA continuity | [W08](../p1-w08-host-stage1-address-space/README.md) (parallel design) | post-MMU channel availability | an attribute or VA change is a W08 design change recording the W06 impact; never a local constant edit |
| Diagnostic-category semantics; identity association | P0-W12 / P0-W16 (planned) | the channel's role boundaries; start-line identity | upstream defect recorded per [workflow](04-implementation-and-review.md) §1; the channel does not invent level or trace semantics |
| Canonical boot environment (the reference console exists) | [W01](../p1-w01-reference-boot-contract/README.md) (accepted design) | the assumption's scope | outside the canonical path the channel is best-effort by design (the W01 R4 posture family) |

Produced for consumers: the channel API (write/line/availability), the
marker format and category vocabulary, the mapping requirement, the token
classes, and the recorded reference-console assumption.
