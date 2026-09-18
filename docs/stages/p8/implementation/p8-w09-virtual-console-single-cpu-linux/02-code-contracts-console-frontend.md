# P8-W09 Code Contracts — PL011 Console Frontend

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W09 detailed design](README.md).  
**Modules covered:** M1 (frontend), M5 (containment). Backend/input and log
contracts are in [03](03-code-contracts-console-backend-and-input.md); the
boot path in [04](04-single-vcpu-boot-path.md).

All names are logical contract names (README decision D8); pseudocode is an
implementation outline, not runnable production code. Register semantics are
the ARM PL011 subset declared in README decision D1 (PL011 TRM as the
specification source); MMIO base and INTID are machine-gated
([01 §3.1](01-architecture-and-state.md)).

## 1. Access dispatch contract

```text
Name and stability: console_mmio_access(vcpu, ipa, size, is_write, value) -> AccessOutcome — internal
Purpose and caller: sole entry from the Stage-2 MMIO fault path for the
  console window; routes to the register handlers
Inputs (untrusted): ipa, size (1/2/4 bytes), is_write, value
Outputs: handled (with readback for reads) | Unmapped (caller applies the
  W05-classified outcome)
Preconditions: vcpu Running in Guest context; VM has console machine facts
Postconditions: bounded work per access; exactly one outcome; one telemetry
  event per access class; state mutation only via the register handlers
State and ownership change: per-VM frontend state only ([01 §3.1])
Concurrency/allocation: VM device lock; no allocation; no polling
Errors and failure guarantee: out-of-window -> Unmapped; bad width ->
  defined per §5; never a Host fault, never partial state
Security/authorization checks: window membership bounds-check; the window is
  per-VM — no cross-VM reach exists in the map
Logic (pseudocode):
  fn console_mmio_access(vcpu, ipa, size, is_write, value):
      if !in_window(console_window, ipa): return Unmapped
      off = ipa - console_window.base
      lock(vcpu.vm.console_lock):
          match decode_pl011(off):
              DR        => if is_write { tx_byte(vcpu, value as u8) }
                           else        { rx_pop(vcpu) }
              FR        => readback(fr_flags(vcpu))       # §3
              RSR_ECR   => if is_write { clear_errors(vcpu) } else { readback(0) }
              Writable(c)  => if is_write { store(c, masked(value)) }
                              else            { readback(image(c)) }
              ReadOnly(c)  => if is_write { telemetry(ignored_write) }
                              else            { readback(const(c)) }
              PID/CID      => readback(pl011_id_bytes(off))   # §4, constants
              _            => RAZ_WI()
      telemetry(access_class)
Validation: dispatch review; P8-V12 containment rows; W18 illegal-MMIO set
```

## 2. Register subset (the frozen v1 Guest ABI for the console)

Implemented per the PL011 TRM; offsets relative to `<CONSOLE-MMIO-BASE>`:

| Offset | Register | Semantics owned here |
|---|---|---|
| 0x000 | DR | Write: TX byte (transmit path §3.1; received-characters error bits in the write value are masked/ignored in the P8 subset). Read: pops one RX byte (§3.2); read of empty RX returns 0 with FR.RXFE still reporting empty |
| 0x004 | RSR/ECR | Read RAZ; write clears error latches (no-op state, kept for driver compatibility) |
| 0x018 | FR | Generated view (§3): TXFE=1, RXFE, RXFF per RX state, BUSY=0, DCD/DSR/CTS read inactive |
| 0x024/0x028 | IBRD/FBRD | Stored, masked to field width; no functional effect (virtual line has no timing) |
| 0x02C | LCR_H | Stored (FEN/width/parity bits); no functional effect beyond readback in the P8 subset |
| 0x030 | CR | Stored (UARTEN/TXE/RXE bits honored for enable-gating of TX/RX behavior); modem/loopback bits stored, loopback is not implemented (RTS/CTS handshake unused; recorded simplification) |
| 0x034 | IFLS | Stored; no functional effect (no real FIFO thresholds) |
| 0x038 | IMSC | RX/TX interrupt mask, honored exactly (§3.3) |
| 0x03C/0x040 | RIS/MIS | Generated from RX/TX state (§3.3) |
| 0x044 | ICR | Write clears interrupt state per field |
| 0xFE0–0xFFC | PrimeCellID/PeripheralID | Standard PL011 identification constants (§4) |
| Any other offset | — | RAZ/WI (includes the optional DMA registers — README Reserved list) |

Enable-gating rule: TX writes with UARTEN=0 or TXE=0 are accepted and
dropped (byte not forwarded); RX interrupts fire only with UARTEN=1 and
RXE=1 and IMSC.RXIM=1 — Linux's driver sequence (enable → program →
unmask) therefore behaves as the real device would.

## 3. Behavior contracts

### 3.1 TX path

```text
Name and stability: console_tx_byte(vcpu, byte: u8) -> () — internal
Purpose and caller: DR writes; the entire Guest→Host output path
Inputs: one byte (payload)
Outputs: seam::send([byte]) ([03 §2](03-code-contracts-console-backend-and-input.md))
Preconditions: UARTEN/TXE gating per §2; device lock held
Postconditions:
  - byte accepted into the backend sink or counted as dropped per the
    backend's bounded policy ([03 §4](03-code-contracts-console-backend-and-input.md));
    the Guest-visible consequence is identical either way (FR.TXFE stays 1 —
    decision D4), so the Guest can never hang on output
  - the byte is data only: no byte value affects control flow
State/ownership change: backend buffer contents (behind the seam)
Concurrency/allocation: bounded; the seam call must not block (policy in [03 §4]);
  no allocation on the Guest path
Errors and failure guarantee: backend failure (bound backend gone) -> M5
  internal-fault escalation; Guest sees a silent device, not a hang
Security/authorization checks: none beyond gating — output is Guest-controlled
  data going to the Guest's own console record
Logic: if tx_enabled(vcpu): seam.send(&[byte]) else telemetry(tx_gated)
Validation: P8-V12/V13 output rows; flood scenario (W18)
```

### 3.2 RX path (Guest side)

```text
Name and stability: console_rx_pop(vcpu) -> u8 — internal
Purpose and caller: DR reads; the Guest's only input channel
Inputs: none
Outputs: oldest queued RX byte, or 0 if empty (FR.RXFE reports truth)
Preconditions: device lock held
Postconditions:
  - a pop that empties the queue deasserts the RX interrupt source (via the
    MIS recompute of §3.3), keeping edge-level semantics consistent
  - reads never block and never reorder (FIFO order)
State/ownership change: rx_queue ([01 §3.1])
Concurrency/allocation: bounded; no allocation
Errors and failure guarantee: empty read is defined (0 + RXFE), not an error
Security/authorization checks: input content is Host-controlled ([01 §6]);
  the Guest cannot enqueue
Logic: if rx_empty(vcpu): return 0 else { b = pop(rx_queue); recompute_irq(vcpu); b }
Validation: P8-V13 interaction row; RX/IRQ consistency rows
```

### 3.3 Interrupt-state contract

```text
Name and stability: console_recompute_irq(vcpu) -> () — internal
Purpose and caller: recompute RIS/MIS and the injection decision after any
  state change (RX push/pop, IMSC/ICR writes, CR enable changes)
Inputs: frontend state
Outputs: RX interrupt asserted (level) iff {RX pending} ∧ {IMSC.RXIM} ∧
  {UARTEN ∧ RXE}; TX interrupts never assert (TX is always ready — §2)
Preconditions: device lock held
Postconditions:
  - assertion changes are delivered through [W07 §3](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md)
    `vgic_request_irq` on <CONSOLE-SPI-INTID> with level config; rising edge
    injects, falling edge is the Guest's mask/ICR/drain consequence
  - no storm: the assertion condition is level and monotone with queue
    content; a queue that stays non-empty asserts once (level), not per byte
State and ownership change: RIS/MIS image; W07-side pending via its bridge
Concurrency/allocation: bounded; the W07 call issues, never blocks
Errors and failure guarantee: W7 rejection/deferral is W07's declared
  outcome; the frontend state stays correct regardless
Security/authorization checks: INTID is machine-gated; target is this VM
Logic (pseudocode):
  ris_rx = !rx_empty(vcpu)
  mis_rx = ris_rx and imsc.RXIM and cr.UARTEN and cr.RXE
  if mis_rx and !last_mis_rx:
      vgic_request_irq(VmTarget(vcpu.vm), CONSOLE_SPI_INTID, LevelAsserted,
                       Source::ConsoleRx)
  last_mis_rx = mis_rx
Validation: RX IRQ rows of [06](06-validation-and-handoff.md); W18 storm rows
```

## 4. Device identity contract

```text
Name and stability: PL011_ID — read-only constant table; internal
Purpose and caller: PrimeCellID/PeripheralID reads; Linux's AMBA bus probes
  these values from the device itself to bind amba-pl011 (README decision D2)
Contents: the standard PL011 peripheral identifier and PrimeCell CID values
  from the PL011 TRM (constants of the device spec, registered with the W02
  gate alongside the machine placement)
Preconditions: registered with the machine facts (P8-V05 consistency with
  the DTB compatible string, owned by W04)
Postconditions: pure constants; identical for every VM
Errors: none
Security checks: never read from Host hardware
Validation: DV02 probe; W14 compat dimensions (ID values are ABI facts)
```

## 5. Containment table (W05 classification applied)

| Guest behavior | Class | Outcome |
|---|---|---|
| In-window, implemented register, legal width | Direct | Emulated per §2–§3 |
| In-window, unimplemented offset (incl. DMA regs) | Direct (RAZ/WI) | Read 0 / write ignored + telemetry |
| Out-of-window or width-violating access | Reject | W05-classified fault; W13 diagnostic slot gets {vCPU, IPA, class} ([01 §6](01-architecture-and-state.md)) |
| TX flood (sustained DR writes) | Direct + bounded | Backend drop policy + counter ([03 §4](03-code-contracts-console-backend-and-input.md)); Guest never hangs; Host unaffected beyond bounded work |
| Gated-path abuse (CR toggling at rate) | Direct + bounded | Per-access bounded work; telemetry shows rate; no state corruption (idempotent stores) |
| Read-only register writes | Direct (ignored) | Telemetry; state unchanged |

The P8-V12 "Host serial ownership" requirement is satisfied by construction:
no Guest path reaches Host serial hardware; Host-side ownership decisions
([03 §5](03-code-contracts-console-backend-and-input.md)) are Host-owned.
