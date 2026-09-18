# P8-W09 Single-vCPU Boot Path Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W09 detailed design](README.md).

## 1. Scope and authority of this file

This file binds the sibling contracts into the ordered one-vCPU Linux path
that P8-V13 judges. It owns only the integration order, the milestone/marker
model, and the interaction criterion; each milestone's mechanism is owned by
the cited package. The Linux fixture itself (source, config, initramfs,
bootargs content) is W15/W03 material — this file fixes the expectations the
fixture must satisfy, not its content.

## 2. Boot inputs assumed (from W03/W04/W15)

| Input | Owner | W09 consumption |
|---|---|---|
| Linux Image, optional initramfs, loaded Guest RAM | W03 contract (placement per machine facts) | preconditions of M0 |
| Guest DTB with CPU (1 node), memory, chosen/bootargs, PSCI, timer, GIC, console nodes | W04 contract; W02-gated values | earlycon + driver probes; consistency reviewed at P8-V05 |
| bootargs including `earlycon=pl011,mmio32,<CONSOLE-MMIO-BASE>` and console= specification | W03/W15 | M2's earlycon output depends on it; the exact spelling is fixture material within this expectation |
| boot vCPU entry state | W03 contract ([W06](../p8-w06-psci-virtualization/README.md) consumes it for CPU_ON too) | M1 |
| PSCI `arm,psci` node, `method = "hvc"` | W04; frozen via W02 ([W06](../p8-w06-psci-virtualization/README.md)) | shutdown behaviors at M8 are out of this file's bar (they are W06's S4) |

## 3. Milestone model (M0–M7) and markers

Milestones are cumulative; each requires all prior. Markers are exact byte
patterns in the retained log ([03 §5](03-code-contracts-console-backend-and-input.md));
their concrete strings are fixture material (W15) chosen so that each is
produced only after the named mechanism succeeded — this file fixes the
*requirements* each marker must evidence.

| ID | Milestone | Mechanism owner | Marker must evidence | Failure meaning |
|---|---|---|---|---|
| M0 | VM loaded and created | W03/P4 foundations | — (pre-boot; verified by creation success, not log) | boot inputs/contract broken |
| M1 | boot vCPU entered Guest EL1 | P4/W03 entry | first console byte written by Guest code (any output) | entry or MMIO path broken |
| M2 | earlycon alive | W09 frontend + bootargs | earlycon banner text emitted via DR writes | frontend DR/FR subset broken |
| M3 | kernel GICv3 initialized | [W07](../p8-w07-linux-vgicv3/README.md) | Linux GIC probe/init completion text (TYPER/RG enumeration on 1 CPU) | vGIC register subset broken |
| M4 | clock/events alive | [W08](../p8-w08-linux-timer-integration/README.md) | clocksource/clockevent registration text | timer contracts broken |
| M5 | console driver up, IRQ-driven | W09 + W07 | amba-pl011 probe/bind text (proves ID regs + DTB + SPI path) | RX/IRQ or ID contract broken |
| M6 | initramfs userspace started | W03 (initramfs), W08 (time) | init/shell start text | userspace handoff broken |
| M7 | **interactive shell** | W09 RX path + W07 SPI | input-script command echoed back and its output present in the log | input path broken — the round trip is the anti-forgery control (§5) |

M7 is the P8-V13 completion criterion: a shell prompt marker alone is
necessary but not sufficient — the log must show a Host-injected command
line *and* the Guest's response to it. `start_kernel`-reaching boots satisfy
at most M1–M4 and explicitly do not close the package (plan work sequence
5).

## 4. Boot-log and evidence rules

- The retained log is the only in-band evidence surface; it is a faithful
  record ([03 §5](03-code-contracts-console-backend-and-input.md)) — no
  post-processing, no marker insertion by the Hypervisor.
- Guest-forged markers: because the log contains Guest-authored bytes, a
  Guest can print text resembling any marker. Two controls keep the evidence
  honest: (1) M7's round trip requires output that functionally depends on
  Host-injected input; (2) W16's automation combines the log with
  Hypervisor-side telemetry events (VM-exit class distribution, IRQ
  injections) that a Guest cannot fabricate. Marker text alone is therefore
  evidence-input, never evidence-complete.
- The milestone report ([03 §6](03-code-contracts-console-backend-and-input.md))
  records first-offset and relative time per milestone; boot-time numbers
  derived from it are W17 baseline observations, not KPIs.

## 5. Single-vCPU path stage gates (the integration order W09 implements)

```text
Stage A  device-only:     frontend+backend exercised by host-side tests and a
                          minimal boot (M1–M2) — no GIC/timer dependency
Stage B  + interrupts:    A + W07 (M3, M5): probe, RX IRQ round trip without
                          userspace (scripted input at kernel stage)
Stage C  + time:          B + W08 (M4): registration and a timed sleep
Stage D  full baseline:   C + initramfs (M6–M7): interactive shell round trip
```

Each stage has a passing condition in
[06 §1](06-validation-and-handoff.md); a stage that fails blocks the next —
this ordering localizes failures to the newest dependency, which is the
point of planning the 1-vCPU path before SMP ([W10](../p8-w10-linux-smp-bringup/README.md)
builds on a green Stage D).

## 6. Clean-shutdown observation (out of the P8-V13 bar, in scope for evidence)

After M7, a `poweroff` in the shell exercises [W06](../p8-w06-psci-virtualization/README.md)
SYSTEM_OFF (its S4 row); the expected observable is Guest execution ceasing
with the VM reaching its stopped lifecycle state and the pCPU returning to
Host control ([W06 §5](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md)).
This is recorded as supporting evidence for W06's matrix, not as a W09
validation row.
