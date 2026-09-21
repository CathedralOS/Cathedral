#!/usr/bin/env python3
"""Validate retained body evidence and current portable source closure; no rerun."""
import argparse
import hashlib
import json
from pathlib import Path
import check
import generate

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--record',default='checked-verification.json');parser.add_argument('--runner');args=parser.parse_args()
 record=json.loads((check.HERE/args.record).read_text())
 assert record['format']=='cathedral-acpi-pci-routing-checked-v1'
 assert record['omega_revision']==check.OMEGA_REVISION
 assert record['source_sha256']==check.inputs(),'current source/harness closure differs'
 source,selections=generate.render(record['cases'])
 assert record['suite_sha256']==hashlib.sha256(source.encode()).hexdigest()
 assert record['selections']==selections
 assert record['case_count']==record['control_count']==len(selections)//2
 assert record['scope']==('full' if record['cases'] is None else 'selected')
 runner=Path(args.runner or record['runner_path'])
 assert record['runner_sha256']==check.sha(runner),'runner binary differs'
 check.validate_output(record['output'],selections)
 print('PASS',record['scope'],record['case_count'],'positives +',record['control_count'],'controls;',len(record['source_sha256']),'current input hashes')
if __name__=='__main__':main()
