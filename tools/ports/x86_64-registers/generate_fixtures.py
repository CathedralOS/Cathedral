#!/usr/bin/env python3
from pathlib import Path
import sys
CHECK="--check" in sys.argv
def emit(path,text):
 if CHECK:
  if not path.exists() or path.read_text()!=text:raise SystemExit("generated artifact differs: "+str(path))
 else:path.write_text(text)
import json
HERE=Path(__file__).resolve().parent
measurements=json.loads((HERE/'measurements.json').read_text())
schema=json.loads((HERE/'schema.json').read_text())
scalars={name for name in measurements if name.endswith('_KNOWN_BITS')}|{'MXCSR_RESET_BITS','DR7_VALID_BITS'}
checks=[f'x86_registers::{name}'+('' if name in scalars else '.raw')+' == '+str(row['value']) for name,row in measurements.items()]
lines=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Generated independent numeric expectations; tests real fact bindings/helpers.','module values;','use facts::x86_registers;','use x86_values::registers;']
def groups(label,checks):
 names=[]
 for start in range(0,len(checks),12):
  name=f'{label}_{start//12}';names.append(name+'()')
  lines.append('machine '+name+'() -> bool {\n    '+' &&\n    '.join(checks[start:start+12])+'\n}')
 lines.append('pub machine '+label+'() -> bool { '+' && '.join(names)+' }')
groups('constants_match',checks)
checks=[];base=0x12342bff
for n in range(4):
 for condition in range(4):
  for size in range(4):
   expected=(base&~(15<<(16+4*n)))|(condition<<(16+4*n))|(size<<(18+4*n))
   checks.append(f'registers::dr7_set_size(registers::dr7_set_condition(x86_registers::Dr7Bits {{ raw: {base} }}, {n}, {condition}), {n}, {size}).raw == {expected}')
groups('debug_matrix',checks)
emit(HERE/'values.omg','\n'.join(lines)+'\n')
positive=(HERE/'main.omg').read_text();needle='registers::dr7_size(registers::dr7_set_size(base, 0, 2), 0) == 2';assert positive.count(needle)==1
emit(HERE/'negative.omg',positive.replace(needle,needle[:-1]+'3'))
print('177 fact comparisons and64 exact-preservation debug field combinations authored')
