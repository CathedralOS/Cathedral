#!/usr/bin/env python3
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
s=(ROOT/'source/libraries/x86_64/interrupt_bytes.omg').read_text().replace('module interrupt_bytes;','module table_impl;').replace('use interrupts::GateDecodeResult;','use x86_values::interrupts::GateDecodeResult;')
s+='\n'+(HERE/'table_bridge.template.omg').read_text()
p=HERE/'table_impl.omg'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=s:raise SystemExit('private production scan fixture differs')
else:p.write_text(s)
