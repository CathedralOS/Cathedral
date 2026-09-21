#!/usr/bin/env python3
"""Reproduce the final source overlay audit; not a new semantic/native test."""
import argparse,hashlib,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
DEST=ROOT/'source/libraries/x86_64';UP=ROOT/'reference_code/rust-osdev/x86_64'
spec=importlib.util.spec_from_file_location('port_inventory',HERE.parent/'inventory.py')
api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def audit():
 historical=json.loads((DEST/'remaining-pure.json').read_text())
 profile=historical['profile_composition_review'];assert not profile['remaining'],'ordinary pure families remain'
 inventories=sorted(DEST.glob('*-inventory.json'))
 facts=[ROOT/'source/drivers/facts'/name for name in ['x86_descriptors-inventory.json','x86_interrupts-inventory.json','x86_registers-inventory.json']]
 overlay={};sources={}
 for path in inventories+facts:
  value=json.loads(path.read_text());api.check(value,UP,ROOT)
  for source,file in value['files'].items():
   sources[source]=file['sha256']
   for key,row in file['symbols'].items():
    if row['disposition']=='translated':overlay.setdefault((source,key),[]).append(str(path.relative_to(ROOT)))
 baseline=json.loads((DEST/'inventory.json').read_text());api.check(baseline,UP,ROOT)
 for path,file in baseline['files'].items():assert sha(UP/path)==file['sha256']
 anchors=[];exceptions=[]
 for source,file in historical['modules'].items():
  for key,row in file['symbols'].items():
   if row['classification']!='remaining-pure-component':continue
   item={'source':source,'anchor':key,'translated_component_inventories':sorted(overlay.get((source,key),[]))}
   if not item['translated_component_inventories']:
    assert (source,key)==('src/structures/gdt.rs','127:MAX'),item
    item['reason']='Impl const-generic parameter, represented by checked owned GDT capacity; not a missing declaration.'
    item['evidence']='source/libraries/x86_64/gdt-storage.PORT.md';exceptions.append(item)
   else:anchors.append(item)
 completed={x['family']for x in historical['subsequent_completions']if x['stage']=='tested'}
 assert set(historical['remaining_families'])<=completed
 assert len(baseline['files'])==41 and len(anchors)==71 and len(exceptions)==1
 assert len(inventories)==27 and len(facts)==3
 inputs=[Path(__file__).resolve(),DEST/'inventory.json',DEST/'remaining-pure.json',*inventories,*facts]
 return {'format':'cathedral-x86-pure-closure-v1','stage':'source and component-boundary audit; does not rerun semantic tests','upstream_revision':baseline['upstream']['revision'],'source_hashes':{p:f['sha256']for p,f in baseline['files'].items()},'library_slice_inventory_count':len(inventories),'fact_inventory_count':len(facts),'historical_families_completed':sorted(set(historical['remaining_families'])),'historical_pure_anchors_overlaid':anchors,'historical_lexical_exceptions':exceptions,'completed_profile_components':profile['completed_components'],'remaining_ordinary_pure_families':profile['remaining'],'profiles':['64-bit usize, 48-bit canonical virtual and 52-bit physical geometry','explicit coherent encryption snapshots; current physical bit differs from accumulated PTE mask','checked overflow where documented, preserving separately retained pinned release behavior','numeric/captured/owned detached algorithms; no pointer/trait or live authority facsimiles'],'outside_boundary':['native ABI and instructions','live provider, mapping and custody integration','Kani universal proofs and 32-bit usize profiles','stale typed values surviving concurrent ambient encryption reconfiguration'],'input_sha256':{str(p.relative_to(ROOT)):sha(p)for p in inputs}}
def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();value=audit();path=DEST/'closure-review.json'
 if a.write:path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==value,'stale closure audit'
 print('PASS41 source hashes,27 library +3 fact inventories,12 historical families,71 overlays +1 generic-parameter exception; no semantic execution implied')
if __name__=='__main__':main()
