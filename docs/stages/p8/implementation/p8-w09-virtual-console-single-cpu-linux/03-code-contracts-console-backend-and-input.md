# P8-W09 Code Contracts — Console Backend, Input Path, and Log Retention

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W09 detailed design](README.md).  
**Modules covered:** M2 (seam), M3 (host backend), M4 (RX path Host side),
M6 (boot-path observer). Frontend contracts are in
[02](02-code-contracts-console-frontend.md).

All names are logical contract names (README decision D8); pseudocode is an
implementation outline, not runnable production code. The seam shape follows
ADR §8's frontend/backend separation; the P8 backend is the EL2-local host
backend (ADR-058 experimental allowance).

## 1. Ownership recap for this file

Everything behind the seam is Host-owned and Guest-invisible: the retained
log, the input queue source, and the drop counters. The Guest can influence
neither capacity nor content of these structures except by producing output
(bounded) and consuming input (draining).

## 2. Backend seam contract

```text
Name and stability: ConsoleBackend (seam) — internal trait/boundary; two
  operations plus lifecycle:
  send(&mut self, bytes: &[u8])          — frontend -> sink (must not block)
  poll_input(&mut self, max: usize) -> ArrayVec<u8, MAX> — sink -> frontend (non-blocking)
  bind(vm)/unbind(vm)                    — lifecycle at VM create/destroy
Purpose and caller: the designed seam (README decision D3) that keeps the
  Guest ABI independent of backend location; P9 (virtio-console) or W19
  (Validation Guest debug reuse) may supply other implementations
Inputs/outputs: byte chunks; `max` bounds each poll to the frontend's RX
  free space (no queue growth from the Host side)
Preconditions:
  - send: device lock held by the frontend caller (serialization)
  - poll_input: called from the frontend/input path only, bounded by `max`
  - bind once per VM lifetime in P8 (unbind/rebind Reserved, [01 §4](01-architecture-and-state.md))
Postconditions:
  - send either records or counted-drops every byte; it never blocks, never
    allocates on the Guest path, and never changes Guest-visible state
  - poll_input returns at most `max` bytes, preserving submission order
State and ownership change: backend-owned buffers only ([01 §3.2])
Concurrency/allocation: implementable without allocation on send (fixed
  buffer + drop policy); Host-context input injection is a Host-side concern
Errors and failure guarantee: a broken backend (unbound mid-flight) is
  detected at call time; frontend behavior degrades to a silent device with
  M5 escalation — never a Host fault, never a Guest hang
Security/authorization checks: the seam carries payload bytes only; no
  control information crosses from backend to Guest register state except
  through the defined RX/IRQ path ([02 §3](02-code-contracts-console-frontend.md))
Logic: interface contract only; P8 implementation in §4
Validation: DV04 seam review; W19/W9 reuse readiness review
```

## 3. RX input path (Host to Guest)

```text
Name and stability: console_inject_input(vm, bytes: &[u8]) -> InjectedCount — internal
Purpose and caller: Host/fixture entry for Guest input (the interactive-shell
  channel); called by the Host console source or the automation harness
Inputs: byte chunk from the Host side (Host-controlled data)
Outputs: count accepted; remainder dropped with counter (bounded queue)
Preconditions:
  - VM exists with console device; backend bound
  - chunk length ≤ the queue's total capacity (enforced by chunking, not by
    growing)
Postconditions:
  - accepted bytes are appended in order to the frontend rx_queue under the
    device lock ([01 §5](01-architecture-and-state.md) Host-side discipline:
    queued to M4, applied at a bounded work point)
  - RX interrupt recompute follows ([02 §3.3](02-code-contracts-console-frontend.md))
  - overflow drops are counted in telemetry; the Host sender is not blocked
State and ownership change: rx_queue only
Concurrency/allocation: Host context; device lock acquisition bounded; no
  allocation (fixed-capacity queue)
Errors and failure guarantee: VM quiesced/destroyed mid-injection -> counted
  drop; never a Host fault
Security/authorization checks: caller is hypervisor/Host-side (not
  Guest-reachable); input bytes are payload only
Logic (pseudocode):
  fn console_inject_input(vm, bytes) -> InjectedCount:
      accepted = 0
      for b in bytes:
          if queue_full(vm.console.rx): { telemetry(rx_drop); break }
          enqueue(vm.console.rx, b); accepted += 1
      console_recompute_irq_locked(vm)      # [02 §3.3]
      return accepted
Validation: P8-V13 interaction; overflow scenario (W18); capacity bound
  review (DV05)
```

The RX queue capacity is a design-owned constant (stage-local design
freedom): fixed, small (one native FIFO depth scale, exact value an
implementation constant within the declared bound), documented in the
implementation record, never Guest-visible. It exists to bound Host work and
Guest-observable latency, not to be tuned as ABI.

## 4. P8 host backend contract

```text
Name and stability: HostMemConsoleBackend — internal; the P8 ConsoleBackend
Purpose and caller: retained-log sink + Host stdout mirror + fixture input
  source behind the §2 seam
Inputs: TX bytes via send(); Host input via console_inject_input
Outputs: log content for evidence ([§5](#5-retained-log-contract)); bytes to
  Host stdout (operational convenience, Host-side policy)
Preconditions: bound to exactly one VM
Postconditions:
  - send appends to a fixed-capacity log buffer; on overflow it drops the
    newest bytes and increments a drop counter (oldest-first retention —
    early markers are the evidence P8-V13 needs)
  - the mirror-to-stdout decision is Host policy (compile-time/run-time
    switch, ADR-048 prunability); it cannot alter Guest-visible behavior
State and ownership change: log buffer + counters (backend-owned)
Concurrency/allocation: small fixed buffer created at bind time (bounded
  allocation at VM creation, none on the TX path); Host-context reads take
  the backend lock and snapshot
Errors and failure guarantee: log-full is a counted, defined condition;
  backend lock poisoning-free design (no lock held across the seam call)
Security/authorization checks: log contains Guest console output (the Guest
  wrote it to its own console); export controls are Host/evidence policy,
  not Guest-reachable
Logic (pseudocode):
  send(bytes):   for b in bytes { if log_full: drop_newest_counter += 1
                                 else: log.push(b) }
                 if mirror_enabled: host_stdout(bytes)   # Host policy
  poll_input(max): backend drains from its input staging (fed by
                   console_inject_input) up to max bytes
Validation: DV05 log retention; DV04 seam; W16 marker extraction
```

## 5. Retained-log contract

```text
Name and stability: ConsoleLogRecord — internal; evidence material
Purpose and caller: the retained boot log P8-V13 requires; read by M6
  (milestone observation), W16 (automated verdicts), and the verification
  record
Contents: the TX byte stream since VM creation, oldest-first, with
  {capacity, dropped_count} recorded; milestone markers are matched in M6,
  never rewritten into the log (the log is a faithful record — W16's
  P8-V22 "stale state" checks depend on that)
Preconditions: backend bound
Postconditions: reads are consistent snapshots; content is immutable once
  written; cleared only at VM destroy
Errors: none (bounded by construction)
Security checks: Guest-authored content; treated as untrusted *data* when
  parsed — marker matching uses exact byte patterns, and a Guest that
  forges markers can only affect its own VM's evidence (noted in [04 §4](04-single-vcpu-boot-path.md))
Validation: P8-V13 log rows; W16 consumption
```

## 6. Boot-path observer contract (M6)

```text
Name and stability: boot_marker_scan(log: &ConsoleLogRecord, script: &InputScript) -> MilestoneReport — internal
Purpose and caller: map retained-log content plus the input script to the
  milestone model of [04](04-single-vcpu-boot-path.md); consumed by the
  verification record and W16
Inputs: log snapshot; the input script used (which markers are *expected*)
Outputs: per-milestone {reached: bool, first_offset: option, timestamp_rel}
Preconditions: log snapshot taken after the scenario completes
Postconditions: pure function over the snapshot; no interpretation beyond
  the declared marker table
Errors: none (missing markers are reported, not errors)
Security checks: Guest-forged markers are indistinguishable from real ones
  in the byte stream; the report is evidence *input*, and the P8-V13
  interaction row (input→output round trip) is the anti-forgery control
Logic: scan for each declared marker pattern; record first offset
Validation: P8-V13; W16 automation consumes the same table
```

## 7. Consumer obligations recorded here

- **W16** builds automated verdicts on the marker table
  ([04 §3](04-single-vcpu-boot-path.md)) and the log's faithful-record rule
  (§5); it must not require markers the table does not declare.
- **W15** supplies the input script content and bootargs (earlycon
  configuration) the milestones assume ([04 §2](04-single-vcpu-boot-path.md)).
- **W19** may implement `ConsoleBackend` for the Validation Guest debug
  channel; the seam (§2) is the only supported reuse point.
- **W17** consumes the telemetry events (tx bytes, rx drops, irq asserts).
- Any consumer needing different RX/TX semantics (flow control, DMA, second
  instance) extends this design first — all are Reserved
  ([01 §3.1](01-architecture-and-state.md), README classification).
