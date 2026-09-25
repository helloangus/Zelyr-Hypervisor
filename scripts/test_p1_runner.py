"""Tool-mechanism tests, not EL2 or hardware evidence."""
import itertools
import json
from pathlib import Path
import sys
import tempfile
import unittest

import p1_runner as runner


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)

    def child(self, code, timeout=1, observe=0.03):
        output = self.root / "capture"
        output.mkdir()
        result = runner.capture([sys.executable, "-c", code], output, timeout,
                                (b"READY",), (b"BAD",), observe)
        return result, (output / "serial.log").read_bytes()

    def test_complete_capture_and_observed_terminal_state(self):
        result, capture = self.child("import os,time; os.write(1,b'READY\\n'); time.sleep(5)")
        self.assertEqual(result["status"], 0)
        self.assertEqual(capture, b"READY\n")
        self.assertIsNotNone(result["raw_exit"])

    def test_late_fatal_cannot_pass_after_stable(self):
        result, capture = self.child("import os,time; os.write(1,b'READY\\n'); time.sleep(.01); os.write(1,b'BAD\\n'); time.sleep(5)")
        self.assertEqual(result["status"], 4)
        self.assertIn(b"BAD", capture)

    def test_timeout_even_with_stable_before_observation_finishes(self):
        result, _ = self.child("import os,time; os.write(1,b'READY'); time.sleep(5)", timeout=.08, observe=.5)
        self.assertEqual(result["status"], 3)
        self.assertLess(result["elapsed_seconds"], 1.5)

    def test_unexpected_exit_after_success_token_is_failure(self):
        result, _ = self.child("print('READY')", observe=.5)
        self.assertEqual(result["status"], 4)
        self.assertEqual(result["reason"], "early-exit")

    def test_silent_exit_is_launch_failure(self):
        result, _ = self.child("pass")
        self.assertEqual(result["status"], 2)

    def test_missing_program_is_launch_failure(self):
        result = runner.capture([str(self.root / "absent")], self.root, 1, (b"READY",), ())
        self.assertEqual(result["status"], 2)

    def test_output_limit_retains_full_received_line(self):
        result, capture = self.child("import os,time; os.write(1,b'x'*5000); time.sleep(5)")
        self.assertEqual(result["status"], 4)
        self.assertEqual(result["reason"], "output-limit")
        self.assertEqual(len(capture), 5000)

    def test_stderr_cannot_forge_target_success(self):
        result, capture = self.child("import os,time; os.write(2,b'READY'); time.sleep(5)", timeout=.1)
        self.assertEqual(result["status"], 3)
        self.assertEqual(capture, b"")

    def test_termination_drains_last_serial_bytes(self):
        result, capture = self.child("import os,signal,time; signal.signal(signal.SIGTERM,lambda *a:(os.write(1,b'tail'),exit(0))); os.write(1,b'READY'); time.sleep(5)")
        self.assertEqual(result["status"], 0)
        self.assertEqual(capture, b"READYtail")

    def test_invalid_grammar_writes_usage_evidence(self):
        directory = self.root / "usage"
        result, _ = runner.run(["run", "--profile", "p1-boot-smoke", "--param", "smp=2", "--evidence-dir", str(directory)])
        self.assertEqual(result["status"], 1)
        self.assertEqual(json.loads((directory / "outcome.json").read_text())["status"], 1)
        self.assertTrue((directory / "serial.log").exists())

    def test_existing_evidence_is_never_overwritten(self):
        sentinel = self.root / "serial.log"
        sentinel.write_bytes(b"original")
        result, _ = runner.run(["run", "--profile", "p1-boot-smoke", "--evidence-dir", str(self.root)])
        self.assertEqual(result["status"], 5)
        self.assertEqual(sentinel.read_bytes(), b"original")

    def test_equals_form_respects_evidence_root_on_usage_error(self):
        directory = self.root / "usage"
        result, actual = runner.run(["run", "--profile=unknown", "--evidence-dir=" + str(directory)])
        self.assertEqual(result["status"], 1)
        self.assertEqual(actual, directory)
        self.assertTrue((directory / "outcome.json").exists())

    def test_clean_exit_missing_marker_is_marker_control(self):
        self.assertEqual(runner.verdict({"status": 4, "reason": "early-exit", "raw_exit": 0, "predicates": {}}), "FAIL-MARKER")

    def test_driver_invalid_grammar_preserves_evidence(self):
        directory = self.root / "driver"
        status = runner.regression(["--cycles", "2", "--evidence", str(directory)])
        self.assertEqual(status, 2)
        self.assertEqual(json.loads((directory / "summary.txt").read_text())["outcome"], "ERROR-INVOCATION")

    def test_invalid_timeout_domain(self):
        for value in ("0", "-1", "nan", "inf", "garbage"):
            with self.subTest(value=value), self.assertRaises(runner.UsageError):
                runner.duration(value)

    def test_chunk_boundaries_and_line_permutations(self):
        lines = [runner.STABLE, *runner.START, b"benign noise"]
        for order in itertools.permutations(lines):
            matcher = runner.Matcher((runner.STABLE,) + runner.START, runner.FORBIDDEN)
            for byte in b"\n".join(order):
                matcher.feed(bytes([byte]))
            self.assertTrue(matcher.complete())
            self.assertFalse(matcher.failed())
            self.assertEqual(runner.verdict({"status": 0, "predicates": matcher.predicates()}), "PASS")


if __name__ == "__main__":
    unittest.main()
