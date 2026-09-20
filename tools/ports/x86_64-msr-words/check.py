#!/usr/bin/env python3
"""Exact pinned pure expressions: Rust observations, Omega bodies and controls."""
import argparse,hashlib,subprocess,tempfile,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');args=p.parse_args()
 pin=ROOT/'reference_code/rust-osdev/x86_64';source=pin/'src/registers/model_specific.rs';text=source.read_text()
 assert subprocess.check_output(['git','-C',str(pin),'rev-parse','HEAD'],text=True).strip()=='cc35c876d3badb57df54a66e22f7768a52be95f2'
 assert source.read_bytes()==subprocess.check_output(['git','-C',str(pin),'show','HEAD:src/registers/model_specific.rs'])
 join='((high as u64) << 32) | (low as u64)';low='let low = value as u32;';high='let high = (value >> 32) as u32;'
 for fragment in [join,low,high]:assert text.count(fragment)==1,fragment
 values=sorted(set([0,(1<<64)-1,0x0123456789abcdef,0xfedcba9876543210]+[1<<n for n in range(64)]+[(1<<n)-1 for n in range(1,65)]))
 pairs=[(0,0),(1,2),(0xffffffff,0),(0,0xffffffff),(0xffffffff,0xffffffff),(0x89abcdef,0x01234567)]
 rust=f'fn split(value:u64)->(u32,u32){{{low}{high}(low,high)}}\nfn join(low:u32,high:u32)->u64{{{join}}}\nfn main(){{\n'
 rust+='\n'.join(f'let (low,high)=split({v});println!("split {v} {{}} {{}}",low,high);' for v in values)
 rust+='\n'+'\n'.join(f'println!("join {lo} {hi} {{}}",join({lo},{hi}));' for lo,hi in pairs)+'\n}'
 with tempfile.TemporaryDirectory(prefix='cathedral-msr-words-') as folder:
  folder=Path(folder);(folder/'main.rs').write_text(rust);subprocess.run(['rustc','--edition=2021',str(folder/'main.rs'),'-o',str(folder/'reference')],check=True)
  rows=[line.split() for line in subprocess.check_output([str(folder/'reference')],text=True).splitlines()]
 print(f'PASS {len(values)} split and {len(pairs)} join exact-expression Rust observations',flush=True)
 subprocess.run([sys.executable,str(HERE/'map_inventory.py'),'--check'],cwd=ROOT,check=True)
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 body='use x86::msr_words;\nmachine checks()->i32 {\n';checks=[]
 for n,row in enumerate(rows):
  mode,*nums=row;v,lo,hi=map(int,nums)
  if mode=='split':
   body+=f'let low{n}:u32=msr_words::low_word({v});\nlet high{n}:u32=msr_words::high_word({v});\nlet both{n}:u64=msr_words::combine(low{n},high{n});\n';checks += [f'low{n}=={lo}',f'high{n}=={hi}',f'both{n}=={v}']
  else:
   body+=f'let joined{n}:u64=msr_words::combine({v},{lo});\n';checks+=[f'joined{n}=={hi}']
 body+='transition '+' && '.join(checks)+' { true -> (0) _ -> (1) } }\nconst RESULT:i32=checks();\nmachine require_success(value:i32) requires value == 0; {}\ndata Main{}\nmachine Main::main(&mut self){require_success(RESULT);}\n'
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 for mutation in [None,('low0==0','low0==1'),('high0==0','high0==1')]:
  with tempfile.TemporaryDirectory(prefix='cathedral-msr-words-') as folder:
   folder=Path(folder);(folder/'build.omg').write_text(build);candidate=body
   if mutation:assert body.count(mutation[0])==1;candidate=body.replace(*mutation)
   (folder/'main.omg').write_text(candidate);r=subprocess.run([str(compiler),'--check',str(folder/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if mutation:
    assert r.returncode and 'cannot prove requires contract' in out and '1 == 0' in out,out
   else:assert not r.returncode,out
  print('PASS MSR word body mutation' if mutation else 'PASS Omega split/combine and round trips',flush=True)
if __name__=='__main__':main()
