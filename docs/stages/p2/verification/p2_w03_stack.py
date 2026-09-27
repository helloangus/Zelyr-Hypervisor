"""Pinned AArch64 disassembly inventory and W03 trace filter (stdlib only).

This reports immediate stack reservations and direct-call reachability; it
explicitly reports indirect branches/cycles rather than treating them as zero.
The selected trace includes all formatting targets and their direct callees.
"""
import re
import subprocess


def analyze(elf):
    listing = subprocess.check_output(["aarch64-linux-gnu-objdump", "-d", "-C", str(elf)], text=True)
    functions = {}
    current = None
    for line in listing.splitlines():
        match = re.match(r"^([0-9a-f]+) <(.*)>:$", line)
        if match:
            current = {"name": match[2], "instructions": []}
            functions[int(match[1], 16)] = current
        elif current is not None:
            match = re.match(r"\s*([0-9a-f]+):\s+[0-9a-f]{8}\s+(.*)", line)
            if match:
                current["instructions"].append((int(match[1], 16), match[2]))
    for address, function in functions.items():
        calls = set()
        indirect = []
        frame = 0
        probe_bytes = None
        for pc, instruction in function["instructions"]:
            match = re.match(r"sub\s+x9, sp, #0x([0-9a-f]+), lsl #12", instruction)
            if match:
                probe_bytes = int(match[1], 16) * 4096
            match = re.match(r"sub\s+sp, sp, #0x([0-9a-f]+)(, lsl #12)?", instruction)
            if match:
                amount = int(match[1], 16) * (4096 if match[2] else 1)
                if probe_bytes is not None:
                    amount, probe_bytes = probe_bytes, None
                frame += amount
            match = re.search(r"\[sp, #-(\d+)\]!", instruction)
            if match:
                frame += int(match[1])
            match = re.match(r"(?:bl|b)\s+([0-9a-f]+) <", instruction)
            if match:
                target = int(match[1], 16)
                if target in functions and target != address:
                    calls.add(target)
            if re.match(r"(?:blr|br)\s+x", instruction):
                indirect.append({"pc": pc, "instruction": instruction})
        function.update(frame=frame, calls=calls, indirect=indirect)
    root = next(address for address, f in functions.items() if f["name"] == "hypervisor::boot::p2::memory_map")
    selected = {root}
    # Formatting vtables contain callback addresses, not direct branch edges.
    selected.update(address for address, f in functions.items() if "fmt" in f["name"])
    work = list(selected)
    while work:
        address = work.pop()
        for target in functions[address]["calls"] - selected:
            selected.add(target)
            work.append(target)
    ranges = []
    for address in sorted(selected):
        instructions = functions[address]["instructions"]
        if instructions:
            end = instructions[-1][0] + 4
            if ranges and address == ranges[-1][1]:
                ranges[-1][1] = end
            else:
                ranges.append([address, end])
    relevant = {f["name"]: {"address": address, "frame_bytes": f["frame"],
                           "indirect": f["indirect"]}
                for address, f in functions.items()
                if any(word in f["name"] for word in ("boot::p2", "bootmap", "discovery::normalize", "el2_rust_entry"))}
    # Repeated visits on a cycle are reported, not claimed bounded.
    cycles = set()
    memo = {}

    def longest(address, active):
        if address in active:
            cycles.add(functions[address]["name"])
            return 0
        if address in memo:
            return memo[address]
        value = functions[address]["frame"] + max(
            (longest(child, active | {address}) for child in functions[address]["calls"]), default=0)
        memo[address] = value
        return value

    direct = longest(root, set())
    fmt_max = max((longest(a, set()) for a, f in functions.items() if "fmt" in f["name"]), default=0)
    parents = sum(f["frame"] for f in functions.values()
                  if f["name"] in ("el2_rust_entry", "hypervisor::boot::p2::run"))
    owning = [f["name"] for f in functions.values()
              if f["name"].endswith("BootMapBuilder>::draft") or f["name"].endswith("MapStorage>::new")]
    report = {"root": root, "parent_frames": parents,
              "reviewed_envelope_bytes": parents + direct + 4 * fmt_max,
              "envelope_scope": "W03 normal/typed-error paths; max four non-pretty formatting callback levels (MapFatal -> SourceId/SealViolation -> scalar -> Line writer); excludes invariant panic recursion and interrupts",
              "owning_map_entries": owning, "functions": relevant, "trace_function_count": len(selected),
              "direct_acyclic_frame_sum": direct, "fmt_direct_acyclic_frame_max": fmt_max, "cycles": sorted(cycles),
              "indirect_sites": [{"name": functions[a]["name"], "sites": functions[a]["indirect"]}
                                 for a in sorted(selected) if functions[a]["indirect"]],
              "trace_ranges": ranges,
              "limit": "Direct paths exclude vtable edges; review indirect callbacks and use instruction trace. Not an unqualified whole-program bound."}
    return listing, report
