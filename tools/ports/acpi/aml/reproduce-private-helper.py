#!/usr/bin/env python3
"""Reproduce the observed private helper spelling failure on an isolated AML copy."""
import argparse
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[4]
HERE=Path(__file__).resolve().parent

def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 args=parser.parse_args()
 with tempfile.TemporaryDirectory(prefix='cathedral-aml-private-helper-') as directory:
  folder=Path(directory);package=folder/'aml';shutil.copytree(ROOT/'source/libraries/acpi/aml',package)
  source=package/'references.omg';text=source.read_text()
  text=re.sub(r'\breference_scan\(', 'scan(',text)
  text=re.sub(r'\breference_step\(', 'step(',text)
  source.write_text(text)
  (folder/'main.omg').write_text((HERE/'cases/reference-cycle.omg').read_text())
  build=(HERE/'build.omg').read_text().replace('../../../../source/libraries/acpi/aml',str(package))
  (folder/'build.omg').write_text(build)
  result=subprocess.run([str(args.omega.resolve()),'--check',str(folder/'main.omg')],cwd=ROOT,capture_output=True,text=True)
  output=result.stdout+result.stderr
  print(output,end='')
  if result.returncode==0 or 'no field `absolute` on `ReferenceState`' not in output:
   raise SystemExit('The recorded source-form failure did not reproduce; investigate before claiming the compiler issue persists.')
  print('Observed failure reproduced by changing only private helper spellings; production source retains module-specific names.')
if __name__=='__main__':main()
