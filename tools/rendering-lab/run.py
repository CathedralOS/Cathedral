#!/usr/bin/env python3
"""Measure the scalar rendering fixture; save CSV and enough context to repeat it."""
import argparse
import csv
import datetime
import io
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "build/rendering-lab")
    args = parser.parse_args()
    command = ["cargo", "run", "--locked", "--release", "-p", "cathedral-rendering-lab",
               "--features", "host", "--bin", "cathedral-rendering-bench"]
    result = subprocess.run(command, cwd=ROOT / "source-rs", check=True,
                            stdout=subprocess.PIPE, text=True, encoding="utf-8")
    rows = list(csv.DictReader(io.StringIO(result.stdout)))
    if len(rows) != 36:
        raise RuntimeError("Incomplete rendering comparison")
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "measurements.csv").write_text(result.stdout, encoding="utf-8", newline="\n")
    context = {
        "utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "platform": platform.platform(), "processor": platform.processor(),
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True).strip(),
        "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "working_tree_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT)),
        "command": command, "samples_per_case": 5,
        "timing": "Median host microseconds: metadata resolution, allocation/zeroing, scalar render and copy. Excludes pixel verification and deallocation.",
        "bytes": "Logical pixel buffer lengths/work, not allocator capacity, RSS or physical memory bus traffic. flat-pages rounds sizes; host storage is not page aligned or MMU isolated.",
    }
    (args.output / "context.json").write_text(json.dumps(context, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(result.stdout, end="")
    print(f"PASS: all 36 outputs verified; measurements and context in {args.output}")


if __name__ == "__main__":
    main()
