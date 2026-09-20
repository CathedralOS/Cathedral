#!/usr/bin/env python3
"""Reproduce the witnessed unsigned record-operand semantic comparison mismatch."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile

def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 args=parser.parse_args();compiler=args.omega.resolve()
 print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest())
 variants={
  'scalar':('let offset:u64=0;let bound:u64=0xffffffffffffffff;transition offset<=bound {true -> (0) _ -> (1)}',True),
  'direct_record':('let value:Limits=Limits {offset:0,bound:0xffffffffffffffff};transition value.offset<=value.bound {true -> (0) _ -> (1)}',False),
  'staged_record':('let value:Limits=Limits {offset:0,bound:0xffffffffffffffff};let offset:u64=value.offset;let bound:u64=value.bound;transition offset<=bound {true -> (0) _ -> (1)}',True),
 }
 for name,(body,should_pass) in variants.items():
  source='data Limits [copy]{offset:u64;bound:u64;} machine test()->u64{'+body+'} const RESULT:u64=test();machine verify(value:u64) requires value==0;{} data Main{}machine Main::main(&mut self){verify(RESULT);}'
  with tempfile.TemporaryDirectory(prefix='cathedral-u64-'+name+'-') as directory:
   work=Path(directory)
   (work/'build.omg').write_text('machine build(builder:&mut Build){builder.application("u64-comparison-probe");builder.freestanding=true;}')
   (work/'main.omg').write_text(source)
   result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],capture_output=True,text=True)
   output=result.stdout+result.stderr
   if should_pass and result.returncode:raise SystemExit(name+' failed:\n'+output)
   if not should_pass and (result.returncode==0 or '1 == 0' not in output):
    raise SystemExit('Recorded direct-field failure did not reproduce; review the evidence before retaining the issue claim.\n'+output)
   print(name+(': PASS' if should_pass else ': witnessed incorrect comparison result 1'))
 print('No Omega source changed; production parser binds both operands to typed scalar locals.')
if __name__=='__main__':main()
