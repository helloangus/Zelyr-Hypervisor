"""Reproduce bounded W01/W02 smoke evidence using the existing QEMU owner.

This is a package evidence recipe, not W09's full regression entry.
"""
import argparse
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts"))
from p1_runner import capture, image_identity, profile, stamp, write_json  # noqa: E402


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    results = []
    for name, gic, cpus, memory, reject in (
        ("canonical", None, "1", "128M", False),
        ("gic3", "3", "1", "128M", False),
        ("gic3-four-cpus", "3", "4", "128M", False),
        ("outside-bootstrap-envelope", "3", "1", "256M", True),
    ):
        command, _, _, _ = profile("p1-boot-smoke", ["boot-smoke=" + str(args.image.resolve())])
        command[command.index("-smp") + 1] = cpus
        command[command.index("-m") + 1] = memory
        if gic:
            command[command.index("-machine") + 1] += ",gic-version=" + gic
        forbidden = [b"ZELYR P1 PANIC", b"ZELYR P1 FATAL", b"ZELYR P1 BOOT REJECT"]
        if reject:
            required = [b"ZELYR P1 STABLE", b"ZELYR P2 REJECT reason=DtbUnreachable"]
            forbidden += [b"ZELYR P2 INTAKE complete", b"ZELYR P2 DISCOVERY complete"]
        else:
            state = "usable" if gic else "unsupported"
            required = [b"ZELYR P1 STABLE", b"ZELYR P2 INTAKE complete",
                        f"ZELYR P2 DISCOVERY complete cpus={cpus} banks=1 boot=0".encode(),
                        f"ZELYR P2 FACTS gic={state} timer=usable psci=usable".encode()]
            forbidden += [b"ZELYR P2 REJECT"]
        directory = args.output / name
        directory.mkdir()
        result = capture(command, directory, 8, tuple(required), tuple(forbidden))
        result.update(name=name, command=command, timestamp=stamp(), image=image_identity(args.image))
        write_json(directory / "result.json", result)
        results.append(result)
        print(name, result["status"], result["reason"])
    write_json(args.output / "summary.json", results)
    return int(any(result["status"] != 0 for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
