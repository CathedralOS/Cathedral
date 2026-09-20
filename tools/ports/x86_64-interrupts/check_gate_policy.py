#!/usr/bin/env python3
"""Ensure modernization retained the canonical fields/constants/seven placements."""
import re,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
old=subprocess.check_output(['git','show','d4b8fa5aae189e9ee10768a1e6de5c1370fb5dcb:source/drivers/facts/x86_idt_gate.omg'],cwd=ROOT,text=True)
new=(ROOT/'source/drivers/facts/x86_idt_gate.omg').read_text();plan=(ROOT/'source/drivers/facts/x86_idt_gate_layout.omg').read_text()
def clean(s):return re.sub(r'\s+','',re.sub(r'//[^\n]*','',s))
def fields(s):return re.search(r'pub data X86IdtGate(?: \[copy\])?\s*\{(.*?)\n\}',s,re.S).group(1)
assert clean(fields(old))==clean(fields(new)),'canonical gate fields changed'
assert re.findall(r'pub const .*?;',old,re.S)==re.findall(r'pub const .*?;',new,re.S),'gate constants changed'
for index in range(7):
 pattern=r'entries\['+str(index)+r'\] = FieldEntry \{(.*?)\n    \};'
 a=re.search(pattern,old,re.S).group(1);b=re.search(pattern,plan,re.S).group(1)
 assert clean(a)==clean(b),(index,'placement changed')
for field,value in [('entry_count','7'),('size_fixed','16'),('size_is_dynamic','false'),('align','16')]:
 assert re.search(field+r':\s*'+value+r'\b',old) and re.search(field+r':\s*'+value+r'\b',plan)
print('PASS canonical gate fields/constants and seven selected16-byte/align16 placements unchanged')
