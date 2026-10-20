#!/usr/bin/env python3
"""Summarize buffered kernel-phase observations from a smoke serial log.

Phases overlap. Do not sum them or interpret observed maxima as latency bounds.
The five periodic cases are shown separately; --all includes the other sessions.
"""
import argparse
from pathlib import Path
import re

PHASE = re.compile(
    r"KERNEL_PHASE phase=(\w+) count=(\d+) total_us=(\d+) max_us=(\d+) over_1ms=(\d+)"
)
CASE = re.compile(r"CPU timing: cap_us=(\d+) yielding_peer=(\d+)")
NAMES = {0: "busy", 1: "yielding", 2: "one-page objects", 3: "128-page objects"}


def summarize(text, include_all=False):
    groups = {}
    case = None
    for line in text.splitlines():
        match = CASE.search(line)
        if match:
            cap, peer = map(int, match.groups())
            case = f"{NAMES[peer]}, cap={cap} us"
        if "CPU timing: periodic=" in line:
            case = None
        match = PHASE.search(line)
        if not match or (case is None and not include_all):
            continue
        phase, *values = match.groups()
        count, total, maximum, over = map(int, values)
        sample = groups.setdefault(case or "other sessions", {}).setdefault(phase, [0, 0, 0, 0])
        sample[0] += count
        sample[1] += total
        sample[2] = max(sample[2], maximum)
        sample[3] += over
    return groups


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("--all", action="store_true", help="also aggregate other task sessions")
    args = parser.parse_args()
    raw = args.log.read_bytes()
    text = raw.decode("utf-16" if raw.startswith((b"\xff\xfe", b"\xfe\xff")) else "utf-8", errors="replace")
    groups = summarize(text, args.all)
    if not groups:
        parser.error("no matching observations; use the smoke serial log (or --all)")
    for case, phases in groups.items():
        print(f"\n{case}")
        print(f"{'phase':18} {'count':>9} {'mean us':>12} {'max us':>12} {'>=1 ms':>9}")
        for phase, (count, total, maximum, over) in sorted(phases.items(), key=lambda item: -item[1][2]):
            print(f"{phase:18} {count:9} {total / count:12.2f} {maximum:12} {over:9}")
    print("\nOverlapping elapsed observations, including host descheduling and probe overhead; no WCET bound.")


if __name__ == "__main__":
    main()
