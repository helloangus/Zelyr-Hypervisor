"""Focused regression tests for the QG-DOCS language-edition check."""

import contextlib
import hashlib
import importlib.util
import io
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/check-doc-translations.py"
SPEC = importlib.util.spec_from_file_location("check_doc_translations", SCRIPT)
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def blob_id(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


class TranslationGateTest(unittest.TestCase):
    def run_fixture(self, source_name, source_text, edition_status, mutate_source=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = Path(source_name)
            edition = source.with_name(source.stem + ".zh-CN.md")
            (root / source).parent.mkdir(parents=True)
            original = (source_text + f"[Chinese edition]({edition.name})\n").encode()
            (root / source).write_bytes(original + (b"changed\n" if mutate_source else b""))
            (root / edition).write_text(
                f"**Translation status:** {edition_status}\n"
                f"**Translation source:** [English source]({source.name})\n"
                f"**Source blob:** `{blob_id(original)}`\n"
                "**Authority:** English source governs.\n",
                encoding="utf-8",
            )
            output = io.StringIO()
            with patch.object(CHECKER, "ROOT", root), patch.object(
                CHECKER, "tracked_markdown", return_value={source, edition}
            ), patch.object(sys, "argv", ["check-doc-translations.py"]), contextlib.redirect_stdout(output):
                code = CHECKER.main()
            return code, output.getvalue()

    def test_current_pair_passes(self):
        code, _ = self.run_fixture("docs/development/policy.md", "**Status:** Normative policy.\n", "Current")
        self.assertEqual(code, 0)

    def test_stale_normative_policy_fails(self):
        code, output = self.run_fixture("docs/development/policy.md", "**Status:** Normative policy.\n", "Outdated", True)
        self.assertEqual(code, 1)
        self.assertIn("source changed", output)

    def test_marked_stale_informative_note_passes(self):
        code, _ = self.run_fixture("docs/development/note.md", "**Status:** Informative note.\n", "Outdated", True)
        self.assertEqual(code, 0)

    def test_unmarked_stale_note_fails(self):
        code, _ = self.run_fixture("docs/development/note.md", "**Status:** Informative note.\n", "Current", True)
        self.assertEqual(code, 1)

    def test_approved_design_must_sync(self):
        code, _ = self.run_fixture(
            "docs/stages/p2/implementation/w01/design.md",
            "**Status:** Approved detailed design.\n",
            "Outdated",
            True,
        )
        self.assertEqual(code, 1)

    def test_proposed_design_may_be_marked_outdated(self):
        code, _ = self.run_fixture(
            "docs/stages/p2/implementation/w01/design.md",
            "**Status:** Proposed detailed design.\n",
            "Outdated",
            True,
        )
        self.assertEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
