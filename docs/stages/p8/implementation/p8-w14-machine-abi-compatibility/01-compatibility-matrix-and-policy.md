# P8-W14 Compatibility Matrix and Policy

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W14 detailed design](README.md).

## 1. Authoritative artifact and source facts

W14 authors exactly one normative artifact: the **machine compatibility matrix
and policy document**, created at implementation time. Its home is
`docs/abi/` per the documentation index ("versioned ABI and wire-format
contracts"); the exact file name is recorded in the implementation record when
created (suggested: `docs/abi/rusthv-arm-virt-v1-compatibility.md`). It is the
sole authoritative home of the matrix, change classes, escalation rules, test
plan, and firewall rules defined here. Other documents may link to it but must
not restate values.

Every fact value cited in the matrix comes from one authoritative source:

| Source | Owning plan | Facts contributed |
|---|---|---|
| Machine-contract governance record | P8-W02 (`../p8-w02-machine-contract-governance/README.md`) | machine identity/version, address categories, reservation policy, compatibility decision route |
| Guest DTB contract record | P8-W04 (`../p8-w04-guest-dtb-contract/README.md`) | DT compatible/model, CPU/topology nodes, memory node, chosen/bootargs/initrd facts, PSCI node, timer node, GIC node, console node, reserved-memory |
| PSCI contract record | P8-W06 (`../p8-w06-psci-virtualization/README.md`) | PSCI version, mandated functions, calling behavior, system-off route |
| vGIC contract record | P8-W07 (`../p8-w07-linux-vgicv3/README.md`) | GIC model presented, interrupt assignments (PPI/SPI), maintenance behavior |
| Timer contract record | P8-W08 (`../p8-w08-linux-timer-integration/README.md`) | counter/timer virtual presentation, frequency semantics, wakeup behavior |
| Console contract record | P8-W09 (`../p8-w09-virtual-console-single-cpu-linux/README.md`) | console device presentation, register-window location category, containment semantics |
| CPU compatibility record | P8-W05 (`../p8-w05-linux-cpu-virtualization/README.md`) | Guest EL, system-register presentation classes, feature classification |

A row whose source record is unapproved carries an **empty value cell and
status `blocked`**; it never carries a provisional value. All rows are blocked
today; approval flips them by citation.

## 2. Matrix entry schema

Per the Coding Guidelines ABI rule (explicit representation, width,
endianness, padding, versioning, compatibility behavior), every matrix entry
states:

```text
fact_id          stable identifier (e.g., MAP-RAM-WINDOW, DT-COMPATIBLE)
category         one of §3's categories
description      what the fact is, in Guest-visible terms
representation   how the fact is expressed where applicable (width, count,
                 units) — cited from the source record, not redefined here
value            the approved v1 value; EMPTY + blocked until approved
source           the owning record and section (citation, not transcription)
comparison       how drift is detected: value-equality | semantic-assertion |
                 presence/absence | version-equality
owner            the package accountable for the fact's definition
```

`comparison` semantics: **value-equality** for discrete values (addresses,
counts, IDs); **semantic-assertion** for behaviors (PSCI function works per
contract; timer wakeup semantics hold); **presence/absence** for reserved or
forbidden entries (a reserved window stays reserved; a Host fact stays absent);
**version-equality** for identity facts.

## 3. Guest-visible fact enumeration (matrix content)

The matrix must contain at least the following entries. The enumeration is
required content; each entry's value waits for its source record.

| Category | Required entries (fact groups) | Source |
|---|---|---|
| Machine identity | machine name; machine version identity (ADR-040) | W02 |
| Guest IPA map | RAM window; GIC window; UART/console window; reserved windows (incl. virtio-mmio and PCI reservations as declared by the approved contract); any reserved-memory region | W02/W04 |
| Device location | console device location and type class; declared minimal devices | W02/W09 |
| Interrupt assignment | timer PPI; console IRQ; GIC maintenance (as declared); SGI usage; declared SPI set | W07/W08 |
| DTB facts | DT compatible/model strings; CPU/topology node semantics; memory node; chosen/bootargs/initrd presence and semantics; PSCI node; timer node; GIC node; console node; reserved-memory node | W04 |
| CPU topology semantics | vCPU count semantics; topology expression (as declared); Guest EL; system-register presentation classes; feature classification outcomes | W05/W04 |
| PSCI | PSCI version; mandated function set; calling convention facts; CPU_ON/OFF and SYSTEM_OFF route semantics | W06 |
| Timer | counter presentation; virtual timer semantics; frequency semantics; WFI/wakeup interaction | W08 |
| Console | console device presentation; register-window containment semantics | W09 |
| vGIC | GIC model/version presented; distributor/redistributor presentation; LR/maintenance behavior class | W07 |

Explicitly **non-comparable** entries (listed so they cannot accrete into ABI):
validation RAM-class capacities (W12), workload markers and scenario bounds
(W11/W12), fixture build metadata (W15), telemetry/trace encodings (P0-W12),
and Host-side implementation details of any kind.

## 4. Change classes

| Class | Definition | Required handling |
|---|---|---|
| C1 — compatible internal change | Zero delta in every matrix entry's value and semantics (implementation, host-side organization, performance-neutral internals) | Ordinary PR review; matrix untouched; reviewer confirms "no matrix entry affected" |
| C2 — additive, contract-governed change | Adds a reservation or Guest-visible surface that alters no existing entry (e.g., a newly reserved window declared by the approved governance route) | Routed through the machine-contract governance ([W02](../p8-w02-machine-contract-governance/README.md)); matrix gains rows by citation; version decision belongs to that route, not to W14 |
| C3 — Guest-visible change | Any alteration of an existing entry's value or semantics | Machine version change per ADR-040 and the governance route; the matrix's prior state is cited; **never** absorbed silently |
| C4 — ADR-level change | A change contradicting or superseding a decided ADR (e.g., machine identity, platform independence, versioning model) | `ADR Required` / superseding ADR per the baseline's change rules; the affected entries are re-baselined only after the ADR lands |

Classification aid for reviewers: if a proposed change alters a `value` cell,
or the meaning a `semantic-assertion` checks, or a `presence/absence` state, it
is C3 or C4 — never C1. If it only adds and the governance route has approved
the addition, it is C2.

## 5. Escalation conditions

- **Drift detected** (a comparison fails against the approved record): the
  affected change is blocked; the delta is classified C1–C4; C3 requires the
  version route before merge; C4 requires the ADR process. Drift is never
  resolved by updating the matrix to match the implementation.
- **Source record changes** (an owner package revises its contract): the matrix
  rows citing it are re-reviewed in the same change that approves the revision;
  a lagging matrix is drift.
- **Unapproved source** (v1 facts absent): all dependent rows stay `blocked`;
  the drift test stays `blocked`; P8-V19 cannot pass. This is the plan's
  explicit condition, not a W14 discretion.
- **QEMU-vs-hardware divergence**: recorded as a Platform Investigation per the
  task book; it never changes a matrix value (see §7).
- **Proposed validation-parameter change** that would alter a Guest-visible
  fact: stop — it is C2/C3 by definition and leaves the validation lane.

## 6. Drift-detection test plan (policy level; mechanics are W16's)

- **Control variable:** the same approved v1 configuration — the approved
  machine-contract record plus the pinned
  [W15](../p8-w15-reproducible-linux-fixture/README.md) fixture targeting it.
  Any change to either invalidates the comparison and re-baselines the test.
- **Procedure:** boot the fixture on the reference environment; collect, for
  each unblocked matrix entry, the evidence its `comparison` column requires —
  Guest-observable facts via the Guest DTB/console/session and hypervisor-query
  facts via the evidenced inspection paths; compare against the approved
  record's values.
- **DTB assertions:** the Guest DTB presented at boot is compared against the
  W04 contract's declared facts (compatibles, topology, memory, PSCI, timer,
  GIC, console, reserved-memory) — the DTB is both a matrix source and a drift
  surface, so it is compared, not trusted.
- **Passing condition:** every unblocked entry matches; every blocked entry
  remains explicitly blocked. One mismatched unblocked entry fails the run and
  triggers §5 escalation.
- **Evidence:** per-entry comparison results in the verification record;
  failures carry the fact_id and both sides of the delta.
- **Boundary:** the test proves fact-level stability of the declared machine
  surface. It does not prove guest-software compatibility, performance
  stability, hardware equivalence, or migration/snapshot compatibility.

## 7. QEMU firewall

The following can never enter a matrix value or pass a comparison as an ABI
fact: QEMU version or build behavior; timing, latency, or performance
observations; Host CPU/firmware facts; Host physical addresses or IRQ numbers;
counter rate measurements (only the declared frequency *semantics* compare);
debug/semihost behavior; and any observation not declared by an owner record.
QEMU `virt` is the reference test environment (ADR-003); it does not define the
machine ABI (task book §1). A reviewer finding a QEMU-derived value in any
matrix cell, or a test assertion keyed to QEMU behavior, rejects the change and
cites this section; persistent disagreement routes through the W02 governance
as a host-independence issue, and if it would alter the machine-contract
boundary, it is labeled `Architecture Change Request`.
