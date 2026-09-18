# P8-W04 DTB Fact Catalog

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W04 detailed design](README.md).

## 1. Guest-DTB document contract

Create `docs/machine-types/guest-dtb-contract-v0.1.md` as a normative,
versioned document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
exactly the following sections:

1. **Identity and binding** — machine identity `rusthv-arm-virt-v1`; DTB
   contract version; family-contract statement under the
   [machine-contract governance](../p8-w02-machine-contract-governance/README.md)
   (cite the governance document instead once it exists).
2. **Wire format and generation source** — §2 below.
3. **Required facts** — fact groups D1–D10 (§3–§12 below).
4. **Binding-source rule** — §13.
5. **Consistency requirements** — by reference to
   [02-consistency-and-host-leakage-review.md](02-consistency-and-host-leakage-review.md)
   §2.
6. **Prohibited content** — by reference to
   [02-consistency-and-host-leakage-review.md](02-consistency-and-host-leakage-review.md)
   §3.
7. **Testability and evidence expectations** — the observation form of
   [02-consistency-and-host-leakage-review.md](02-consistency-and-host-leakage-review.md)
   §4.
8. **Compatibility** — §14 below.

Required content per fact group is stated below. Additional informative
detail is allowed but must not contradict a required statement, may not fix
a routed value, and may not add a Guest-visible fact outside the change
rules of §14.

## 2. Wire format and generation source

The contract must state:

- **Representation.** The Guest DTB is a flattened devicetree (FDT) blob per
  the pinned Devicetree-specification revision: a header, a structure block
  of BEGIN_NODE/END_NODE/PROP/END tokens and property values, and a strings
  table; all fields and cell values are explicit-width 32-bit big-endian
  quantities as the specification defines. The pinned revision is recorded
  in the contract's identity section; Zelyr does not redefine the format.
- **No native-struct serialization.** DTB content is never defined as the
  memory image of a Rust or C type; any future generator emits the token
  stream explicitly with checked cell widths and counts (Coding-Guidelines
  ABI rule).
- **Generation source.** The DTB is a deterministic function of (a) the
  bound machine identity's approved facts, (b) the VM's boot configuration
  (vCPU count, RAM size, boot artifact placement per the boot contract),
  and (c) this contract. It is never a function of the Host DTB, host
  PlatformInfo, host addresses/IRQs, or host observation.
- **Size and placement relation.** The contract states the DTB's size-bound
  rule and that its placement and entry-time transfer follow the boot
  contract (fact B4); values are routed.

## 3. D1 — Root identity: `compatible` and `model`

| ID | Required fact | Source / route |
|---|---|---|
| D1-1 | The root node carries a `compatible` string that expresses the machine identity (`rusthv-arm-virt-v1` naming space), so Guest software can identify the virtual machine family | Value routed via W02 C1 review (Guest-visible ABI value); rule fixed here |
| D1-2 | The root node carries a human-readable `model` string derived from the machine identity | Value routed with D1-1 |
| D1-3 | No board, SoC, vendor, or hypervisor-host identity string appears in the root or any node except as this contract defines | Prohibition ([02](02-consistency-and-host-leakage-review.md) §3) |

## 4. D2 — CPU and topology

| ID | Required fact | Source / route |
|---|---|---|
| D2-1 | `/cpus` describes exactly the configured vCPUs, each as an enabled CPU node with a Guest-meaningful reg property, per the CPU binding of the pinned Devicetree/Linux documentation | Count/placement from VM boot configuration; binding from published documentation; topology expression routed (W02 C2 `Specification Investigation`) |
| D2-2 | CPU nodes state the PSCI enable-method so Linux's secondary-CPU path uses the machine's PSCI route (consistency with boot-contract fact B6-2 and the [P8-W06](../p8-w06-psci-virtualization/README.md) contract) | Binding fixed here; PSCI values routed (D5) |
| D2-3 | CPU nodes present only Guest-visible CPU facts (MPIDR-equivalent affinity representation per the binding, Guest EL posture per ADR-022); no host CPU identity, host frequency, or host erratum data appears | Prohibition ([02](02-consistency-and-host-leakage-review.md) §3); CPU feature posture routed (W02 C2) |

## 5. D3 — Memory

| ID | Required fact | Source / route |
|---|---|---|
| D3-1 | One or more `/memory` nodes describe exactly the Guest RAM the VM configuration defines, as Guest-physical ranges within the machine's RAM category | Values routed (W02 C3 `ADR Required`); rule fixed here |
| D3-2 | The described RAM equals the Guest RAM the Stage-2 address space actually maps (no node may describe memory the Guest cannot access, and no accessible RAM may be undescribed) | Consistency ([02](02-consistency-and-host-leakage-review.md) §2); Stage-2 facts per [P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md) assumed contract |

## 6. D4 — `/chosen`: bootargs and initrd

| ID | Required fact | Source / route |
|---|---|---|
| D4-1 | `/chosen` carries the bootargs string (boot-contract fact B1-3: bootargs are delivered via `/chosen`, not registers) | Delivery rule fixed here (W03 B1-3); string value pinned by the [W15](../p8-w15-reproducible-linux-fixture/README.md) fixture |
| D4-2 | When an initramfs is configured, `/chosen` carries its Guest-physical location per the Linux documented initrd properties; when absent, the properties are absent (no zero placeholders) | Binding from published Linux documentation; placement values from boot configuration (B1-2/B7-1); consistency with boot contract |
| D4-3 | `/chosen` carries the `stdout-path` reference to the console node (D8) | Rule fixed here; console device category routed (W02 C7; [W09](../p8-w09-virtual-console-single-cpu-linux/README.md) owns the device) |

## 7. D5 — PSCI node

| ID | Required fact | Source / route |
|---|---|---|
| D5-1 | The DTB presents a PSCI node describing exactly the conduit and function set the machine's approved firmware category defines — the DTB must not promise a PSCI function the hypervisor does not implement, and must not omit one it requires Guests to use | Consistency with [P8-W06](../p8-w06-psci-virtualization/README.md) contract and W02 C6; PSCI version/subset/conduit values routed (`Specification Investigation`) |
| D5-2 | The PSCI node uses the published Linux/Devicetree PSCI binding form | Binding fixed here |

## 8. D6 — Timer node

| ID | Required fact | Source / route |
|---|---|---|
| D6-1 | The DTB presents the architected-timer node describing the virtual timer access the machine provides (per-CPU virtual timer interrupts and frame access per the binding), consistent with the [P8-W08](../p8-w08-linux-timer-integration/README.md) contract and W02 C5 | Rule fixed here; interrupt specifiers and frequency expression routed (`Specification Investigation`) |
| D6-2 | No host timer/counter fact (host frequency tuning, host erratum workaround properties) appears | Prohibition ([02](02-consistency-and-host-leakage-review.md) §3) |

## 9. D7 — GIC node

| ID | Required fact | Source / route |
|---|---|---|
| D7-1 | The DTB presents the interrupt controller as the GICv3-class device the machine's interrupt category defines (distributor and redistributor register frames, maintenance interrupt per the binding), consistent with the [P8-W07](../p8-w07-linux-vgicv3/README.md) contract and W02 C4 | Rule fixed here; register frames, interrupt numbers routed (`ADR Required` values) |
| D7-2 | No ITS/LPI/MSI node is presented in v1 (later-stage capability, ADR §15 P9+; task book out-of-scope list) | Exclusion fixed here |
| D7-3 | Interrupt cells use Guest-physical interrupt identifiers of the machine's interrupt category; host IRQ numbers never appear | Values routed; prohibition ([02](02-consistency-and-host-leakage-review.md) §3) |

## 10. D8 — Console node

| ID | Required fact | Source / route |
|---|---|---|
| D8-1 | The DTB presents the non-Virtio console device as a standard, documented binding node (the machine's console category), referenced by `/chosen` stdout-path (D4-3) | Device category routed (W02 C7); device contract owned by [P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md); binding form from published documentation |
| D8-2 | The console node describes only the Guest-visible device surface (register frame within the machine's MMIO category, Guest interrupt per D7-3 rules); no host serial-port or board UART identity appears | Values routed; prohibition ([02](02-consistency-and-host-leakage-review.md) §3) |

## 11. D9 — Reserved memory

| ID | Required fact | Source / route |
|---|---|---|
| D9-1 | The DTB describes Guest-reserved memory regions exactly as the machine's reserved categories define, so Linux excludes them from allocation | Rule fixed here; region values routed (W02 C3/C9) |
| D9-2 | No Host reserved-memory region, Host kernel region, or Host firmware reservation is reflected | Prohibition ([02](02-consistency-and-host-leakage-review.md) §3) |

## 12. D10 — P8 minimal device set

| ID | Required fact | Source / route |
|---|---|---|
| D10-1 | The v1 DTB describes only the P8 minimal device set: console (D8) plus the firmware/timer/GIC/CPU/memory facts of D1–D7 and D9 | Exclusion fixed here |
| D10-2 | Virtio devices are not described; any statement about future virtio capacity is limited to machine reservations under the W02 C9 rules and is absent until those resolve | Task book out-of-scope list; W02 reservation prohibition |

## 13. Binding-source rule

Every node and property in the DTB must be expressible under a published
binding: the pinned Devicetree specification, the Devicetree org bindings
repository, or the Linux kernel's documented bindings of the pinned fixture
series. Zelyr defines no private binding in v1. The contract must state this
rule and carry, per fact group, the binding document its form comes from.
A needed fact with no published binding is either unnecessary (drop it) or a
routed specification-investigation item (propose through the W02 C6 route);
it is not emitted ad hoc.

## 14. Compatibility section requirements

Per the W02 governance §8: the contract's compatibility section must declare
that DTB content facts are Guest-visible machine presentation; any change to
them (adding, removing, or redefining a node/property the Guest consumes)
forces a new DTB-contract version and a machine-compatibility review; the
DTB-contract version binds to the machine identity; and re-pinning the
Devicetree-specification revision or the Linux binding set is a reviewed
compatibility event assessed against the W15 fixture. Generation-details
changes that produce byte-identical semantics are compatible changes.
