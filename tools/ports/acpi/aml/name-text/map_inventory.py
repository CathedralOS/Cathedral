#!/usr/bin/env python3
"""Text-conversion slice only; unrelated namespace operations remain outside scope."""
import argparse,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
def main():
 p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=ROOT);p.add_argument('--check',action='store_true');a=p.parse_args()
 spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
 checkout=a.repository/'reference_code/rust-osdev/acpi';value=mod.snapshot(checkout,'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',['src/aml/namespace.rs'],'https://github.com/rust-osdev/acpi')
 entry=value['files']['src/aml/namespace.rs'];entry['reason']='This manifest covers textual conversion only; namespace storage, lookup and other name operations are inventoried in their owning slices.'
 mappings={'473:as_string':('name_text.omg','pub machine format','Bounded canonical uppercase, padded output; no Rust allocation.'),'586:from_str':('name_text.omg','pub machine parse','Primary ASL grammar with explicit prefix, lowercase and capacity differences.'),'651:from_str':('name_text.omg','state start','Embedded segment parsing validates and underscore-pads one to four characters; no separate NameSeg nominal type.'),'642:as_str':('name_text.omg','machine output_byte','Emit canonical segment bytes as part of initialized text; no unsafe UTF-8 borrow.'),'679:is_lead_name_char':('names.omg','pub machine lead_char','Reuse canonical uppercase facts after explicit lowercase folding.'),'683:is_name_char':('names.omg','pub machine name_char','Reuse canonical uppercase/digit facts after explicit lowercase folding.'),'584:Err':('model.omg','pub data Outcome','Canonical explicit outcomes replace Rust errors/panics.'),'649:Err':('model.omg','pub data Outcome','Canonical explicit outcomes replace Rust errors/panics.')}
 for key,symbol in entry['symbols'].items():
  symbol['reason']='Outside this textual-conversion slice; no whole namespace completion claim.'
  if key in mappings:
   file,anchor,reason=mappings[key];symbol.update(disposition='translated',reason=reason,targets=[dict(path='source/libraries/acpi/aml/'+file,anchor=anchor)])
  elif key in ['617:fmt','688:fmt']:symbol.update(disposition='omitted',reason='Rust Formatter/Debug integration is not a pure ASL conversion API; ordinary initialized text is supplied by format.')
 path=ROOT/'source/libraries/acpi/aml/name-text-inventory.json';text=json.dumps(value,indent=2,sort_keys=True)+'\n'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 result=mod.check(value,checkout,repository=ROOT);print(result)
if __name__=='__main__':main()
