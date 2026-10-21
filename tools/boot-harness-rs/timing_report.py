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
WAIT_FIELDS = ("completed", "selected", "cancelled", "pending", "total_ready_us",
               "max_ready_us", "due", "total_late_ticks", "max_late_ticks")
WAIT = re.compile(r"WAIT_DISPATCH task=(?P<task>\d+) kind=(?P<kind>\w+) " +
                  " ".join(rf"{field}=(?P<{field}>\d+)" for field in WAIT_FIELDS))


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


def summarize_waits(text, include_all=False):
    groups = {}
    case = None
    for line in text.splitlines():
        match = CASE.search(line)
        if match:
            cap, peer = map(int, match.groups())
            case = f"{NAMES[peer]}, cap={cap} us"
        if "CPU timing: periodic=" in line:
            case = None
        match = WAIT.search(line)
        if not match or (case is None and not include_all):
            continue
        task, kind = int(match["task"]), match["kind"]
        sample = groups.setdefault(case or "other sessions", {}).setdefault(
            (task, kind), dict.fromkeys(WAIT_FIELDS, 0))
        for field in WAIT_FIELDS:
            value = int(match[field])
            sample[field] = max(sample[field], value) if field.startswith("max_") else sample[field] + value
    return groups


def verify_periodic_waits(text):
    """Check producer/consumer agreement, never assert an elapsed-time threshold."""
    case, rows, verified = None, [], 0
    for line in text.splitlines():
        if match := CASE.search(line):
            if case is not None:
                raise ValueError("periodic case ended without its task report")
            case, rows = match.groups(), []
        if case is None:
            continue
        if match := WAIT.search(line):
            if match["task"] == "0" and match["kind"] == "Readiness":
                rows.append({field: int(match[field]) for field in WAIT_FIELDS})
        if "CPU timing: periodic=" not in line:
            continue
        count = re.search(r"periodic_waits=(\d+)", line)
        if count is None:
            raise ValueError(f"periodic case {case} lacks its blocked-wait count")
        blocked = int(count[1])
        if blocked == 0 and not rows:
            verified += 1
            case = None
            continue
        if len(rows) != 1:
            raise ValueError(f"periodic case {case} lacks one complete wait report")
        row = rows[0]
        if (row["completed"] != blocked or row["selected"] != blocked
                or row["due"] != blocked or row["cancelled"] != 0 or row["pending"] != 0):
            raise ValueError(f"periodic case {case} wait counts disagree: blocked={blocked}, {row}")
        if (row["max_ready_us"] > row["total_ready_us"]
                or row["max_late_ticks"] > row["total_late_ticks"]):
            raise ValueError(f"periodic case {case} has inconsistent elapsed counters")
        verified += 1
        case = None
    if case is not None or verified == 0:
        raise ValueError("missing complete periodic observations")
    return verified


def print_waits(rows):
    print("Registered waits: completion to first CPU selection (includes ceiling/peer delays)")
    print(f"{'task/kind':18} {'done':>6} {'picked':>6} {'cancel':>6} {'pending':>7} {'mean us':>10} {'max us':>10} {'due':>6} {'late mean ticks':>16} {'late max ticks':>14}")
    for (task, kind), row in sorted(rows.items()):
        mean = f"{row['total_ready_us'] / row['selected']:.2f}" if row['selected'] else "-"
        maximum = str(row['max_ready_us']) if row['selected'] else "-"
        late = f"{row['total_late_ticks'] / row['due']:.2f}" if row['due'] else "-"
        late_maximum = str(row['max_late_ticks']) if row['due'] else "-"
        print(f"{str(task) + '/' + kind:18} {row['completed']:6} {row['selected']:6} {row['cancelled']:6} {row['pending']:7} {mean:>10} {maximum:>10} {row['due']:6} {late:>16} {late_maximum:>14}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("--all", action="store_true", help="also aggregate other task sessions")
    parser.add_argument("--verify-waits", action="store_true", help="check periodic blocked-wait/selection counts (requires new probe output)")
    args = parser.parse_args()
    raw = args.log.read_bytes()
    text = raw.decode("utf-16" if raw.startswith((b"\xff\xfe", b"\xfe\xff")) else "utf-8", errors="replace")
    groups = summarize(text, args.all)
    waits = summarize_waits(text, args.all)
    if args.verify_waits:
        try:
            verified = verify_periodic_waits(text)
        except ValueError as error:
            parser.error(str(error))
        print(f"PASS: wait/selection counts agree in {verified} periodic cases")
    if not groups:
        parser.error("no matching observations; use the smoke serial log (or --all)")
    for case, phases in groups.items():
        print(f"\n{case}")
        print(f"{'phase':18} {'count':>9} {'mean us':>12} {'max us':>12} {'>=1 ms':>9}")
        for phase, (count, total, maximum, over) in sorted(phases.items(), key=lambda item: -item[1][2]):
            print(f"{phase:18} {count:9} {total / count:12.2f} {maximum:12} {over:9}")
        if "ImageBacking" in phases:
            print("ImageBacking/File/Zero/Publish: per-page attempts inside ImageBatch; File includes any zero tail, Publish includes cache maintenance and mapping.")
        if case in waits:
            print_waits(waits[case])
    print("\nOverlapping elapsed observations, including host descheduling and probe overhead; no WCET bound.")


if __name__ == "__main__":
    main()
