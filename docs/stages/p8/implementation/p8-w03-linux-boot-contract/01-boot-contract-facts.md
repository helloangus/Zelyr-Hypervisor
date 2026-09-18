# P8-W03 Boot-Contract Fact Catalog

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W03 detailed design](README.md).

## 1. Boot-contract document contract

Create `docs/machine-types/linux-boot-contract-v0.1.md` as a normative,
versioned document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
exactly the following sections:

1. **Identity and binding** — machine identity `rusthv-arm-virt-v1`; boot
   contract version; statement that the contract is a family contract under
   the machine-contract governance of
   [P8-W02](../p8-w02-machine-contract-governance/README.md) (cite the
   governance document at `docs/machine-types/machine-contract-governance-v0.1.md`
   instead once it exists).
2. **Boot inputs** — fact group B1.
3. **Boot-vCPU state and entry** — fact groups B2–B3.
4. **DTB transfer and placement** — fact group B4.
5. **Boot execution model** — fact group B5.
6. **Secondary vCPUs at boot** — fact group B6.
7. **Boot-artifact regions and lifetime** — fact group B7.
8. **Shutdown transition** — fact group B8.
9. **External-standard bindings** — the pinned documentation revisions of
   §2.
10. **Testability and evidence expectations** — per
    [02-fixture-and-shutdown-evidence.md](02-fixture-and-shutdown-evidence.md)
    §3–§4.
11. **Compatibility** — per §10 below.

Required content per fact group is stated below. Additional informative
detail is allowed but must not contradict a required statement, may not fix a
routed value, and may not add a Guest-visible fact outside the change rules
of §10.

## 2. External-standard bindings

The contract binds Guest-visible boot facts to named, pinned external
standards; Zelyr does not redefine them. The contract must carry:

- **Linux AArch64 boot protocol** — the kernel's documented AArch64 Image
  boot requirements (entry register state, MMU/cache expectations, DTB
  pointer passing, Image header/text-offset semantics). The contract states
  the *exact documentation revision* it binds to; the revision is selected at
  contract-implementation time from the Linux documentation of the kernel
  series the fixture will pin (fixture selection belongs to W15) and is
  recorded in the External-standard bindings section. Binding to a revision
  is a Guest-visible fact and follows the W02 category route; Zelyr never
  invents a register convention.
- **Devicetree specification** — the DTB wire format (structure block
  layout, big-endian 32-bit cells) is consumed through the Devicetree
  specification as cited by the Guest DTB contract ([P8-W04](../p8-w04-guest-dtb-contract/README.md));
  the boot contract references it for transfer only and does not restate
  content facts.
- **PSCI** — referenced only as the conduit for facts B6/B8; version, subset,
  and conduit choice are routed (W02 firmware category; mechanism owned by
  [P8-W06](../p8-w06-psci-virtualization/README.md)).

Consistent with the Coding-Guidelines ABI rule: no boot fact is ever defined
as the memory layout of a Rust type; the contract speaks in protocol terms
(register names and required values per the pinned standard, artifact
categories, ordering), and any machine-readable encoding it later gains is
itself a versioned contract with explicit width/endianness/compatibility.

## 3. Fact group B1 — boot inputs

| ID | Required fact (the contract must state) | Class | Authority / route |
|---|---|---|---|
| B1-1 | Boot requires exactly one bootable Linux kernel Image in the documented AArch64 boot-image form of the pinned protocol; no firmware, bootloader, or UEFI stage is presented to the Guest | Permanent Guest-visible | ADR-021; protocol binding §2 |
| B1-2 | Boot optionally carries one initramfs artifact as a single contiguous Guest-RAM region; absence is a supported configuration | Permanent Guest-visible | ADR-021 (Image+DTB+initramfs); task book §2 |
| B1-3 | Bootargs are supplied through the Guest DTB (`/chosen` bootargs), not through registers or a separate wire structure | Permanent Guest-visible | ADR-025 (DTB-first); content authority [W04](../p8-w04-guest-dtb-contract/README.md) |
| B1-4 | Each artifact has a stated size-bound rule: maximum Image, DTB, and initramfs sizes (or size relations to Guest RAM) that the hypervisor accepts; boot input violating a bound is rejected with a structured boot-input error before Guest entry, never by truncation or silent clamping | Permanent Guest-visible (bounds values routed) | task book §1 (no value selected here); Guest-input-untrusted constraint (W01 register) |
| B1-5 | Artifact placement respects the machine's Guest physical-address categories (RAM, reserved-memory, MMIO windows) of the bound machine identity; placement values are routed machine facts | Permanent Guest-visible (categories via W02 C3; values `ADR Required`) | W02 governance §4 C3; §6 |
| B1-6 | Artifact backing is allocatable Host memory owned by the VM per the P2/P4 ownership contracts; no artifact aliases Host image, DTB, hypervisor, or another VM's memory | Implementation fact (with Guest-visible isolation consequences per B7) | W01 register; [P4-W03](../../../p4/plans/p4-w03-guest-memory-image.md) assumed contract |

## 4. Fact groups B2–B3 — boot-vCPU state, entry, EL and MMU expectations

| ID | Required fact | Class | Authority / route |
|---|---|---|---|
| B2-1 | The hypervisor constructs the boot vCPU's entry state to satisfy the pinned Linux boot protocol: the register that carries the Guest-physical DTB pointer carries the placed DTB address; remaining general-purpose registers conform to the protocol's requirements; no Zelyr-private register convention exists | Permanent Guest-visible | Linux AArch64 boot protocol (§2); W02 C2 route |
| B2-2 | The entry program counter and execution state correspond to the Image entry defined by the pinned protocol (including the protocol's text-offset rule), expressed as protocol facts, not as Zelyr numbers | Permanent Guest-visible | Linux AArch64 boot protocol (§2) |
| B2-3 | The contract states which exception level, stack, and system-register baseline the hypervisor guarantees at entry, matching the protocol and ADR-022 (Guest EL1, no virtual EL2); any state the protocol leaves free is fixed here explicitly as a Guest-visible fact through the W02 route | Permanent Guest-visible | ADR-022; Linux protocol; W02 §4 C2 |
| B3-1 | Stage-2 translation is active for the Guest from entry; the Guest's addresses are Guest-physical (IPA) per the bound machine identity; there is no hypervisor-provided identity-mapping service at runtime | Permanent Guest-visible | ADR-018; ADR-022; task book §2 (P4 row) |
| B3-2 | The Guest's stage-1 MMU and cache state at entry match the pinned protocol's expectations (the protocol's documented off/initial state); the contract states the guarantee, not the mechanism | Permanent Guest-visible | Linux protocol (§2); W02 C2 route |

## 5. Fact groups B4–B5 — DTB transfer and boot execution model

| ID | Required fact | Class | Authority / route |
|---|---|---|---|
| B4-1 | The Guest DTB is produced by the hypervisor per the Guest DTB contract (by reference — the boot contract owns transfer and placement only), placed in Guest RAM within the machine's DTB placement category, and its Guest-physical address is passed per B2-1 | Permanent Guest-visible (placement value routed) | [W04](../p8-w04-guest-dtb-contract/README.md); W02 C6; task book §1 |
| B4-2 | The DTB is valid and complete at the moment of Guest entry; the contract states the Guest-visible guarantee and the ownership rule from entry onward (see B7-3) | Permanent Guest-visible | ADR-025; [W04](../p8-w04-guest-dtb-contract/README.md) |
| B5-1 | After entry, the Guest runs without hypervisor-mediated assistance for normal instruction execution; hypervisor involvement occurs only through the defined machine mechanisms (traps per the W02 categories, interrupts, timer, PSCI, console) | Permanent Guest-visible | ADR-005/ADR-008; task book §2 (P4 row) |
| B5-2 | The boot sequence is deterministic per the fixture: same pinned inputs and machine identity produce the same boot-visible state at entry and the same boot-progress markers, subject only to documented non-determinism (e.g., counter values) | Implementation fact (with test force) | P8-V04/V21; [P8-W16](../p8-w16-automated-linux-regression/README.md) consumes |

## 6. Fact group B6 — secondary vCPUs at boot

| ID | Required fact | Class | Authority / route |
|---|---|---|---|
| B6-1 | At boot entry exactly one vCPU — the boot vCPU — executes Linux; secondary vCPUs, if the VM configuration defines them, exist in a hypervisor-held state that presents no executing context to the Guest | Permanent Guest-visible | plan scope (secondary state); ADR-022 |
| B6-2 | Secondary vCPUs begin Guest execution only through the machine's PSCI CPU-start path (version, subset, and conduit routed; mechanism owned by [P8-W06](../p8-w06-psci-virtualization/README.md)); the boot contract states the boot-time fact — no secondary enters Linux by any other route — and defers the mechanism entirely | Permanent Guest-visible (subset routed) | task book work map (W06); ADR-015/016 |
| B6-3 | The contract states what a secondary's entry state must satisfy when started (protocol-conformant state per the pinned Linux CPU-start expectations), as a constraint handed to W06, without choosing PSCI values | Permanent Guest-visible (values routed) | Linux protocol (§2); [W06](../p8-w06-psci-virtualization/README.md) consumes |

## 7. Fact group B7 — boot-artifact regions and lifetime

| ID | Required fact | Class | Authority / route |
|---|---|---|---|
| B7-1 | Each boot artifact occupies a region within exactly one Guest physical-address category of the bound machine; regions do not overlap each other or any reserved-memory or MMIO category; placement values are routed machine facts | Permanent Guest-visible (values `ADR Required`) | W02 C3/C9; task book §1 |
| B7-2 | From Guest entry onward, all boot artifacts are Guest VM memory: the hypervisor does not reclaim, relocate, or rely on their contents while the VM exists; region lifetime ends with VM destruction per the P4 GuestAddressSpace contract | Permanent Guest-visible | ADR §19; [P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md) assumed contract |
| B7-3 | The DTB and bootargs are boot-time data: the Guest may consume them per Linux norms; the contract states whether the Guest-visible guarantee of validity persists after mutation and what the hypervisor itself may assume — proposed posture: the hypervisor makes no post-entry assumption about artifact contents | Permanent Guest-visible | Linux DTB-consumption norms; [W04](../p8-w04-guest-dtb-contract/README.md) consistency review |
| B7-4 | Boot-input acceptance (bounds, placement, overlap checks of B1-4/B7-1) occurs before Guest entry and its failure is a structured boot-input rejection attributable to the boot configuration, not a Guest fault and not a hypervisor error | Implementation fact | Guest-input-untrusted (W01); P5 error-classification assumed contract |

## 8. Fact group B8 — shutdown transition

| ID | Required fact | Class | Authority / route |
|---|---|---|---|
| B8-1 | A Guest-initiated shutdown is expressed only through the machine's PSCI system-level path (subset routed; mechanism W06); the boot contract states the observable outcome, not the conduit encoding | Permanent Guest-visible (subset routed) | task book work map (W06); ADR-008 |
| B8-2 | On accepted shutdown, the VM reaches a defined terminal lifecycle state consistent with the ADR §4.1 VM lifecycle and the P4/P5 lifecycle facts; terminal state, diagnostics, and idempotent repeat behavior are stated as facts | Permanent Guest-visible | ADR §4.1; P4-W09/P5-W10 assumed contracts |
| B8-3 | After shutdown, all VM resources (memory, vCPUs, artifact regions) are released per the established VM-destruction contracts; the contract states the resource-visibility fact for evidence (a shut-down VM leaves no Guest-owned memory live) | Implementation fact | P4/P5 lifecycle assumed contracts |
| B8-4 | Shutdown behavior is testable without a userspace signal path: the evidence expectations (§10 of the contract) define the markers and terminal-state observations | Implementation fact | [02](02-fixture-and-shutdown-evidence.md) §4; [W16](../p8-w16-automated-linux-regression/README.md) consumes |

## 9. Testability form

Every fact row in the contract must be phrased so a reviewer can name the
observation that would test it without selecting an implementation detail:
the observation names the Guest-visible signal (boot marker, register state
asserted at entry, DTB property, terminal-state observation) and the
environment (QEMU reference platform), never the mechanism that produces it.
A fact that cannot be phrased this way is either misclassified (implementation
fact) or is a routed value — the reviewer rejects the row, not the rule.

## 10. Compatibility section requirements

Per the W02 governance §8: the contract's compatibility section must list
compatible changes (clarifications, implementation-fact changes, fixture
revisions that change no Guest-visible fact), declare that any change to a
Permanent Guest-visible fact forces a new boot-contract version and a
machine-compatibility review, and state the binding to the machine identity —
including which machine identities (initially only `rusthv-arm-virt-v1`) the
version applies to. The section must also state the external-standard
re-binding rule: pinning a different Linux documentation revision is a
reviewed compatibility event, assessed against the pinned fixture of W15,
because it can move Guest-visible entry facts.
