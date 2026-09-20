#!/usr/bin/env python3
"""Pin-checked register expressions rendered as actual Omega call assertions."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
PREFIX='''use x86::encrypted_register_operands;
use x86::encrypted_register_operands::ObservedCr3;
use x86::register_operands::ApicObservation;
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
use x86::addresses::NumberResult;
use facts::x86_registers::Cr3Flags;
use facts::x86_registers::ApicBaseFlags;
use facts::x86_tlb_operands::Pcid;
machine matches_profile_number(result:NumberResult, valid:bool, expected:u64)->bool {
 transition result { NumberResult::Value {value} -> (valid && value==expected) _ -> (!valid) }
}
machine matches_profile_cr3(result:ObservedCr3, valid:bool, expected:u64, low:u16)->bool {
 transition result { ObservedCr3::Value {observation} -> (valid && observation.frame==expected && observation.low==low) _ -> (!valid) }
}
'''
TAIL='''const RESULT:i32=scenario();
machine require_success(value:i32) requires value==0; {}
data Main{}
machine Main::main(&mut self){require_success(RESULT);}
'''
def main():
 control=(ROOT/'reference_code/rust-osdev/x86_64/src/registers/control.rs').read_text();msr=(ROOT/'reference_code/rust-osdev/x86_64/src/registers/model_specific.rs').read_text()
 for fragment in ['let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);','(frame, (value & 0xFFF) as u16)','let value = ((top_bit as u64) << 63) | addr.as_u64() | val as u64;']:assert fragment in control,fragment
 for fragment in ['let addr = PhysAddr::new_truncate(raw);','let reserved = old_flags & !(ApicBaseFlags::all().bits());','let new_flags = reserved | flags.bits();','msr.write(flags | addr.as_u64());']:assert fragment in msr,fragment
 rows=[json.loads(line) for line in (HERE/'reference.jsonl').read_text().splitlines()];assert len(rows)==100
 folder=HERE/'cases';folder.mkdir(exist_ok=True)
 for start in range(0,100,10):
  group=rows[start:start+10];body=PREFIX
  for n,row in enumerate(group):
   raw=row['raw'];body+=f'machine row{n}()->bool {{\nlet profile:State=memory_encryption::initial();\n'
   if row['profile']!='disabled':
    for i,part in enumerate(row['profile'].split('_')):
     case='EncryptedBit' if part[0]=='e' else 'SharedBit';body+=f'let configured{i}:bool=memory_encryption::configure(&mut profile,Configuration::{case} {{position:{part[1:]}}});\n'
   frame=row['cr3_frame'];body+=f'let observation:ObservedCr3=encrypted_register_operands::cr3_observed(&profile,{raw});\nlet cr3_ok:bool=matches_profile_cr3(observation,{str(frame is not None).lower()},{frame or 0},{row["cr3_low"]});\n'
   body+=f'let apic:ApicObservation=encrypted_register_operands::apic_observed(&profile,{raw});\nlet apic_ok:bool=apic.frame=={row["apic_frame"]} && apic.raw_flags=={raw} && apic.flags.raw=={row["apic_flags"]};\n'
   calls=[('cr3_operand',f'cr3_raw_operand(&profile,{raw},64188,false)'),('cr3_no_flush',f'cr3_raw_operand(&profile,{raw},64188,true)'),('cr3_flags',f'cr3_flags_operand(&profile,{raw},Cr3Flags {{raw:32792}})'),('cr3_pcid',f'cr3_pcid_operand(&profile,{raw},Pcid {{value:2748}},true)'),('apic_operand',f'apic_raw_operand(&profile,{raw},9223372036854843392)'),('apic_preserving',f'apic_preserving_operand(&profile,48358647703818240,{raw},ApicBaseFlags {{raw:526336}})')]
   for i,(key,call) in enumerate(calls):
    expected=row[key];body+=f'let operand{i}:NumberResult=encrypted_register_operands::{call};\nlet pass{i}:bool=matches_profile_number(operand{i},{str(expected is not None).lower()},{expected or 0});\n'
   body+='cr3_ok && apic_ok && '+' && '.join(f'pass{i}' for i in range(6))+'\n}\n'
  body+='machine scenario()->i32 {\n'+''.join(f'let case{n}:bool=row{n}();\n' for n in range(10))+'transition '+' && '.join(f'case{n}' for n in range(10))+' { true -> (0) _ -> (1) } }\n'+TAIL
  path=folder/f'batch-{start//10:02}.omg'
  if '--check' in sys.argv:assert path.read_text()==body,path
  else:path.write_text(body)
 print('100 pinned pure-expression observations;10 Omega profile fixtures')
if __name__=='__main__':main()
