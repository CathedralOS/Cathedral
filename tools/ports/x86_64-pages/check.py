#!/usr/bin/env python3
"""Audit and execute bounded page/frame fixtures, with actual Rust and behavior mutations."""
import argparse,hashlib,re,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega')
 parser.add_argument('--host-only',action='store_true')
 parser.add_argument('--extras-only',action='store_true',help='Validate only the separately authored extra scenarios and their mutation')
 parser.add_argument('--numeric-controls-only',action='store_true',help='Recheck only the two numeric-corpus body mutations')
 args=parser.parse_args()
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/pages-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed')
 run(sys.executable,HERE/'generate.py','--check')
 run('cargo','+nightly-2026-09-04','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 for scope in ['structures::paging::page::tests','structures::paging::frame::tests']:
  run('cargo','+nightly-2026-09-04','test','--quiet','--manifest-path',ROOT/'reference_code/rust-osdev/x86_64/Cargo.toml','--no-default-features','--features','step_trait','--lib',scope)
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 source=(HERE/'main.omg').read_text();extra=(HERE/'extras.omg').read_text()
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64'))
 groups=re.findall(r'^machine (group_\d+)\(',source,re.M)
 def evaluate(selected,mutation=None):
  with tempfile.TemporaryDirectory(prefix='cathedral-x86-pages-') as directory:
   p=Path(directory);additional=extra
   # Keep only selected test bodies: even unused aggregate bodies impose costly
   # proof work. Each original group is compiled and evaluated in its own batch.
   prefix=source[:source.index('machine group_0(')]
   if 'extras::result' not in selected:prefix=prefix.replace('use extras;\n','')
   bodies=re.findall(r'^machine (group_\d+)\(\)[\s\S]*?^}',source,re.M)
   blocks=re.findall(r'^machine group_\d+\(\)[\s\S]*?^}',source,re.M)
   body=prefix+'\n'.join(block for name,block in zip(bodies,blocks) if name in selected)
   body+='\n'+source[source.index('const TEST_RESULT:'):].replace('= test_result();','= selected_result();')
   if mutation:
    filename,old,new=mutation
    if filename=='main.omg':assert body.count(old)==1;body=body.replace(old,new)
    else:assert additional.count(old)==1;additional=additional.replace(old,new)
   statements=[f'    let case{i}: bool = {name}() == 0;' for i,name in enumerate(selected)]
   body+='\nmachine selected_result() -> i32 {\n'+'\n'.join(statements)+'\n    transition '+' && '.join(f'case{i}' for i in range(len(selected)))+' { true -> (0) _ -> (1) }\n}\n'
   (p/'main.omg').write_text(body)
   if 'extras::result' in selected:(p/'extras.omg').write_text(additional)
   (p/'build.omg').write_text(build)
   result=subprocess.run([str(compiler),'--check',str(p/'main.omg')],cwd=ROOT,capture_output=True,text=True)
   output=result.stdout+result.stderr
   if mutation:
    if result.returncode==0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:raise SystemExit('body mutation did not produce expected failure:\n'+output)
   elif result.returncode:raise SystemExit('positive semantic fixture failed:\n'+output)
 for start in ([] if args.extras_only or args.numeric_controls_only else range(len(groups))):
  selected=groups[start:start+1];evaluate(selected);print('PASS semantic groups:',', '.join(selected),flush=True)
 if not args.numeric_controls_only:
  evaluate(['extras::result']);print('PASS range and overflowing-step extras',flush=True)
 controls=[] if args.extras_only else [(['group_0'],('main.omg','equal(result0, 0)','equal(result0, 1)')),(['group_64'],('main.omg','equal(result65, 18446603336221196288)','equal(result65, 18446603336221196289)'))]
 if not args.numeric_controls_only:controls.append((['extras::result'],('extras.omg','let passed13: bool = overflow.overflow;','let passed13: bool = !overflow.overflow;')))
 for selected,mutation in controls:
  evaluate(selected,mutation);print('PASS body mutation rejected:',mutation[1],flush=True)
 print('Selected semantic batches and controls passed; no native/ABI/mapping claims.')
if __name__=='__main__':main()
