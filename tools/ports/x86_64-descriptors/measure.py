#!/usr/bin/env python3
"""Observe pinned Rust values/packed layouts and assert them on UEFI x64."""
from pathlib import Path
import argparse,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');args=p.parse_args()
schema=json.loads((HERE/'schema.json').read_text());measurements={}
for c in schema['constants']:
 if c['public']:measurements['DESCRIPTOR_'+c['name']]={'kind':'value','value':c['value'],'expr':'DescriptorFlags::'+c['name']+'.bits()'}
for name,row in schema['records'].items():
 if name=='TssDescriptorWords':continue # Detached pair is Cathedral's encoded hardware words, not Rust enum ABI.
 rust={'GdtEntry':'Entry'}.get(name,name)
 measurements[name+'.size']={'kind':'size','value':row['size'],'expr':'size_of::<'+rust+'>()'}
 measurements[name+'.alignment']={'kind':'alignment','value':row['align'],'expr':'align_of::<'+rust+'>()'}
 for field,_,offset in row['fields']:
  if field.startswith('reserved_') or name=='GdtEntry':continue # Rust private fields, no fabricated visibility.
  measurements[name+'.'+field+'.offset']={'kind':'offset','value':offset,'expr':'offset_of!('+rust+', '+field+')'}
imports='use core::mem::{size_of,align_of,offset_of};\nuse x86_64::structures::{DescriptorTablePointer,tss::TaskStateSegment,gdt::{DescriptorFlags,Entry}};\n'
source=['#[cfg(test)] mod tests;',imports,'fn main() {']
for name,row in measurements.items():source.append('println!("'+name+'={}",'+row['expr']+');')
source+=['}'];(HERE/'src/main.rs').write_text('\n'.join(source)+'\n')
(HERE/'src/lib.rs').write_text('#![no_std]\n'+imports+'\n'.join('const _: () = assert!(('+row['expr']+') as u64 == '+str(row['value'])+');' for row in measurements.values())+'\n')
subprocess.run(['cargo','generate-lockfile','--offline','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,check=True) if not (HERE/'Cargo.lock').exists() else None
output=subprocess.check_output(['cargo','run','--locked','--quiet','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,text=True)
actual={k:int(v) for k,v in (line.split('=',1) for line in output.splitlines())}
assert actual=={k:r['value'] for k,r in measurements.items()},(actual,measurements)
subprocess.run(['cargo','check','--locked','--quiet','--lib','--manifest-path',str(HERE/'Cargo.toml'),'--target','x86_64-unknown-uefi'],cwd=ROOT,check=True)
vector={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'UEFI-x64 numeric descriptors and Rust packed records; not Omega native ABI'},'provenance':{'kind':'upstream','revision':schema['revision'],'sources':schema['files'],'description':'Actual pinned Rust host observations and independent UEFI-x64 const assertions. TSS private reserved offsets come from source/spec/requested plan only and are not presented as Rust offset observations. TssDescriptorWords is an encoded hardware pair, not the native Rust Descriptor enum.'},'measurements':{k:{'kind':row['kind'],'value':actual[k]} for k,row in measurements.items()}}
p=ROOT/'source/drivers/facts/x86_descriptors.vectors.json'
if args.write:p.write_text(json.dumps(vector,indent=2)+'\n')
elif not p.exists() or json.loads(p.read_text())!=vector:raise SystemExit('vectors changed; review --write')
print('PASS',len(measurements),'Rust observations and UEFI-x64 assertions')
