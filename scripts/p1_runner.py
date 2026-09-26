"""P1 single QEMU process owner and bounded regression policy (Python stdlib)."""

import argparse
import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import selectors
import signal
import subprocess
import sys
import time
import uuid

VERSION = "0.1"
ROOT = Path(__file__).resolve().parent.parent
DEFAULT_IMAGE = ROOT / "target/p1/hypervisor-boot.img"
DEFAULT_TIMEOUT = 8.0
OBSERVE = 0.2
CAPTURE_LIMIT = 1024 * 1024
LINE_LIMIT = 4096
STABLE = b"ZELYR P1 STABLE"
START = (b"ZELYR P1 PHASE entry", b"ZELYR P1 PHASE runtime")
FORBIDDEN = (b"ZELYR P1 PANIC", b"ZELYR P1 FATAL", b"ZELYR P1 BOOT REJECT")
MARKER_CONTROL_SHA256 = "399114e99cddcdf6686e2b04d8632dbc0409be3628aa22098a07caf3a6bd685b"
RESERVED = {"boot-smoke", "smp", "memory", "gic", "smmu", "guest-image", "regression"}


class UsageError(Exception):
    """Invalid contract invocation (P0 status 1)."""


class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise UsageError(message)


def duration(value):
    try:
        result = float(value.removesuffix("s"))
    except ValueError as error:
        raise UsageError("timeout must be positive finite seconds") from error
    if not math.isfinite(result) or result <= 0:
        raise UsageError("timeout must be positive finite seconds")
    return result


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def fresh_evidence():
    return ROOT / "target/p1-evidence" / ("run-" + uuid.uuid4().hex)


def selected_evidence(argv, option):
    for index, argument in enumerate(argv):
        if argument.startswith(option + "="):
            return Path(argument.split("=", 1)[1]).resolve()
        if argument == option and index + 1 < len(argv):
            return Path(argv[index + 1]).resolve()
    return fresh_evidence()


def image_identity(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(65536), b""):
            digest.update(chunk)
    return {"path": str(path.resolve()), "sha256": digest.hexdigest(), "size": path.stat().st_size}


class Matcher:
    """Streaming membership predicates; never buffers an unbounded line."""

    def __init__(self, required, forbidden):
        self.required = tuple(required)
        self.forbidden = tuple(forbidden)
        self.seen = set()
        self.tail = b""
        self.line_size = 0
        self.total = 0
        self.limit = False
        self.overlap = max(map(len, self.required + self.forbidden), default=1) - 1

    def feed(self, chunk):
        self.total += len(chunk)
        combined = self.tail + chunk
        self.seen.update(token for token in self.required + self.forbidden if token in combined)
        self.tail = combined[-self.overlap:] if self.overlap else b""
        lines = chunk.split(b"\n")
        self.limit |= self.line_size + len(lines[0]) > LINE_LIMIT
        self.limit |= any(len(line) > LINE_LIMIT for line in lines[1:])
        self.line_size = self.line_size + len(chunk) if len(lines) == 1 else len(lines[-1])
        self.limit |= self.total > CAPTURE_LIMIT

    def complete(self):
        return all(token in self.seen for token in self.required)

    def failed(self):
        return any(token in self.seen for token in self.forbidden)

    def predicates(self):
        return {token.decode("ascii"): token in self.seen for token in self.required + self.forbidden}


def capture(command, directory, timeout, required, forbidden, observe=OBSERVE):
    """Own one child, stream both channels, terminate its group and drain pipes.

    Internal command injection is for mechanism tests only. The public runner
    obtains its command exclusively from the fixed profile below.
    """
    matcher = Matcher(required, forbidden)
    process = None
    launching = False
    start = time.monotonic()
    marker_at = None
    reason = "launch"
    status = 2
    try:
        with (directory / "serial.log").open("xb") as serial, (directory / "emulator.log").open("xb") as diagnostic:
            launching = True
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            launching = False
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ, (serial, True))
                selector.register(process.stderr, selectors.EVENT_READ, (diagnostic, False))
                stopped = None
                diagnostic_size = 0
                while selector.get_map():
                    now = time.monotonic()
                    if stopped is None:
                        if now - start >= timeout:
                            status, reason = 3, "timeout"
                        elif matcher.limit or diagnostic_size > CAPTURE_LIMIT:
                            status, reason = 4, "output-limit"
                        elif marker_at is not None and now - marker_at >= observe:
                            status, reason = (4, "forbidden-marker") if matcher.failed() else (0, "observed")
                        else:
                            reason = "running"
                        if reason != "running":
                            stopped = now
                            try:
                                os.killpg(process.pid, signal.SIGTERM)
                            except ProcessLookupError:
                                pass
                    elif now - stopped >= 0.5:
                        # Kill the group even if the leader exited with pipes
                        # held open by a descendant; drain buffered bytes below.
                        try:
                            os.killpg(process.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    for key, _ in selector.select(0.01):
                        chunk = os.read(key.fileobj.fileno(), 65536)
                        if not chunk:
                            selector.unregister(key.fileobj)
                            key.fileobj.close()
                            continue
                        output, is_serial = key.data
                        output.write(chunk)
                        if is_serial:
                            matcher.feed(chunk)
                        else:
                            diagnostic_size += len(chunk)
                    if marker_at is None and (matcher.complete() or matcher.failed()):
                        marker_at = time.monotonic()
                process.wait(timeout=1)
                if stopped is None:
                    status, reason = (4, "early-exit") if matcher.total or diagnostic_size else (2, "silent-exit")
                elif status == 0 and matcher.failed():
                    status, reason = 4, "forbidden-marker"
                elif status == 0 and matcher.limit:
                    status, reason = 4, "output-limit"
    except (FileNotFoundError, PermissionError) as error:
        status, reason = (2, "launch: " + str(error)) if launching else (5, str(error))
    finally:
        if process is not None:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            for stream in (process.stdout, process.stderr):
                if stream and not stream.closed:
                    stream.close()
    return {"status": status, "reason": reason, "elapsed_seconds": time.monotonic() - start,
            "raw_exit": None if process is None else process.returncode, "serial_bytes": matcher.total,
            "predicates": matcher.predicates(), "observation_seconds": observe}


def profile(name, params):
    if name not in ("p1-boot-smoke", "p1-marker-control"):
        raise UsageError("unknown profile: " + name)
    values = {}
    for parameter in params:
        key, separator, value = parameter.partition("=")
        if not separator or key not in RESERVED or key != "boot-smoke" or not value or key in values:
            raise UsageError("unsupported or duplicate parameter: " + parameter)
        values[key] = value
    image = Path(values.get("boot-smoke", str(DEFAULT_IMAGE))).resolve()
    command = ["qemu-system-aarch64", "-machine", "virt,virtualization=on", "-cpu", "cortex-a57",
               "-smp", "1", "-m", "128M", "-display", "none", "-monitor", "none",
               "-serial", "stdio", "-kernel", str(image)]
    if name == "p1-marker-control":
        command += ["-semihosting-config", "enable=on,target=native"]
    return command, image, (STABLE,) + START, FORBIDDEN


def run(argv):
    """P0 v0.1 grammar/status entry; returns status and evidence location."""
    directory = selected_evidence(argv, "--evidence-dir")
    meta = {"contract_version": VERSION, "runner_version": VERSION, "argv": argv,
            "host": platform.platform(), "start": stamp(), "evidence_root": str(directory),
            "success_condition": "P1-W10 marker protocol", "machine": "virt", "artifacts": [],
            "emulator": {"program": "qemu-system-aarch64", "version": "unavailable"},
            "runner_source": image_identity(Path(__file__)), "profile": "unparsed", "parameters": [],
            "timeout_seconds": None}
    outcome = {"status": 5, "reason": "unfinished"}
    created = False
    try:
        directory.mkdir(parents=True, exist_ok=False)
        created = True
        write_json(directory / "invocation.json", meta)
        parser = Parser(add_help=False)
        parser.add_argument("verb", choices=["run"])
        parser.add_argument("--profile", required=True)
        parser.add_argument("--param", action="append", default=[])
        parser.add_argument("--timeout", type=duration, default=DEFAULT_TIMEOUT)
        parser.add_argument("--evidence-dir")
        options = parser.parse_args(argv)
        command, image, required, forbidden = profile(options.profile, options.param)
        meta.update(profile=options.profile, parameters=options.param, timeout_seconds=options.timeout,
                    command=command, required=[x.decode() for x in required], forbidden=[x.decode() for x in forbidden])
        try:
            meta["artifacts"] = [image_identity(image)]
            if options.profile == "p1-marker-control" and meta["artifacts"][0]["sha256"] != MARKER_CONTROL_SHA256:
                raise UsageError("marker-control image identity does not match the reviewed W10 fixture")
            version = subprocess.run([command[0], "--version"], capture_output=True, timeout=2, check=True)
            meta["emulator"]["version"] = version.stdout.decode(errors="replace").splitlines()[0]
        except (OSError, subprocess.SubprocessError) as error:
            outcome = {"status": 2, "reason": str(error)}
        else:
            write_json(directory / "invocation.json", meta)
            outcome = capture(command, directory, options.timeout, required, forbidden)
    except UsageError as error:
        outcome = {"status": 1, "reason": str(error)}
    except (Exception, KeyboardInterrupt) as error:
        outcome = {"status": 5, "reason": str(error)}
    finally:
        if created:
            try:
                (directory / "serial.log").touch(exist_ok=True)
                meta["end"] = stamp()
                write_json(directory / "invocation.json", meta)
                write_json(directory / "outcome.json", outcome)
            except OSError as error:
                outcome = {"status": 5, "reason": str(error)}
    return outcome, directory


def verdict(outcome):
    status = outcome["status"]
    predicates = outcome.get("predicates", {})
    if status in (1, 2, 5):
        return "ERROR-INVOCATION"
    if any(predicates.get(token.decode(), False) for token in FORBIDDEN):
        return "FAIL-PANIC"
    if status == 3:
        return "FAIL-TIMEOUT"
    if outcome.get("reason") == "early-exit":
        if outcome.get("raw_exit") == 0 and not predicates.get(STABLE.decode(), False):
            return "FAIL-MARKER"
        return "FAIL-EXIT"
    if status != 0 or not all(predicates.get(token.decode(), False) for token in (STABLE,) + START):
        return "FAIL-MARKER"
    return "PASS"


def regression(argv):
    directory = selected_evidence(argv, "--evidence")
    created = False
    parser = Parser()
    parser.add_argument("--cycles", type=int, choices=(1, 100), default=1)
    parser.add_argument("--timeout", type=duration, default=DEFAULT_TIMEOUT)
    parser.add_argument("--image", type=Path, default=DEFAULT_IMAGE)
    parser.add_argument("--evidence", type=Path, default=directory)
    parser.add_argument("--marker-control", action="store_true")
    try:
        directory.mkdir(parents=True, exist_ok=False)
        created = True
        write_json(directory / "meta.txt", {"argv": argv, "start": stamp()})
        options = parser.parse_args(argv)
        if options.marker_control and options.cycles != 1:
            raise UsageError("marker control is only valid for one cycle")
        write_json(options.evidence / "meta.txt", {"argv": argv, "start": stamp(), "cycles": options.cycles,
                   "timeout": options.timeout, "retention": "complete captures for every attempted cycle",
                   "profile": "p1-marker-control" if options.marker_control else "p1-boot-smoke"})
        outcomes = []
        for cycle in range(1, options.cycles + 1):
            outcome, directory = run(["run", "--profile",
                    "p1-marker-control" if options.marker_control else "p1-boot-smoke", "--param",
                    "boot-smoke=" + str(options.image), "--timeout", str(options.timeout),
                    "--evidence-dir", str(options.evidence / f"cycle-{cycle:03}")])
            result = verdict(outcome)
            outcomes.append({"cycle": cycle, "outcome": result, "evidence": str(directory)})
            print(f"cycle={cycle:03} {result}", flush=True)
            write_json(options.evidence / "summary.txt", {"requested": options.cycles, "outcomes": outcomes,
                       "passed": sum(row["outcome"] == "PASS" for row in outcomes),
                       "counted": sum(row["outcome"] != "ERROR-INVOCATION" for row in outcomes), "end": stamp()})
            if result != "PASS":
                return 2 if result == "ERROR-INVOCATION" else 1
        return 0
    except (Exception, KeyboardInterrupt) as error:
        if created:
            try:
                write_json(directory / "summary.txt", {"outcome": "ERROR-INVOCATION", "reason": str(error), "end": stamp()})
            except OSError:
                pass
        print(str(error), file=sys.stderr)
        return 2


def main():
    if sys.argv[1:] == ["--version"]:
        print("zelyr-qemu-runner " + VERSION + " entry-contract " + VERSION)
        return 0
    outcome, directory = run(sys.argv[1:])
    print(json.dumps({**outcome, "evidence": str(directory)}))
    return outcome["status"]
