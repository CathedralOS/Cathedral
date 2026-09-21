#!/usr/bin/env python3
"""Complete lexical source map for the strict detached _PRT adaptation."""
import argparse
import importlib.util
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
PREFIX='source/libraries/acpi/pci_routing/'
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--checkout',type=Path,default=ROOT/'reference_code/rust-osdev/acpi');parser.add_argument('--check',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');inventory=importlib.util.module_from_spec(spec);spec.loader.exec_module(inventory)
 result=inventory.snapshot(args.checkout,PIN,['src/aml/pci_routing.rs'],'https://github.com/rust-osdev/acpi')
 entry=result['files']['src/aml/pci_routing.rs'];entry.update(disposition='translated',reason='Strict bounded pure decomposition only; interpreter evaluation and interrupt installation remain caller boundaries. See PORT.md for exact pin deviations.',targets=[dict(path=PREFIX+'routes.omg',anchor='pub machine decode'),dict(path=PREFIX+'link_resources.omg',anchor='pub machine select_bytes')])
 mapping={
 '14:crate::aml::resource::IrqDescriptor;':('model.omg','irq:Irq;','Reuse canonical resources::resource_model::Irq; no competing IRQ representation.'),
 '17:Pin':('routes.omg','pin<4','Validated numeric 0..3 operand; no invented native enum layout.'),
 '25:PciRouteType':('model.omg','pub data Target','Semantic Gsi/Link cases; Link retains SourceIndex omitted by the pin; None is explicit empty/failure carrier.'),
 '40:PciRoute':('model.omg','pub data Route','Device/pin/target retained. Strict address requires functionFFFF, eliminating per-row function state.'),
 '52:PciRoutingTable':('model.omg','pub data Routes','Ordinary initialized 32-row storage/count, an explicit bounded profile replacing Vec.'),
 '61:from_prt_path':('routes.omg','pub machine decode','Pure package decoding and captured NameReference level search; supplied already-evaluated ObjectStore object. No interpreter evaluation. Four members, DWORD widths, FFFF function and declared scope validation intentionally strict.'),
 '170:route':('routes.omg','pub machine select','Pure first-match selection plus crs_request and detached link_resources::select_bytes. SourceIndex counts physical resource descriptors. GSI remains a number; no interrupt setup or evaluator authority, and no default IRQ flags are fabricated.')}
 assert set(mapping)==set(entry['symbols'])
 for key,(file,anchor,reason) in mapping.items():entry['symbols'][key].update(disposition='translated',reason=reason,targets=[dict(path=PREFIX+file,anchor=anchor)])
 entry['symbols']['170:route']['targets'].extend([dict(path=PREFIX+'routes.omg',anchor='pub machine crs_request'),dict(path=PREFIX+'link_resources.omg',anchor='pub machine select_bytes')])
 path=ROOT/PREFIX/'inventory.json';text=json.dumps(result,indent=2,sort_keys=True)+'\n'
 if args.check:assert path.read_text()==text,'inventory drift'
 else:path.write_text(text)
 inventory.check(result,args.checkout,repository=ROOT,require_transcribed=True)
 print('PASS seven pinned lexical anchors; complete file hash and explicit pure/evaluation split')
if __name__=='__main__':main()
