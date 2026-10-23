"""One matrix startup as a timeline from the stamps in a traced serial log.

A matrix writes two trace-calibrate stamps when it starts. From the chosen
start (the last matrix's by default, the deepest of a nested entry) the tool
lists, in milliseconds after it, every request of the matrix's chain that
reached the root provider (root-send, with the provider's end) and every
console line between them, and stops after the first request that follows the
matrix's 'configured from realm paths' line: its launcher naming the tiles,
where the startup ends. Later requests belong to what the user does next.
"""
import re
import sys

STAMP = re.compile(r'STORAGE_TRACE ([a-z-]+) (-?\d+) (-?\d+) (\d+) (\d+) (\d+)$')
OPERATIONS = {1: 'HEAD', 2: 'READ', 3: 'PUBLISH', 4: 'COPY', 5: 'DELETE', 6: 'LIST', 7: 'BULK',
              8: 'PIN', 9: 'RELEASE', 10: 'BULK_PIN', 11: 'COLLECT', 12: 'CLOSE', 13: 'COPY_TREE',
              14: 'METADATA', 15: 'SET_METADATA', 16: 'LOOKUP', 17: 'FILTER', 18: 'CREATE_NAMED',
              19: 'ENTRIES'}
CONFIGURED = 'Cathedral matrix: configured from realm paths'
PREFIX = 'Cathedral Rust lab: '


def operation(code):
    # Matrix stages carry 256 plus the storage opcode; the provider's carry the opcode.
    return OPERATIONS.get(code % 256, str(code))


def timeline(text, start=-1):
    """Rows: (ms, 'root-send' | 'provider-end', operation) or (None, 'console', line)."""
    lines = [line.replace(PREFIX, '').strip() for line in text.splitlines()]
    starts = [index for index, line in enumerate(lines) if 'STORAGE_TRACE trace-calibrate' in line][::2]
    if not starts:
        return []
    origin = None
    configured = False
    rows = []
    for line in lines[starts[start]:]:
        match = STAMP.search(line)
        if match:
            label, code, micros = match.group(1), int(match.group(3)), int(match.group(6))
            if origin is None:
                origin = micros
            if label in ('root-send', 'provider-end'):
                rows.append(((micros - origin) / 1000, label, operation(code)))
                if configured and label == 'provider-end':
                    break
        elif line and 'STORAGE_TRACE' not in line and 'KERNEL_PHASE' not in line:
            rows.append((None, 'console', line))
            configured = configured or line == CONFIGURED
    return rows


def summarize(rows):
    requests = [row for row in rows if row[1] == 'root-send']
    answered = [row for row in rows if row[1] == 'provider-end']
    if not answered:
        return 'no request of this startup reached the provider'
    return (f'{len(requests)} requests reached the provider; the last was answered '
            f'{answered[-1][0]:.1f} ms after the matrix started')


def report(path, start=-1):
    rows = timeline(open(path, encoding='utf-8', errors='replace').read(), start)
    for ms, label, detail in rows:
        if ms is None:
            print(f'          {detail}')
        else:
            print(f'{ms:8.1f}  {label:13} {detail}')
    print(summarize(rows))


if __name__ == '__main__':
    if len(sys.argv) not in (2, 3):
        sys.exit('usage: startup_timeline.py SERIAL_LOG [START_INDEX]')
    report(sys.argv[1], int(sys.argv[2]) if len(sys.argv) == 3 else -1)
