"""Focused pure tests for the W11 paired-verdict parser."""
from importlib.machinery import SourceFileLoader
from pathlib import Path
import types
import unittest


SCRIPT = Path(__file__).with_name("p1-w11-verify")
verifier = types.ModuleType("p1_w11_verify")
verifier.__file__ = str(SCRIPT)
SourceFileLoader(verifier.__name__, str(SCRIPT)).exec_module(verifier)


class ScenarioVerdictTests(unittest.TestCase):
    def test_nc3_requires_raw_undefined_ec_not_unknown_label_alone(self):
        serial = b"\n".join((
            b"ZELYR P1 FATAL kind=E",
            b"ph=fatal-path.complete",
            b"org=current-el-spx cat=sync disp=fatal-syndrome",
            b"esr=0000000002000000 cls=unknown il=true",
            b"pc=00000000400875ac spsr=00000000600003c9",
            b"far=na", b"ZELYR P1 REPORT END"))
        expected = verifier.classify("nc3", serial, {"status": 4, "reason": "forbidden-marker"})
        self.assertTrue(expected["undefined_ec"])
        self.assertTrue(expected["class"])
        wrong_ec = serial.replace(b"02000000", b"96000000")
        self.assertFalse(verifier.classify("nc3", wrong_ec, {"status": 4})["undefined_ec"])

    def test_nc5_requires_translation_class_and_exact_far(self):
        serial = b"\n".join((
            b"ZELYR P1 FATAL kind=E",
            b"ph=stage1.complete",
            b"org=current-el-spx cat=sync disp=fatal-syndrome",
            b"esr=0000000096000006 cls=data-abort-translation il=true",
            b"pc=000000004008c004 spsr=00000000600003c9",
            b"far=0000000050000000", b"ZELYR P1 REPORT END"))
        expected = verifier.classify("nc5", serial, {"status": 4, "reason": "forbidden-marker"})
        self.assertTrue(expected["data_abort_ec"])
        self.assertTrue(expected["far"])
        wrong_far = serial.replace(b"50000000", b"50001000")
        self.assertFalse(verifier.classify("nc5", wrong_far, {"status": 4})["far"])

    def test_continuation_or_missing_end_fails(self):
        lines = ["ZELYR P1 REPORT END", "ZELYR P1 STABLE"]
        self.assertFalse(verifier.lines_after_terminal(lines))
        self.assertFalse(verifier.lines_after_terminal(
            ["ZELYR P1 REPORT END", "ZELYR P1 FATAL kind=E"]))
        self.assertFalse(verifier.lines_after_terminal(["ZELYR P1 FATAL kind=E"]))


if __name__ == "__main__":
    unittest.main()
