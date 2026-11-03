#!/usr/bin/env python3
"""Check only the handler reachable from the selected inspection entry."""
import re
import sys
from pathlib import Path


def check(path):
    lines = Path(path).read_text().splitlines()
    entry = re.fullmatch(r"terminal selected_machine=timer_entry entry=Some\(Machine\(MachineId\((\d+)\)\)\)", lines[0])
    assert entry, "missing selected timer entry"
    machines = {}
    current = None
    for line in lines:
        header = re.match(r"machine id=machine:(\d+) ", line)
        if header:
            current = int(header[1])
            machines[current] = []
        if current is not None:
            machines[current].append(line)
    pending = [int(entry[1])]
    visited = set()
    operations = []
    while pending:
        identity = pending.pop()
        assert identity not in visited, "timer closure contains recursion or repeated calls"
        visited.add(identity)
        body = machines[identity]
        assert (
            "result=unit services=[" in body[0]
            and re.search(r"services=\[service:\d+:toolchain::PortIo\]$", body[0])
        ), "timer handler gained another service or result"
        parameters = [line for line in body if line.startswith("parameter ")]
        assert len(parameters) == 1 and "multiplicity=Linear" in parameters[0], "acknowledgement must remain linear"
        assert "InterruptAcknowledgement" in parameters[0], "unexpected handler argument"
        for line in body:
            if line.startswith("operation "):
                kind = line.split(" kind=", 1)[1]
                operations.append(kind)
                call = re.match(r"CallUnit callee=machine:(\d+) ", kind)
                if call:
                    assert "transfers=[claim:1->argument:0]" in kind, "call lost the acknowledgement"
                    pending.append(int(call[1]))
                else:
                    assert kind.startswith(("IntegerConstant ", "RegisterAccess ")), "unexpected handler work: " + kind
            if line.startswith("terminator "):
                assert "kind=Jump " in line or "kind=ReturnUnit " in line, "unexpected handler control flow"
    assert len(visited) == 3, "expected caller, timer handler, and PIC completion"
    constants = [kind for kind in operations if kind.startswith("IntegerConstant ")]
    assert constants == ["IntegerConstant { value: Unsigned(32) }"] * 2, "PIC command must be port 0x20, value 0x20"
    writes = [kind for kind in operations if kind.startswith("RegisterAccess ")]
    assert len(writes) == 1 and "event: PortWrite {" in writes[0], "expected exactly one PIC port write"
    print("Timer closure transfers one linear acknowledgement and writes one PIC EOI")


if __name__ == "__main__":
    try:
        check(sys.argv[1])
    except (AssertionError, KeyError, ValueError, IndexError) as error:
        sys.exit("error: " + str(error))
