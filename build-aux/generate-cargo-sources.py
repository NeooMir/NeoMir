#!/usr/bin/env python3
"""
Zero-dependency Flatpak Cargo source generator for NeoMir.
Parses Cargo.lock and generates cargo-sources.json for offline Flatpak builds.
"""

import json
import os
import sys
import tomllib

def generate_sources(cargo_lock_path: str, output_path: str):
    with open(cargo_lock_path, "rb") as f:
        lock_data = tomllib.load(f)

    packages = lock_data.get("package", [])
    sources = []

    # 1. Vendor config for Cargo
    cargo_config = (
        "[source.crates-io]\n"
        "replace-with = \"vendored-sources\"\n\n"
        "[source.vendored-sources]\n"
        "directory = \"cargo/vendor\"\n"
    )

    sources.append({
        "type": "inline",
        "contents": cargo_config,
        "dest": "cargo",
        "dest-filename": "config.toml"
    })

    # 2. Add each crate
    for pkg in packages:
        name = pkg.get("name")
        version = pkg.get("version")
        checksum = pkg.get("checksum")

        # Skip local package (neomir itself) or packages without checksum
        if not checksum or name == "neomir":
            continue

        crate_url = f"https://static.crates.io/crates/{name}/{name}-{version}.crate"
        dest_dir = f"cargo/vendor/{name}-{version}"

        sources.append({
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": crate_url,
            "sha256": checksum,
            "dest": dest_dir
        })

        # Checksum file required by cargo vendor
        checksum_content = json.dumps({"package": checksum, "files": {}})
        sources.append({
            "type": "inline",
            "contents": checksum_content,
            "dest": dest_dir,
            "dest-filename": ".cargo-checksum.json"
        })

    with open(output_path, "w", encoding="utf-8") as out:
        json.dump(sources, out, indent=2, ensure_ascii=False)

    print(f"==> Успешно сгенерирован {output_path} ({len(sources)} записей)")

if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.dirname(script_dir)
    lock_file = os.path.join(root_dir, "Cargo.lock")
    out_file = os.path.join(root_dir, "cargo-sources.json")

    if len(sys.argv) > 1:
        lock_file = sys.argv[1]
    if len(sys.argv) > 2:
        out_file = sys.argv[2]

    generate_sources(lock_file, out_file)
