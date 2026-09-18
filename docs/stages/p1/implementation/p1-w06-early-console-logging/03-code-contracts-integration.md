# P1-W06 Integration Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W06 detailed design](README.md).

These contracts bind the channel to its producers, consumers, and the MMU
transition. Each states what the other side must provide and what W06
guarantees in return; a conflict is a recorded coordination issue, never a
local adaptation.

## 1. W05 — exception-context callability

```text
Guarantee (W06 -> W05/W07): from channel availability onward,
  console_write_line is callable from the guarded exception path with all
  masks set: no lock is taken, no DAIF state is read or changed, no
  allocation occurs, and the only shared state touched is the UART itself
  (single-consumer discipline held by the W05/W07 guard, not by channel
  internals).
Obligation accepted (W06 <- W05/W07): the exception path enters the
  channel at most once per fatal event (the guard is held for the whole
  route), so no second consumer can interleave.
Failure boundary: if a W07 design change ever needs concurrent channel
  use from two contexts, that is a P3-class concurrency design change,
  not a channel patch — the synchronization model of the architecture
  file §6 is revisited, not extended ad hoc.
Validation: W06-DV03 (contract review); NC3/NC6/NC4 executions observe
  the property (W11, deferred).
```

## 2. W03 and W07 — producer obligations

```text
Guarantee (W06 -> producers): lines are transported verbatim in call
  order, terminated with CRLF, with no re-framing, filtering, or
  interpretation (W09 M4; parent README decision 5).
Obligation (producers -> W06): each line is complete content owned by the
  producer (W03: `cap <label>=<value> (<classification>)` per its §7;
  W07: its report lines with its class prefixes), bounded to the channel
  maximum (over-long lines truncate with a recorded truncation marker),
  and rendered through the bounded formatter discipline so the call is
  allocation-free.
Emission points (fixed here to close W09's wiring):
  capability content — once per boot, during the `console` phase wiring,
    after the replay (W09 orders replay before `phase_complete(Console)`;
    W03's §7 caller is that wiring);
  report lines — only from the fatal path (W07's renderer), i.e. from
    panic/exception events, never from the normal boot path; a normal
    boot's capture therefore contains no report-class lines (the W10
    forbidden-class property).
Failure boundary: a producer that needs ordering guarantees beyond call
  order, or a second emission point, is a design change in that producer's
  package plus a recorded note here — never a silent addition.
Validation: W06-DV03; W10's R1/R2 verdict classes observe the property
  (deferred execution).
```

## 3. W08 — MMU-transition continuity (work seq 3)

```text
Guarantee (W06 -> W08/W09): the channel holds no MMU-off-era state that
  breaks under the recorded mapping: MMIO access is volatile polled I/O
  through the Device window; code/data are position-local; the start-line
  and marker buffers are stack-local per call. The channel emits nothing
  of its own during the transition — transition markers are W09
  emissions, so W08's obligation is purely the mapping of two windows.
Obligation (W08 -> W06): before `SCTLR_EL2.M` is set, both windows are
  mapped with the recorded attributes — (a) the transport's code/static
  data with the normal code/data classes of W08's table, (b)
  CONSOLE_REGION with CONSOLE_REQUIRED_ATTRIBUTES (Device, XN, RW) — and
  the console VA is unchanged by the transition (the recorded P1
  identity-window decision). Post-MMU channel liveness is demonstrated by
  the post-transition markers themselves (W09 `stage1` emissions),
  observed by W10's R1 and W11's NC5 runs.
Failure boundary: a W08 change to the console VA or attributes is a W08
  design change that records the W06 coordination impact (the constant is
  W06-owned; it is never edited to absorb a mapping change). If the
  channel cannot emit after the transition, the W09 matrix `stage1` row
  routes the failure — the fatal report falls back to W02's early writer,
  whose constant is independent of this channel by the W01 §6 rule.
Validation: W06-DV03; P1-V14's executed half belongs to W10/W11 (deferred).
```

## 4. W09 and W10 — markers, replay, and verdict classes

```text
Guarantee (W06 -> W09): format_marker renders every (phase, event) record
  in the fixed line format §5 of the channel contracts, callable from the
  tracker's record path, allocation-free, and truncation-safe; the replay
  is a sequence of such lines and nothing else.
Guarantee (W06 -> W10): the marker class prefix is a fixed, distinctive
  literal (recorded before the first verdict-bearing run; never adjusted
  afterwards — the token precedent), so boot-start markers are matchable
  content classes independent of line order; W06 adds no variable content
  to the marker lines themselves.
Obligation (W09): emission points stay exactly where its state machine
  fixed them (record on every transition; replay once at the availability
  signal; stable marker at `stable` entry with W10's token as its
  content). W06 owns neither points nor token.
Failure boundary: a marker-format or token-collision finding from W10's
  review is a recorded vocabulary change under §2 C2 — with the evidence
  invalidation it carries — never a silent rewording.
Validation: W06-DV02/DV06; W10-DV02 reads the same vocabulary from its
  side.
```

## 5. Replacement boundary and prohibited content (work seq 4)

- Replacement boundary (recorded, not built): P2 discovery supersedes the
  fixed reference-console constant and the W01/W02 boot-path constants
  through its own design; the full console subsystem (framework, additional
  transports, level filtering) is later-stage work. Consumers depend on the
  channel API, never on the transport's placement or the constant — this is
  the property that makes the replacement bounded.
- Prohibited content (review explicitness): no device probing or DTB
  reads; no baud/clock programming beyond the recorded minimal init; no
  interrupt enable (the UART stays polled; unmasking is a W09-H3
  violation); no ring buffer, DMA, or deferred flushing; no log-level
  filtering; no second console instance; no absolute code addresses in
  output (the channel never prints addresses it derives from its own
  linkage — reported addresses come from producers' frames/reports); any
  output path beyond the recorded four-way set (W01 reporter, W02 writer,
  W06 channel, W07 renderer) is a scope failure. W06-DV04 walks this list.
