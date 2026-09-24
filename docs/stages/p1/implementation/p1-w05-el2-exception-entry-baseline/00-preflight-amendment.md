# W05 preflight design correction

**Status:** Proposed detailed design; implementation not claimed.
**Scope:** Full GPR capture, guard-first entry and vector layout.
**Version:** v0.2
**Owner/change context:** P1 preflight, 2026-09-24.
**Supersedes:** Entry-capture §§1–3 scratch/guard protocol and vector layout;
adds TPIDR_EL2 to the audited boundary.

## Baseline and required foundation

At a05fca6 W02 supplies a boot stack and BSS establishment; no W05 assembly
exists. The old stub overwrites two GPRs before claiming original x0–x30.
Required: retain all original GPRs and acquire the nesting guard before
frame/stack access. Reserved: returning handlers, per-CPU storage and TLS.
Out of scope: Guest, SMP and GIC.

## Entry contract and state ownership

Each of sixteen 128-byte slots contains only a branch to its own landing
label. A macro may generate the sixteen landing bodies, but no stub may
stamp coordinates before saving the interrupted register values.

W05 exclusively owns TPIDR_EL2 as a temporary original-x0 holder during fatal
entry. It is not a thread pointer and retains no firmware-value promise.
No other P1 code writes or consumes it. A future TLS/SMP design must supersede
this use. The [Arm AArch64 register reference](https://documentation-service.arm.com/static/6245e828b059dc5ff9a8ccef)
identifies TPIDR_EL2 as a software register.

W05 owns one aligned guard word. Vector installation establishes Armed (1)
before VBAR is installed. Entry changes it once to Taken (0); it is never
rearmed. Its assembly representation and any Rust declaration must agree;
do not mix ordinary and atomic accesses through incompatible references.

The per-slot landing algorithm is:

1. Save original x0 in TPIDR_EL2. Leave x1–x30 untouched.
2. With x0 alone, address/read the guard. Taken branches to silent stop.
   If Armed, reconstruct its address and store zero using the zero register.
3. Only after Taken is stored, use x0 to address the static frame and save
   original x1–x30. There has been no stack push or function call.
4. Recover original x0 into an already-saved GPR and store it. Capture the
   entry SP before changing SP. Now materialize this landing's coordinates.
5. Capture exception registers, apply syndrome validity, and route through
   the existing terminal report contract.

Single boot CPU and DAIF masking justify the non-looping guard protocol.
The guard itself, landing code and frame must be valid mapping premises.
An inaccessible guard is outside the containment guarantee; after Taken,
recursive entry stops before frame writes. No return/restore path is added.
TPIDR access and guard operations are added explicitly to W05's audit list.

## Linker seam and acceptance

Provide a 4 KiB-aligned vector region with start/end symbols, containing the
2 KiB architectural table and padding through the page end. KEEP the dedicated
section before broad text wildcards. Capture landing code may use ordinary
CodeRx; W08 maps the vector page separately as Vectors.

W05-DV01/DV03 inspect sixteen destinations, original-register preservation,
frame offsets and ABI stack alignment. W05-DV05 checks guard-first ordering
and TPIDR ownership. W11 must inject known values and verify representative
caller/callee-saved registers including x0, x1, x16, x17 and x30, and exercise
recursive containment (at most one report). These are future acceptance
requirements, not evidence of execution in this amendment.

