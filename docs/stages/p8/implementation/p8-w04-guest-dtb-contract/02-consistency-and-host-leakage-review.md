# P8-W04 Consistency and Host-Leakage Review Criteria

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W04 detailed design](README.md).

## 1. Purpose

This file fixes the two review disciplines the plan assigns to W04: the
positive consistency review (DTB facts versus machine and boot contracts —
P8-V05) and the negative host-leakage review (P8-V06). Both are designed to
run at two points: against the *contract* during W04 closure, and later
against *generated DTBs* by the consumers ([P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md)/[W10](../p8-w10-linux-smp-bringup/README.md)
at implementation, [P8-W14](../p8-w14-machine-abi-compatibility/README.md)/[W16](../p8-w16-automated-linux-regression/README.md)
as automated drift/leak checks). The criteria are therefore stated as
checkable properties, not one-off instructions.

## 2. Consistency review (P8-V05)

A DTB (or the DTB contract evaluated against current machine facts) is
consistent when every property below holds. Each check names the facts it
draws from.

| # | Consistency property | Sources |
|---|---|---|
| C-1 | The root identity (D1) matches the bound machine identity; no second machine name appears | W02 C1; [P8-W02](../p8-w02-machine-contract-governance/README.md) §3 |
| C-2 | CPU nodes (D2) describe exactly the configured vCPU set; the enable-method equals the PSCI route the firmware category defines; topology expression, once routed and approved, is presented exactly as approved | VM boot configuration; W02 C2/C6; boot contract B6-2; [P8-W06](../p8-w06-psci-virtualization/README.md) |
| C-3 | Memory nodes (D3) describe exactly the Guest RAM the Stage-2 address space maps — set equality, both directions | W02 C3; boot contract B7-1; P4 Stage-2 facts (assumed contract) |
| C-4 | `/chosen` (D4) carries exactly the configured boot state: bootargs per B1-3, initrd properties iff the boot configuration includes an initramfs (B1-2), `stdout-path` resolving to the presented console node | Boot contract B1/B4; [P8-W15](../p8-w15-reproducible-linux-fixture/README.md) fixture values |
| C-5 | The PSCI node (D5) promises exactly the implemented and required function set — no superset, no subset | W02 C6; [P8-W06](../p8-w06-psci-virtualization/README.md) |
| C-6 | The timer node (D6) presents the virtual-timer access the timer category defines, and the interrupt specifiers it names are inside the machine's interrupt-category assignment | W02 C5/C4; [P8-W08](../p8-w08-linux-timer-integration/README.md) |
| C-7 | The GIC node (D7) presents the approved GICv3-class surface with register frames and maintenance interrupt inside the machine categories; no ITS/LPI node exists in v1 | W02 C4; [P8-W07](../p8-w07-linux-vgicv3/README.md); task book out-of-scope list |
| C-8 | The console node (D8) matches the approved console category and the device the console contract implements; `stdout-path` resolves to it | W02 C7; [P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md) |
| C-9 | Reserved-memory nodes (D9) match the machine's Guest-reserved categories exactly | W02 C3/C9; boot contract B7-1 |
| C-10 | The device set (D10) contains nothing beyond the P8 minimal set; no Virtio node exists, and reservation statements obey the W02 reservation prohibition | Task book out-of-scope list; W02 C9 |
| C-11 | Every presented fact traces to exactly one authoritative source per the contract's source-authority rule (machine category, boot-contract fact, consumer contract, published binding, or resolved routed value) | [01-dtb-fact-catalog.md](01-dtb-fact-catalog.md) source-authority rule |

Before values are frozen (the current state), C-2–C-9 are evaluated against
the *categories* and *routes*: the review passes when each fact's source is
correctly identified and no fact claims a value whose route is unresolved.
After values are frozen, the same checks evaluate concrete equality.

## 3. Host-leakage review (P8-V06)

A DTB (or contract statement) leaks when any property below fails. Every
check is defined so a reviewer (or later, an automated checker) can evaluate
it node by node.

| # | Prohibition | Check |
|---|---|---|
| L-1 | No host physical address | Every address-like value (reg, ranges, initrd location, DTB placement reference) is a Guest-physical (IPA) value from the machine categories or boot configuration; no value equals or encodes a host address fact |
| L-2 | No host IRQ | Every interrupt specifier resolves to a Guest-physical interrupt identifier of the machine's interrupt category; no physical GIC line number, host SPI routing, or host wired-IRQ fact appears |
| L-3 | No board/SoC identity | No node property names QEMU, RK3566/Rockchip, a board vendor, or any host component; compatible strings are limited to D1 and the published bindings' required strings |
| L-4 | No host firmware disclosure | No node exposes host firmware interfaces, host TF-A/U-Boot artifacts, host EL3 services, or host DTB nodes/properties (Host DTB reuse is prohibited outright) |
| L-5 | No host topology/performance fact | No host CPU count beyond the configured vCPUs, host frequency, cache geometry of the host, host erratum data, or host memory-map remainder appears |
| L-6 | No host-reserved disclosure | Reserved-memory nodes contain only machine-defined Guest reservations (D9-2) |
| L-7 | No accidental channel | Comments, chosen/bootargs text, model strings, and property names are reviewed as content too — a host fact in a comment or bootargs string is still a leak |

The leakage rule's authority basis is recorded per item in the DTB contract:
ADR-024 (host independence), ADR-043/052 (no board-name paths), ADR-007 and
the W01 guest-untrusted constraint (the Guest must not learn host layout),
and the task book's P8-V06 wording.

## 4. Observation form

Both reviews must be phrased so their later automated form is determined
now: a check is an equality/set property over enumerated DTB facts with a
named source of truth (machine category, boot fact, fixture value, or
consumer contract). Prose-only criteria ("looks host-free") are rejected in
review. The [P8-W14](../p8-w14-machine-abi-compatibility/README.md)
compatibility policy and the [P8-W16](../p8-w16-automated-linux-regression/README.md)
matrix consume these checks as drift and leak detectors; this file defines
them, does not implement them.

## 5. Non-responsibilities

This file does not define DTB content (fact catalog), build the checker
(Reserved for W14/W16 scope), generate any DTB, or decide any routed value.
Where a consistency check cannot be stated without a value, it is stated
against the value's category, and the value stays routed.
