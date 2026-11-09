"""Collect notices for Rust crates linked into the Linux binary."""

import json
from pathlib import Path
import sys


def linked_crates(metadata):
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    pending = [package["id"] for package in packages.values() if package["name"] == "crest-browser"]
    seen = set()
    linked = []
    while pending:
        package_id = pending.pop()
        if package_id in seen:
            continue
        seen.add(package_id)
        package = packages[package_id]
        if all(target["kind"] in (["proc-macro"], ["custom-build"]) for target in package["targets"]):
            continue
        if package["source"]:
            linked.append(package)
        pending.extend(
            dependency["pkg"]
            for dependency in nodes[package_id]["deps"]
            if any(kind["kind"] is None for kind in dependency["dep_kinds"])
        )
    return sorted(linked, key=lambda package: (package["name"], package["version"]))


for package in linked_crates(json.load(sys.stdin)):
    name = package["name"]
    print(f'{name} {package["version"]} · {package["license"]}')
    print(package.get("repository") or package["source"])
    source = Path(package["manifest_path"]).parent
    notices = {
        path
        for path in source.iterdir()
        if path.is_file() and path.name.upper().startswith(("LICENSE", "COPYING", "NOTICE"))
    }
    if package.get("license_file"):
        notices.add(source / package["license_file"])
    if name in {"cef", "cef-dll-sys"}:
        print("See cef-rs-LICENSE-MIT.txt for the upstream workspace license.\n")
        continue
    if not notices:
        raise SystemExit(f"Missing upstream license text for {name}")
    for path in sorted(notices):
        print(f"\n--- {path.name} ---\n")
        print(path.read_text(encoding="utf-8"))
    print()
