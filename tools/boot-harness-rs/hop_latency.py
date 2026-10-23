#!/usr/bin/env python3
"""Where a forwarded storage request spends its time, from a traced serial log.

Every STORAGE_TRACE stage line carries the kernel's microsecond clock as its
fifth value, so consecutive stages of one request chain can be timed across
tasks: a matrix's send to its parent's receive is the hop itself (kernel
delivery plus the scheduler waking the parent), receive to admit and admit to
send are the parent's own turn, and the provider's begin to end is its work.
Lines are read in serial order; the lab runs one CPU and one request chain at
a time during startup, so each listed transition pairs a stage with the next
stage of the same chain. Overlapping chains dilute the figures rather than
break them. Each matrix writes two `trace-calibrate` stamps back to back at
startup; the gap between them is what one trace line costs, and the report
shows every stage net of that median as well as raw. Microseconds are
measured, not bounds; logging perturbs scheduling.
"""
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

STAGE = re.compile(r'STORAGE_TRACE ((?:root|nested|app|provider)-(?:send|receive|admit|reply|begin|end)|trace-calibrate) (\d+) (\d+) (\d+) (\d+) (\d+)$')
CALIBRATION = 'trace line itself'
# (previous stage, next stage) -> what the interval measures.
TRANSITIONS = {
    ('trace-calibrate', 'trace-calibrate'): CALIBRATION,
    ('app-send', 'nested-receive'): 'hop up: app to matrix',
    ('app-send', 'root-receive'): 'hop up: app to root',
    ('nested-send', 'nested-receive'): 'hop up: matrix to matrix',
    ('nested-send', 'root-receive'): 'hop up: matrix to root',
    ('root-send', 'provider-begin'): 'hop up: root to provider',
    ('nested-receive', 'nested-admit'): 'matrix dispatch',
    ('root-receive', 'root-admit'): 'root dispatch',
    ('nested-admit', 'nested-send'): 'matrix forward',
    ('root-admit', 'root-send'): 'root forward',
    ('provider-begin', 'provider-end'): 'provider work',
    ('provider-end', 'root-reply'): 'hop down: provider to root',
    ('root-reply', 'nested-reply'): 'hop down: root to matrix',
    ('nested-reply', 'nested-reply'): 'hop down: matrix to matrix',
    ('nested-reply', 'app-reply'): 'hop down: matrix to app',
    ('root-reply', 'app-reply'): 'hop down: root to app',
}


def intervals(text):
    """Microsecond intervals between consecutive stages, grouped by transition."""
    groups = defaultdict(list)
    previous = None
    for line in text.splitlines():
        match = STAGE.search(line)
        if not match:
            continue
        label, micros = match.group(1), int(match.group(6))
        if previous is not None:
            name = TRANSITIONS.get((previous[0], label))
            if name is not None and micros >= previous[1]:
                groups[name].append(micros - previous[1])
        previous = (label, micros)
    return groups


def summarize(groups):
    """Rows by total time; `net_median_us` is the median less the trace line's own cost."""
    calibration = sorted(groups.get(CALIBRATION, []))
    baseline = calibration[len(calibration) // 2] if calibration else 0
    rows = []
    for name, values in sorted(groups.items(), key=lambda item: -sum(item[1])):
        ordered = sorted(values)
        median = ordered[len(ordered) // 2]
        rows.append(dict(stage=name, count=len(values), total_us=sum(values),
                         mean_us=round(statistics.fmean(values)), median_us=median,
                         net_median_us=max(0, median - baseline) if name != CALIBRATION else median,
                         p90_us=ordered[min(len(ordered) - 1, (len(ordered) * 9) // 10)]))
    return rows


def report(path):
    rows = summarize(intervals(Path(path).read_text(encoding='utf-8', errors='replace')))
    print(f"{'stage':30} {'count':>6} {'total ms':>9} {'mean us':>8} {'median':>8} {'net':>8} {'p90':>8}")
    for row in rows:
        print(f"{row['stage']:30} {row['count']:6} {row['total_us'] / 1000:9.1f} {row['mean_us']:8} {row['median_us']:8} {row['net_median_us']:8} {row['p90_us']:8}")
    if not rows:
        print('no five-value stage traces; build with --trace-storage on a runtime that reports the microsecond clock')


if __name__ == '__main__':
    report(sys.argv[1])
