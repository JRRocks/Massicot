#!/usr/bin/env python3
"""Writes the list of crates compiled into the Windows executable (name, version, licence,
source) as Markdown, from `cargo metadata`. Usage: dependances.py <output.md>"""

import json
import subprocess
import sys

TARGET = "x86_64-pc-windows-msvc"

meta = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", TARGET]))
packages = {p["id"]: p for p in meta["packages"]}
nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
root = next(p["id"] for p in meta["packages"] if p["name"] == "massicot")

seen, todo = set(), [root]
while todo:
    pid = todo.pop()
    if pid in seen:
        continue
    seen.add(pid)
    for dep in nodes[pid]["deps"]:
        if any(k["kind"] is None for k in dep["dep_kinds"]):
            todo.append(dep["pkg"])
seen.discard(root)

rows = []
for pid in seen:
    p = packages[pid]
    if p.get("source") is None:
        origin = "copie dans ce dépôt (voir ATTRIBUTION.md)"
    else:
        origin = p.get("repository") or "crates.io"
    rows.append((p["name"].lower(), p["name"], p["version"], p.get("license") or "?", origin))
rows.sort()

with open(sys.argv[1], "w", encoding="utf-8") as out:
    out.write("# Bibliothèques compilées dans massicot.exe\n\n")
    out.write(f"{len(rows)} crates (cible {TARGET}). Licences et auteurs : voir chaque dépôt ; "
              "les éléments repris de PdfCraft et de craft-libs sont détaillés dans ATTRIBUTION.md.\n\n")
    out.write("| Crate | Version | Licence | Source |\n|---|---|---|---|\n")
    for _, name, version, licence, origin in rows:
        out.write(f"| {name} | {version} | {licence} | {origin} |\n")
