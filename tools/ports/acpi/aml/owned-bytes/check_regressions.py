#!/usr/bin/env python3
"""Run existing unchanged suites into fresh storage-migration receipt paths."""
import argparse,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5];HERE=Path(__file__).resolve().parent
SUITES={
 'parser':('tools/ports/acpi/pipeline/check.py',['--parser-regression']),
 'pipeline':('tools/ports/acpi/pipeline/check.py',[]),
 'references':('tools/ports/acpi/aml/object-references/check.py',[]),
 'execution':('tools/ports/acpi/interpreter/execution/check_interpreted.py',[]),
 'fields':('tools/ports/acpi/aml/fields/check_interpreted.py',[]),
 'reference-const':('tools/ports/acpi/aml/object-references/check.py',['--const','--match','rust_single_0_all,zero_budget,self_cycle,package_bad_tail_after_selection,copy_method']),
}
p=argparse.ArgumentParser();p.add_argument('--suite',choices=SUITES,action='append');a=p.parse_args();(HERE/'regressions').mkdir(exist_ok=True)
for name in a.suite or SUITES:
 script,arguments=SUITES[name];subprocess.run(['python3',str(ROOT/script),*arguments,'--record',str(HERE/'regressions'/f'{name}.json')],cwd=ROOT,check=True)
