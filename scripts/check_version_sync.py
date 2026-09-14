#!/usr/bin/env python3
"""Check that every file naming this project's version names the same one.

A release bump touches a dozen files across six ecosystems, and the ones that
go stale are never the ones anyone is looking at: a benchmarks pom two releases
behind, a lockfile's second version record that `npm version` does not rewrite,
an install snippet a reader copies verbatim. None of them break a build. They
just publish a number that is not true.

Two design choices, both learned the hard way:

*Derived, not listed.* A hand-written table of touchpoints is only as good as
the last person who remembered to extend it, and the file that ships stale is
always the one nobody added. Every candidate path that exists here is checked.

*Parsed, not grepped.* A regex for `<version>` in a pom matches every plugin's
version, and one for `"version"` in a lockfile matches every dependency. Those
belong to other people. Where the format has structure -- JSON, XML -- this
reads the structure and looks only at the fields that are ours.

    python scripts/check_version_sync.py

Exits non-zero on the first disagreement, naming the file and both versions.
"""
from __future__ import annotations

import json
import os
import re
import sys
import xml.etree.ElementTree as ET

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
SEMVER = r"\d+\.\d+\.\d+"
POM_NS = {"m": "http://maven.apache.org/POM/4.0.0"}


def path(*parts: str) -> str:
    return os.path.join(ROOT, *parts)


def read(rel: str) -> str | None:
    full = path(rel)
    if not os.path.isfile(full):
        return None
    with open(full, encoding="utf-8") as handle:
        return handle.read()


def own_versions(rel: str) -> list[str] | None:
    """The versions in `rel` that belong to this project, or None if absent."""
    text = read(rel)
    if text is None:
        return None

    if rel.endswith("package-lock.json"):
        d = json.loads(text)
        # npm records the package's own version twice: at the top level, and in
        # the packages[""] entry. Only the first is what `npm version` rewrites,
        # so the second goes stale on its own. Every other entry is a dependency.
        return [v for v in (d.get("version"), d.get("packages", {}).get("", {}).get("version")) if v]

    if rel.endswith("package.json"):
        d = json.loads(text)
        found = [d["version"]] if "version" in d else []
        # The platform packages are ours and are pinned exactly; a dependency
        # range like "^1.2.3" is someone else's and is left alone.
        for name, spec in (d.get("optionalDependencies") or {}).items():
            if re.fullmatch(SEMVER, spec or ""):
                found.append(spec)
        return found

    if rel.endswith(".csproj"):
        root = ET.fromstring(text)
        return [e.text.strip() for e in root.iter("Version") if e.text]

    if rel.endswith("pom.xml"):
        root = ET.fromstring(text)
        found = []
        # The project's own version, a direct child of <project> -- not the
        # version of every plugin and dependency underneath it.
        own = root.find("m:version", POM_NS)
        if own is not None and own.text:
            found.append(own.text.strip())
        # A dependency on our own groupId carries our version too.
        for dep in root.iterfind(".//m:dependency", POM_NS):
            gid = dep.find("m:groupId", POM_NS)
            ver = dep.find("m:version", POM_NS)
            if gid is not None and ver is not None and (gid.text or "").startswith("org.wickra"):
                if ver.text and re.fullmatch(SEMVER, ver.text.strip()):
                    found.append(ver.text.strip())
        return found

    if rel.endswith("DESCRIPTION"):
        m = re.search(rf"(?m)^Version: ({SEMVER})", text)
        return [m.group(1)] if m else []

    if rel.endswith("CITATION.cff"):
        m = re.search(rf'(?m)^version: "?({SEMVER})"?', text)
        return [m.group(1)] if m else []

    if rel.endswith("pyproject.toml") or rel.endswith("Cargo.toml"):
        m = re.search(rf'(?m)^version = "({SEMVER})"', text)
        return [m.group(1)] if m else []

    return []


CANDIDATES = [
    "Cargo.toml",
    "bindings/python/pyproject.toml",
    "bindings/node/package.json",
    "bindings/node/package-lock.json",
    "bindings/r/DESCRIPTION",
    "bindings/java/pom.xml",
    "examples/java/pom.xml",
    "examples/host/Cargo.toml",
    "examples/embedded/Cargo.toml",
    "CITATION.cff",
]


def csprojs() -> list[str]:
    base = path("bindings", "csharp")
    if not os.path.isdir(base):
        return []
    out = []
    for dirpath, _dirs, files in os.walk(base):
        for f in files:
            if f.endswith(".csproj") and "Tests" not in dirpath:
                out.append(os.path.relpath(os.path.join(dirpath, f), ROOT).replace("\\", "/"))
    return sorted(out)


def main() -> int:
    root_manifest = read("Cargo.toml")
    if root_manifest is None:
        print("no Cargo.toml at the repository root", file=sys.stderr)
        return 2
    m = re.search(rf'(?m)^version = "({SEMVER})"', root_manifest)
    if not m:
        print("Cargo.toml declares no workspace version", file=sys.stderr)
        return 2
    want = m.group(1)

    checked = failures = absent = 0
    for rel in CANDIDATES + csprojs():
        found = own_versions(rel)
        if found is None:
            absent += 1
            continue
        if not found:
            continue
        checked += 1
        wrong = sorted(set(v for v in found if v != want))
        if wrong:
            failures += 1
            print(f"  {rel}: expected {want}, found {', '.join(wrong)}")
        else:
            print(f"  {rel}: {want} ({len(found)}x)")

    print()
    if failures:
        print(f"version {want}: {failures} file(s) disagree", file=sys.stderr)
        return 1
    print(f"version {want}: {checked} file(s) agree, {absent} not present in this repository")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
