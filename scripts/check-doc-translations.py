#!/usr/bin/env python3
"""Check language-edition metadata and report translation coverage."""

import argparse
import hashlib
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADR000 = Path("docs/adr/adr-000-architecture-baseline-v0.1.md")
SUFFIXES = (".zh-CN.md", ".en.md")
SOURCE_RE = re.compile(r"\*\*Translation source:\*\* \[[^]]+\]\(([^)#]+)\)")
BLOB_RE = re.compile(r"\*\*Source blob:\*\* `([0-9a-f]{40})`")
STATUS_RE = re.compile(r"\*\*Translation status:\*\* (Current|Outdated)")
AUTHORITY_RE = re.compile(r"\*\*Authority:\*\* ")


def tracked_markdown():
    output = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=ROOT)
    return {Path(name.decode()) for name in output.split(b"\0") if name.endswith(b".md")}


def is_translation(path):
    return path.name.endswith(SUFFIXES)


def source_for(path):
    if path.name.endswith(".zh-CN.md"):
        return path.with_name(path.name.removesuffix(".zh-CN.md") + ".md")
    if path == ADR000.with_name(ADR000.stem + ".en.md"):
        return ADR000
    return None


def critical(path):
    parts = path.parts
    if path in (Path("AGENTS.md"), Path("README.md"), Path("docs/README.md")):
        return True
    if len(parts) < 3 or parts[0] != "docs":
        return False
    if parts[1] == "adr":
        return True
    header = (ROOT / path).read_text(encoding="utf-8")[:1500].lower()
    if parts[1] in ("development", "security", "testing", "platform", "abi", "machine-types", "architecture"):
        return any(f"**status:** {word}" in header for word in ("normative", "mandatory", "accepted"))
    if parts[1] == "stages" and len(parts) >= 4:
        if path.name.startswith("task-book-") or path.name.endswith("completion-report.md"):
            return True
        if len(parts) >= 5 and parts[3] == "implementation":
            return "**status:** approved" in header and "design" in header
    return False


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--coverage", action="store_true", help="show untranslated source counts")
    parser.add_argument("--missing", action="store_true", help="list untranslated source paths")
    args = parser.parse_args()
    tracked = tracked_markdown()
    errors = []
    paired = set()
    for edition in sorted(p for p in tracked if is_translation(p)):
        source = source_for(edition)
        if source is None or source not in tracked:
            errors.append(f"{edition}: unsupported translation name or missing source")
            continue
        paired.add(source)
        if source.parts[:2] != ("docs", "adr") and edition.name not in (ROOT / source).read_text(encoding="utf-8"):
            errors.append(f"{source}: missing language-switch link to {edition.name}")
        content = (ROOT / edition).read_text(encoding="utf-8")
        source_match = SOURCE_RE.search(content[:2000])
        blob_match = BLOB_RE.search(content[:2000])
        status_match = STATUS_RE.search(content[:2000])
        if not (source_match and blob_match and status_match and AUTHORITY_RE.search(content[:2000])):
            errors.append(f"{edition}: missing translation metadata")
            continue
        linked = (ROOT / edition.parent / source_match.group(1)).resolve()
        if linked != (ROOT / source).resolve():
            errors.append(f"{edition}: Translation source must link to {source}")
        current_blob = hashlib.sha1(b"blob " + str((ROOT / source).stat().st_size).encode() + b"\0" + (ROOT / source).read_bytes()).hexdigest()
        current = blob_match.group(1) == current_blob
        stated = status_match.group(1)
        if current and stated != "Current":
            errors.append(f"{edition}: source matches; mark translation Current")
        elif not current and (critical(source) or stated != "Outdated"):
            errors.append(f"{edition}: source changed; update critical translation or mark noncritical translation Outdated")
    sources = sorted(p for p in tracked if not is_translation(p) and p.suffix == ".md" and (p.parts[0] == "docs" or p in (Path("AGENTS.md"), Path("README.md"))))
    if args.coverage or args.missing:
        print(f"Translation coverage: {len(paired)}/{len(sources)} sources")
    if args.coverage:
        for group in sorted({p.parts[1] if len(p.parts) > 2 and p.parts[0] == "docs" else "docs-root" if p.parts[0] == "docs" else "root" for p in sources}):
            members = [p for p in sources if (p.parts[1] if len(p.parts) > 2 and p.parts[0] == "docs" else "docs-root" if p.parts[0] == "docs" else "root") == group]
            print(f"  {group}: {sum(p in paired for p in members)}/{len(members)}")
    if args.missing:
        for source in sources:
            if source not in paired:
                print(source)
    if errors:
        for error in errors:
            print(f"QG-DOCS: {error}")
        return 1
    print(f"QG-DOCS: {len(paired)} translation pairs valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
