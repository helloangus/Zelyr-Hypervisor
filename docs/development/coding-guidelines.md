# Zelyr Coding Guidelines — Concise Mandatory Guide

Chinese readers can use the [Chinese edition](coding-guidelines.zh-CN.md).

**Status:** Mandatory for every code-change task.  
**Scope:** Rust implementation of an approved design. This is not permission
to change architecture, ABI, state machines, ownership, locking, or layering.  
**Version:** v0.1  
**Owner/change context:** P0 engineering baseline; updated when coding policy
changes.  
**Supersedes:** None.

Read this document, the repository [agent instructions](../../AGENTS.md), the
baseline ADR, applicable Stage Task Book, and approved detailed design before
coding. Then consult the relevant sections of the [detailed reference](coding-guidelines-v0.1.md).

## Authority and preflight

For coding, authority is:

```text
explicit task > approved detailed design > frozen interface / ABI / state machine
> Architecture ADR > these guidelines > personal preference
```

Before editing, identify the target module/function, inputs/outputs, ownership,
lifecycle state machine, concurrency context, error meaning, platform boundary,
ABI stability, and whether allocation or blocking is allowed. Stop and raise a
design conflict if a required decision is missing or contradictory.

## Non-negotiable implementation rules

- Hypervisor EL2 code is Rust-first and `no_std`; use assembly only for a
  necessary architectural boundary. Do not change toolchain, target, edition,
  or add unstable features/dependencies without explicit approval.
- Preserve crate layering. Generic Core must not depend on a board, SoC,
  QEMU-specific constant, or architecture register. Put platform behavior in
  Arch/SoC/Board/Driver/Quirk layers and select capabilities, not board names.
- Use semantic newtypes for HPA/HVA/GPA/GVA, IDs, lengths, pages, and handles.
  Never substitute naked `usize`/`u64`; use checked arithmetic and conversions
  for all externally influenced address/length calculations.
- Treat Guest, device, firmware, DMA descriptor, MMIO, and management input as
  untrusted. Validate range, alignment, overflow, permission, state, and format
  before use. A guest error must not panic the hypervisor.
- Keep invariants in types and explicit state machines. Do not directly mutate
  lifecycle fields, replace capability checks with role/VM-ID checks, or add a
  convenience global singleton.
- Minimize and isolate `unsafe`. Each `unsafe` block/function needs a nearby
  `SAFETY` explanation and a small audited boundary. Do not use `unsafe` to
  evade the borrow checker; avoid `static mut`, casual `transmute`, and raw
  pointer lifetime assumptions.
- For MMIO, registers, page tables, TLBs, DMA, IOMMU, IRQ, and assembly, use
  volatile access, required reserved-bit handling, barriers, ordering, cache
  maintenance, TLB invalidation, and SMP shootdown semantics. QEMU success is
  not evidence that these hardware rules can be omitted.
- Bound locks, interrupt-context work, allocations, atomics, recursion, queues,
  and hot-path dispatch deliberately. Document synchronization meaning and
  never block or perform long/heavy work in IRQ/VM-exit paths.
- External ABI and persistent/snapshot formats require explicit representation,
  width, endianness, padding, versioning, and compatibility behavior. Never
  serialize raw Rust struct memory as a contract.
- Keep changes minimal. Do not hide warnings with broad `allow`, leave
  reachable `unwrap`/`expect`/`todo!`/`unimplemented!`, or make unrelated
  refactors while fixing a task.

## Quality, documentation, and completion

Use repository-pinned formatting and lint commands. Add/update focused unit,
integration, QEMU, Validation Guest, security, and hardware tests appropriate
to the changed contract. State precisely what ran and what did not; a build or
QEMU boot is not universal test evidence.

Document public behavior, non-obvious hardware assumptions, `unsafe` invariants,
TODO/FIXME ownership, ABI changes, and new dependencies. On completion report:
changed files; design requirements implemented; new unsafe, ABI/public API, and
dependency changes; validation run/not run; TODO/FIXME; and design conflicts.

## When to consult the detailed reference

| Trigger | Detailed-reference sections |
|---|---|
| Rust structure, types, arithmetic, errors, state | 2–27 |
| Unsafe, raw pointers, initialization | 10–18 |
| MMIO, registers, barriers, TLB, assembly, exception | 28–34 |
| ABI, feature/cfg, platform/driver/quirk | 35–44, 89, 135–139 |
| Locks, atomics, IRQ, allocation, ownership | 45–60, 107–115 |
| Guest input, virtio, DMA, IOMMU, emulation | 61–67, 125–127 |
| Traits, macros, crates, comments, telemetry, performance | 68–88 |
| Tests, review, APIs, refactoring | 91–106, 128–134 |
| Capability, machine types, configuration, Control Domain, migration | 144–154 |
| Agent preflight, definition of done, prohibited actions | 155–163 |

If a task crosses a safety, ABI, hardware, concurrency, or security boundary,
the detailed-reference consultation is mandatory even when the change is small.
