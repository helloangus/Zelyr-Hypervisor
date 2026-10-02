# P4-W03 Architecture, Objects, and State Model

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [W03](README.md). Read the revised [code contracts](03-code-contracts-guest-memory.md)
and [W12 foundation](../../../p2/implementation/p2-w12-minimal-memory-objects/README.md).

## 1. Logical modules

`gm-layout` owns versioned temporary IPA values and validation; `gm-ram` owns
MemoryObject control capabilities and content state; `gm-image` validates the
embedded flat image; `gm-loader` sequences initialization and requests architecture
code-visibility preparation. W12 owns backing, W11 owns Host views and W02 owns
Guest mapping leases. No module duplicates allocator accounting or sysreg access.

## 2. Core objects and ownership

### 2.1 `GuestLayout` (value object, module `gm-layout`)

Fixed Guest IPA values are independent of allocator-selected HPA. This replaces
the old identity assumption without imposing physical-placement requirements on
W04. The flat binary is linked for this versioned temporary Guest layout. Image,
stack and boot-info are inside RAM and disjoint; base console is outside RAM.
Layout changes require review with W04/W05/W08 and a version bump; no P8 ABI.

### 2.2 `GuestRam` (module `gm-ram`)

Holds MemoryObject control capability, usable size, layout and content state.
W12's store holds the unique full W04 allocation handle. GuestRam cannot release
it while any view or use pin remains, including quarantined transactions. W02
owns only leases and does not return Guest backing directly to the allocator.

### 2.3 `GuestImage` (module `gm-image`)

Non-owning embedded byte view and build identity. Validation produces a checked
object-relative copy plan; HVA is never part of its ownership authority.

### 2.4 `BootInfo` (module `gm-loader`)

Explicit little-endian fields, zero reserved bytes and deterministic checksum.
No raw Rust-struct serialization. Base P4 Guest validates it independently.
The W10 RAM-only fixture is a separate extension asset, not a silent change to
this base test convention or a shared-console device model.

## 3. Guest RAM lifecycle state machine

`Allocated -> Zeroed -> Loaded -> Released`; a post-write failure enters
Failed/Retained. Zeroed/Loaded describe bytes, not whether mappings exist.
W12/W02 are the sole view-state authorities. Guest entry needs Loaded plus
committed Guest mappings and W04 execution permission. Failed data cannot enter.
Re-init is allowed only after all Guest mappings retire and an exclusive Host
writer is reacquired. Quarantine is not an implicit retryable state.

## 4. Sequencing and integration contract

1. Validate layout/image/scenario and create the W02 space.
2. Allocate/adopt Guest backing into W12; map a W11 Host RW region.
3. Validate full serialization/copy plan, zero every usable byte, write BootInfo
   and image; complete code-visibility preparation and end the byte callback.
4. Revoke the Host writer completely; reserve W12 disjoint Guest regions and map
   them into W02. The base console uses its separate authority.
5. Construct vCPU; select the space, acquire its execution lease and enter via W04.
6. On teardown stop/exit Guest and retire its execution lease, explicitly detach
   the installed context, then unmap/destroy with completed resident retirement.
7. W12 permits take_back only after every view/pin has ended; W03 frees the exact
   returned W04 handle. Failed free remains an owned resource.

Same-session repeats may use new objects or explicitly retire/reinitialize the
old object. Both obey steps 2–7; no stale Guest mapping may coexist with the loader
writer. Merely clearing a vCPU run-state does not authorize memory reuse.

## 5. Deterministic-initialization contract

Successful content is all zeros except deterministic BootInfo and image bytes.
Input validation happens before stores; later hardware/write failure may leave
partial bytes and cannot claim rollback to zero. Failed content is inaccessible
to Guest; only a safely reacquired writer may explicitly reinitialize it.

## 6. Concurrency model

Setup/teardown run with no Guest access to these objects. W12 rejects overlapping
Host-writable and Guest views even on different CPUs. Bounded Host callbacks do
not escape; external use is explicitly pinned. Base W11 is boot-only: actual P4
SMP access needs its reviewed P3 adapter and cross-CPU completion. W10 extends
Stage-2 retirement; it does not automatically extend Host Stage-1 or instruction
visibility. No lock is held across copying, hardware callbacks or transport wait.

## 7. Telemetry points (W03-scope)

P0-routed allocate/load/retire/release events carry object/region/space identity,
size, phase and retained vs completed outcome. Do not log Guest content. W07
counts successful initialization and actual free separately from retained failures.
