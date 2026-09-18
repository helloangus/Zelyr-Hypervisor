# P3-W02 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W02 detailed design](README.md).

## 1. Two-sided architecture

Secondary bring-up has exactly two execution contexts, and every object in
this design belongs to one of them:

```text
BOOT CPU (requester)                          SECONDARY (entry path)
---------------------------                   ---------------------------
allocate provisional stacks                   [still firmware context]
for each Present-class non-boot CPU:          entry symbol (positioned)
    request transition Present->Starting (W03)     <- PSCI CPU_ON entry
    issue PSCI CPU_ON                    set sp = provisional stack top
    on call error: report failure             revalidate entry state (P1)
  poll arrival mailboxes (bounded)            read MPIDR, confirm identity
    arrival+success: transfer stack ownership      -> W03 Initializing
    arrival+failure: record phase/cause       establish minimal environment
    no arrival within bound: timeout          report result (mailbox)
  report outcome map to W05/W13               success: await rendezvous (W05)
                                              failure: diagnose, park forever
```

The two sides share exactly three objects, each with one writer:
the per-CPU arrival mailbox (written by the secondary), the per-CPU result
record (written by the secondary), and the provisional stack (allocated by
the boot CPU, used by exactly one secondary). The lifecycle state machine
is P3-W03's; W02's transitions are calls into it.

## 2. Logical modules

| Logical module | Side | Responsibility | Owned state | Non-responsibility |
|---|---|---|---|---|
| A. Start requester | boot CPU | candidate iteration, transition requests (W03 calls), PSCI call dispatch, outcome mapping, timeout watching, diagnostics | per-CPU outcome map (built, then handed to consumers) | lifecycle state authority (W03); rendezvous (W05) |
| B. PSCI call boundary | boot CPU | the single `unsafe` firmware-call function; conduit dispatch (HVC/SMC) from recorded facts | none (pure call) | PSCI function selection beyond CPU_ON; guest PSCI (P8) |
| C. Secondary entry | secondary | entry revalidation, identity confirmation, minimal environment, result reporting, parked-failure loop | writes its own mailbox + result record only | exception vectors (P1/P3-W09); per-CPU runtime (W04) |
| D. Provisional environment | both (allocated by A, used by C) | stack allocation, sizing constant, ownership transfer/quarantine | the stack allocations | real per-CPU runtime areas (W04) |

Dependency direction: A and B depend on C only through the entry symbol's
address; C depends on A's allocations only through the entry parameters.
Neither side depends on the other's Rust code paths, which keeps the
assembly boundary honest.

## 3. The phase model

Phases exist to make P3-V02's "CPU and phase attribution" precise. A phase
is a named checkpoint on the secondary side; the requester can observe only
two places (call return and mailbox arrival), so intermediate phases are
*reported by the secondary inside its result record*, and the phases
`RequestDispatch` and `Timeout` are attributed by the requester:

```text
RequestDispatch       requester: CPU_ON returned an error
EnteredEl2            secondary: executing at EL2 (first Rust statement)
EntryStateValidated   secondary: P1 entry contract holds (CurrentEL, security state)
IdentityConfirmed     secondary: MPIDR matches requested target + context id
LocalEnvironmentReady secondary: stack active, minimal state applied, W03 Initializing set
```

A successful start completes all five phases. Any failure names the phase
at which it stopped. `Timeout` means the mailbox never became non-empty
within the poll bound; the CPU's true progress is then unknown, which is
precisely why the outcome is terminal and the CPU is excluded from the
online set by W03.

## 4. Concurrency model

- **Single requester.** Only the boot CPU issues CPU_ON during P3. No
  concurrent start requests exist, so the requester needs no locking; the
  plan's consumer ordering (W03–W05 after W02) matches.
- **Single-writer mailbox.** For each attempted CPU: the secondary is the
  only writer of its result record and mailbox word; the requester is the
  only reader during bring-up. Ordering: result record fully written, then
  `Release` store of the mailbox sentinel; requester `Acquire` loads. On
  AArch64 this compiles to the appropriate load/store with implicit
  barrier semantics; no explicit `dmb` is added inside W02's protocol, and
  none is needed for single-variable publication.
- **Pre-release allocation.** All provisional-stack allocation happens
  before the first CPU_ON (guaranteed by the P3-W05 phase model, asserted
  by the requester as a documented precondition). Therefore no allocation
  ever runs concurrently with a secondary's execution, and no P2 allocator
  behavior under concurrency is relied on by W02.
- **No locks.** W02 introduces no spinlock and no IRQ-saving critical
  section; those semantics belong to P3-W06. If implementation appears to
  need a lock, that is a design error to raise, not a local primitive to
  add.
- **Parked CPUs do not spin on shared state.** A parked failed secondary
  loops on a local no-op (WFE Reserved for the P3-W07 trigger) and writes
  nothing further.

## 5. The minimal local environment (what W02 does and does not establish)

On success, before signaling arrival, the secondary has: its provisional
stack active as `sp`; the identity cross-check completed; the lifecycle
state moved to Initializing via the W03 transition call; and the P1
architectural baseline applied/verified to the bounded extent the P1-W04
contract states as per-CPU-reproducible. It does **not**: install per-CPU
runtime areas or a runtime stack (W04), enable interrupts or touch the
GIC (P3-W09/P6), or participate in rendezvous (it signals the P3-W05 gate
after W02's scope ends — the entry path's success tail calls the W05
per-CPU readiness signal, by contract, but the rendezvous semantics are
W05's).

## 6. Failure isolation

- A failed or timed-out secondary never joins the online set (W03 enforces
  via its transition table; W02 supplies the report).
- A secondary that fails inside the entry stub after the P1 console is
  usable emits a CPU-attributed diagnostic before parking; if the failure
  occurs before the console is usable, the phase record still reaches the
  requester's outcome map via the mailbox, and the requester's diagnostic
  identifies the CPU and phase. No failure path leaves both sides silent.
- The requester never re-dispatches a CPU whose call returned an error or
  whose outcome reached a terminal state (no retry in P3).
- Registry or mailbox corruption is an invariant violation, not a
  recoverable error: it takes the P1-W07 fatal path with CPU attribution.
