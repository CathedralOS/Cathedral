#!/usr/bin/env python3
from pathlib import Path
from source_fragments import append_fragment
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
s=(ROOT/'source/platform/libraries/x86_64/interrupt_bytes.omg').read_text().replace('use interrupts.omg;', 'use {x86_values}/interrupts.omg;')
s=append_fragment(s, (HERE/'table_bridge.template.omg').read_text(), local_alias='table_impl')
p=HERE/'table_impl.omg'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=s:raise SystemExit('private production scan fixture differs')
else:p.write_text(s)
