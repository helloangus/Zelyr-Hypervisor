#!/usr/bin/env python3
"""W03 runtime fixtures through the existing QEMU process/capture owner.

Optional single-instruction TCG register traces measure the executed stack
path without adding instructions or unsafe to the hypervisor. This is bounded
W03 evidence, not the full W09 matrix. Python stdlib only.
"""
import argparse
import gzip
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import threading

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts"))
from p1_runner import capture, image_identity, profile, stamp, write_json  # noqa: E402
from p2_w03_stack import analyze  # noqa: E402
from p2_w03_trace import enable_at_entry  # noqa: E402


def words(*values):
    return struct.pack(">" + "I" * len(values), *values)


def pairs(*values):
    return struct.pack(">" + "Q" * len(values), *values)


def fixture(banks, reservations=(), bad_reserved=False):
    """Minimal truthful reference-RAM/boot-CPU FDT, plus explicit fault inputs."""
    strings = bytearray()
    structure = bytearray()

    def aligned(value):
        return value + bytes((-len(value)) % 4)

    def node(name):
        structure.extend(words(1) + aligned(name.encode() + b"\0"))

    def prop(name, value):
        off = len(strings)
        strings.extend(name.encode() + b"\0")
        structure.extend(words(3, len(value), off) + aligned(value))

    def end():
        structure.extend(words(2))

    node("")
    prop("#address-cells", words(2))
    prop("#size-cells", words(2))
    node("cpus")
    prop("#address-cells", words(2))
    prop("#size-cells", words(0))
    node("cpu@0")
    prop("device_type", b"cpu\0")
    prop("reg", pairs(0))
    prop("enable-method", b"psci\0")
    end()
    end()
    for index, bank in enumerate(banks):
        node(f"memory@{bank[0]:x}")
        prop("device_type", b"memory\0")
        prop("reg", pairs(*bank))
        end()
    if bad_reserved:
        node("reserved-memory")
        prop("#address-cells", words(2))
        prop("#size-cells", words(2))
        prop("ranges", b"")
        node("missing-reg")
        end()
        end()
    end()
    structure.extend(words(9))
    rsv = b"".join(pairs(*r) for r in reservations) + bytes(16)
    off_structure = 40 + len(rsv)
    off_strings = off_structure + len(structure)
    return words(0xd00dfeed, off_strings + len(strings), off_structure,
                 off_strings, 40, 17, 16, 0, len(strings), len(structure)) + rsv + structure + strings


def symbols(elf):
    text = subprocess.check_output(["aarch64-linux-gnu-nm", "-n", str(elf)], text=True)
    return {line.split()[2]: int(line.split()[0], 16) for line in text.splitlines()
            if len(line.split()) == 3 and line.split()[2].startswith("__p1_")}


def trace_reader(fifo, archive, lower, upper, root, result):
    try:
        active = False
        smallest = upper
        count = 0
        outside = []
        pc = None
        minimum_pc = None
        with open(fifo, "rb") as source, gzip.open(archive, "wb", compresslevel=1) as out:
            pending = b""
            pattern = re.compile(rb"(PC|SP)=([0-9a-fA-F]+)")
            while chunk := source.read(65536):
                out.write(chunk)
                combined = pending + chunk
                boundary = combined.rfind(b"\n")
                if boundary < 0:
                    raise ValueError("unbounded trace line")
                complete, pending = combined[:boundary + 1], combined[boundary + 1:]
                for match in pattern.finditer(complete):
                    value = int(match[2], 16)
                    if match[1] == b"PC":
                        pc = value
                        if pc == root:
                            active = True
                        continue
                    if not active:
                        continue  # Before the W03 adapter is entered.
                    sp = value
                    count += 1
                    if sp < smallest:
                        smallest, minimum_pc = sp, pc
                    if not lower <= sp <= upper and len(outside) < 8:
                        outside.append({"pc": pc, "sp": sp})
        result.update(samples=count, minimum_sp=smallest, minimum_pc=minimum_pc,
                      stack_start=lower, stack_end=upper, used_bytes=upper - smallest,
                      headroom_bytes=smallest - lower, outside=outside,
                      granularity="one instruction per TB; cpu,nochain; all execution after W03 entry breakpoint")
    except Exception as error:
        result["error"] = str(error)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", type=Path, required=True)
    parser.add_argument("--elf", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--trace", action="store_true")
    parser.add_argument("--case", default="all")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    syms = symbols(args.elf)
    listing, audit = analyze(args.elf)
    (args.output / "disassembly.txt").write_text(listing)
    write_json(args.output / "frames.json", audit)
    full_ram = [(0x40000000, 0x8000000)]
    eight_banks = [(0x40000000 + i * 0x1000000, 0xfff000) for i in range(8)]
    loader_reservation = [(0x43200000, 4096)]
    rsv = [(0x41001000 + i * 0x2000, 13) for i in range(31)] + loader_reservation
    cases = [
        ("canonical", None, None),
        ("repeat", None, None),
        ("multibank-capacity", fixture(eight_banks, rsv), None),
        ("image-conflict", fixture(full_ram, [(syms["__p1_boot_code_start"], 4096)] + loader_reservation), "ProtectionConflict"),
        ("ram-overlap", fixture(full_ram + [(0x41000000, 4096)], loader_reservation), "RamOverlap"),
        ("unusable-reservation", fixture(full_ram, loader_reservation, bad_reserved=True), "UnusableFact"),
    ]
    results = []
    for name, dtb, rejection in cases:
        if args.case not in ("all", name):
            continue
        directory = args.output / name
        directory.mkdir()
        command, _, _, forbidden = profile("p1-boot-smoke", ["boot-smoke=" + str(args.image.resolve())])
        command[command.index("-machine") + 1] += ",gic-version=3"
        if dtb is not None:
            path = directory / "input.dtb"
            path.write_bytes(dtb)
            # QEMU's -dtb boot path rewrites RAM nodes. Use an explicit tiny
            # fixture loader so the hostile/multibank bytes reach W01 intact.
            # Both placements remain inside W01's existing trusted envelope;
            # the trampoline page is declared reserved in every fixture.
            assembly = directory / "loader.S"
            assembly.write_text(".text\nldr x0, 1f\nmov x1, xzr\nmov x2, xzr\nmov x3, xzr\nldr x4, 2f\nbr x4\n1: .quad 0x43000000\n2: .quad 0x40080000\n")
            obj = directory / "loader.o"
            binary = directory / "loader.bin"
            subprocess.run(["aarch64-linux-gnu-as", str(assembly), "-o", str(obj)], check=True)
            subprocess.run(["aarch64-linux-gnu-objcopy", "-O", "binary", str(obj), str(binary)], check=True)
            command += ["-device", f"loader,file={path.resolve()},addr=0x43000000,force-raw=on",
                        "-device", f"loader,file={binary.resolve()},addr=0x43200000,force-raw=on",
                        "-device", "loader,addr=0x43200000,cpu-num=0"]
        if rejection:
            required = (b"ZELYR P2 DISCOVERY complete", f"ZELYR P2 MAP REJECT {rejection}".encode())
            forbidden += (b"ZELYR P2 MAP sealed", b"ZELYR P2 REJECT")
        else:
            required = (b"ZELYR P2 MAP sealed", b"ZELYR P2 MAP records")
            forbidden += (b"ZELYR P2 MAP REJECT", b"ZELYR P2 REJECT", b"[truncated]")
        trace = {}
        thread = None
        debugger = None
        debug_result = {}
        if args.trace:
            fifo = directory / "trace.fifo"
            os.mkfifo(fifo)
            thread = threading.Thread(target=trace_reader,
                                      args=(fifo, directory / "cpu.log.gz", syms["__p1_stack_start"],
                                            syms["__p1_stack_end"], audit["root"], trace), daemon=True)
            thread.start()
            socket_path = directory / "gdb.sock"
            command += ["-accel", "tcg,one-insn-per-tb=on", "-D", str(fifo.resolve()),
                        "-S", "-gdb", f"unix:{socket_path.resolve()},server=on,wait=off"]
            debugger = threading.Thread(target=enable_at_entry,
                                        args=(socket_path, audit["root"], debug_result), daemon=True)
            debugger.start()
        result = capture(command, directory, 55 if args.trace else 8, required, forbidden)
        if audit["owning_map_entries"] or audit["reviewed_envelope_bytes"] + 8192 > syms["__p1_stack_end"] - syms["__p1_stack_start"]:
            result.update(status=4, reason="static-stack-budget")
        if thread:
            debugger.join(timeout=5)
            thread.join(timeout=5)
            trace["debugger"] = debug_result
            fifo.unlink()
            if thread.is_alive() or debugger.is_alive() or debug_result.get("error") or not debug_result.get("logging_enabled") or trace.get("error") or not trace.get("samples") or trace.get("outside"):
                result.update(status=4, reason="invalid-or-out-of-bounds-stack-trace")
            elif trace["headroom_bytes"] < 8192:
                result.update(status=4, reason="less-than-8KiB-stack-headroom")
            write_json(directory / "stack.json", trace)
        serial = (directory / "serial.log").read_text()
        if not rejection and result["status"] == 0:
            match = re.search(r"MAP sealed ram=(\d+) allocatable=(\d+) protected=(\d+) metadata=0 allocator=absent", serial)
            expected_ram = sum(length for _, length in (eight_banks if name == "multibank-capacity" else full_ram)) // 4096
            if not match or int(match[1]) != expected_ram or int(match[1]) != int(match[2]) + int(match[3]):
                result.update(status=4, reason="RAM-accounting-mismatch")
            # Independently derive protected-page union for supplied sources.
            bounds = re.search(r"MAP bounds image=(0x[0-9a-f]+)\+(0x[0-9a-f]+) dtb=(0x[0-9a-f]+)\+(0x[0-9a-f]+)", serial)
            if bounds and match:
                image_base, image_len, dtb_base, dtb_len = (int(v, 16) for v in bounds.groups())
                protected = set()
                banks = eight_banks if name == "multibank-capacity" else full_ram
                for base, length in [(image_base, image_len), (dtb_base, dtb_len)] + (rsv if name == "multibank-capacity" else []):
                    for page in range(base // 4096, (base + length + 4095) // 4096):
                        if any(b <= page * 4096 < b + n for b, n in banks):
                            protected.add(page)
                if image_base != syms["__p1_boot_code_start"] or image_base + image_len != syms["__p1_stack_end"] or len(protected) != int(match[3]):
                    result.update(status=4, reason="protection-or-image-bound-mismatch")
            else:
                result.update(status=4, reason="missing-bounds")
        result.update(name=name, command=command, timestamp=stamp(), image=image_identity(args.image),
                      elf=image_identity(args.elf), stack=trace)
        write_json(directory / "result.json", result)
        results.append(result)
        print(name, result["status"], result["reason"], trace, flush=True)
    if not results:
        parser.error("unknown case")
    write_json(args.output / "summary.json", results)
    return int(any(result["status"] != 0 for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
