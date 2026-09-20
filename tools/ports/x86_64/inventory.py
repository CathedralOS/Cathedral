#!/usr/bin/env python3
"""Reproduce/check the reviewed full-crate classification; never claim translation."""
from pathlib import Path
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
import re
import sys
from module_catalog import MODULES,F,A,E,I,P,R,T
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
DEST=ROOT/'source/libraries/x86_64'
spec=importlib.util.spec_from_file_location('shared_inventory',HERE.parent/'inventory.py')
shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
PIN='cc35c876d3badb57df54a66e22f7768a52be95f2'
CHECKOUT=ROOT/'reference_code/rust-osdev/x86_64'
CATEGORIES={F,A,E,I,P,R,T}
def classify(path,key,text,lines):
    line=int(key.split(':',1)[0]);name=key.split(':',1)[1]
    if text.startswith(chr(34)):return R,'Shared lexical scanner anchor inside a format string; no Rust declaration or public API exists at this anchor.'
    if re.match(r'(?:pub(?:\([^)]*\))?\s+)?(?:use|mod)\b',text):return R,'Rust module/reexport packaging; child module and source file have their own reviewed rows.'
    if name in {'fmt','Output','Item','Sealed','clone','eq','Error'}:return R,'Rust formatting/trait boilerplate is not a separate Cathedral hardware representation; ordinary Omega value behavior replaces it as needed.'
    if path.startswith('testing/'):
        return T,'Retain scenario as integration-test intent; Rust bootloader/spin/lazy_static/ABI harness is deliberately not imported. '+MODULES[path][1]
    test_start=next((n for n,l in enumerate(lines,1) if re.match(r'\s*mod (?:tests?|proofs)\s*\{',l)),None)
    if test_start and line>=test_start:return T,'Upstream unit/Kani assertion or support declaration retained for X86-002; no test executed or proof ported by this audit.'
    if path=='src/instructions/port.rs':
        if name in {'read_from_port','write_to_port','read','write'}:return I,'Exact in/out operation requires existing PortIo instruction/provider boundary; read/write access marker alone grants nothing.'
        return R,'Rust port wrapper/constructor/access marker is not Cathedral PortIo authority; reuse explicit provider contracts.'
    if path=='src/structures/port.rs':return R,'Unsafe PortRead/PortWrite primitive trait is deliberately subsumed by checked instructions and explicit PortIo.'
    if path=='src/structures/paging/frame_alloc.rs':return R,'Do not copy unsafe ambient allocator ownership. Unique live frames/deallocation require qualified Extent and backing lifecycle.'
    if path=='src/structures/mem_encrypt.rs':
        if name=='MemoryEncryptionConfiguration':return F,'Explicit encryption/shared-bit configuration is pure data; does not configure existing mappings.'
        return P,'Global encryption/address-mask mutation must become explicit admitted CPU/mapping configuration; pure transforms can be extracted with configuration supplied as input.'
    if '/paging/mapper/' in path:
        if name in {'ignore','frame_to_pointer'}:return R,'Do not allow freely discarded flush debt or arbitrary numeric-frame pointer conversion to stand in for authority.'
        if name in {'p1_page','p2_page','p3_page'}:return A,'Recursive page-coordinate composition is separable pure arithmetic; establishing the recursive mapping remains policy.'
        if name in {'flush','flush_all'}:return I,'TLB invalidation needs admitted instruction and exact mapping/CPU scope.'
        if re.search(r'\b(?:enum|struct)\s+(?:TranslateResult|MappedFrame|MapToError|UnmapError|FlagUpdateError|TranslateError|InvalidPageTable|PageTableWalkError|PageTableCreateError)\b',text):return F,'Detached translation/error representation is ordinary pure work; success does not establish a live mapping.'
        return P,'Live mapper/walker mutation depends on admitted backing, alias/lifetime, hierarchy role and TLB settlement. Pure snapshot algorithms can be separated; this is engineering/integration work, not a proven language blocker.'
    if path=='src/instructions/smap.rs' and name=='new_unchecked':return R,'Unchecked construction of a Rust CPU-support marker is not Cathedral admission or authority; explicit observed feature evidence must be supplied.'
    if path.startswith('src/instructions/'):
        if path.endswith('/tlb.rs') and ((33<=line<101) or 186<=line<218 or 228<=line<316 or 373<=line<392):
            return A if 'fn ' in text else F,'Inert command/PCID/batch/error geometry can port; feature observations must be supplied explicitly and no flush authority is implied.'
        return I,'Execution or CPU-state discovery requires checked instruction admission, feature/state preconditions and provider lifecycle; no ambient Rust call copied.'
    if path.startswith('src/registers/'):
        if name in {'read','read_raw','write','write_raw','write_raw_impl','update','get_reg','set_reg','read_base','write_base','read_pcid','write_pcid','write_pcid_no_flush','update_pcid','update_pcid_no_flush'}:
            return I,'Live register operation requires admitted instruction/state authority. Pure bit encodings and validation embedded in this operation can be extracted separately.'
    if path=='src/addr.rs' and name in {'from_ptr','as_ptr','as_mut_ptr'}:return P,'Numeric/raw pointer projection is not a grant; preserve provenance/correspondence and require qualified access before use.'
    if path in {'src/structures/gdt.rs','src/structures/idt.rs'}:
        if name in {'load','load_unsafe','iretq'}:return I,'Live table installation or interrupt return requires admitted root/state/backing contracts.'
        if name in {'set_handler_addr','set_handler_fn','HandlerFunc','HandlerFuncWithErrCode','PageFaultHandlerFunc','DivergingHandlerFunc','DivergingHandlerFuncWithErrCode','GeneralHandlerFunc','HandlerFuncType','to_virt_addr','handler','set_general_handler','GENERAL_HANDLER'}:return R,'Rust handler function pointer/ABI generator cannot establish Cathedral installed entry identity or root authority; use existing root protocol and raw gate geometry.'
        if name in {'tss_segment','tss_segment_unchecked','tss_segment_with_iomap','pointer','as_mut','deref'}:return P,'Detached encoding is separable; pointer provenance, admitted backing lifetime and live CPU observation/mutation must be explicit.'
    if path=='src/structures/paging/page_table.rs' and name in {'PageTableEntry','PageTable'}:
        return E,'Existing Cathedral PTE schema and detached512-entry candidate own this representation; extend codecs/operations without a second competing PTE type.'
    if path=='src/structures/idt.rs' and name in {'Entry','ExceptionVector'}:
        return E,'Existing X86IdtGate/exception-vector facts own hardware representation; Rust API typing/alignment is not silently equated with Cathedral policy.'
    return (A if 'fn ' in text else F),'Ordinary missing pure representation/algorithm work for X86-001/002; existing module targets are collision/reuse evidence, not full API equivalence.'

def supplement(source):
    # Lexical supplements enumerate named enum variants (including implicit
    # discriminants) and macro definitions missed by the shared shallow index.
    lines=source.splitlines();result={};owner=None;variant=None;depth=0
    for number,line in enumerate(lines,1):
        clean=line.split('//',1)[0].strip()
        if owner is None:
            match=re.search(r'\b(?:pub(?:\([^)]*\))?\s+)?enum\s+(\w+)',clean)
            if match and '{' in clean:owner=match.group(1);depth=clean.count('{')-clean.count('}');continue
        else:
            if depth==1:
                match=re.match(r'([A-Z]\w*)\s*(?:[=,({]|$)',clean)
                if match:
                    variant=match.group(1)
                    result[f'{number}:variant::{owner}::{variant}']=clean
            elif depth==2 and variant:
                field=re.match(r'([a-z_]\w*)\s*:',clean)
                if field:result[f'{number}:variant-field::{owner}::{variant}::{field.group(1)}']=clean
            depth+=clean.count('{')-clean.count('}')
            if depth<=0:owner=None;depth=0
        macro=re.search(r'\bmacro_rules!\s*(\w+)',clean)
        if macro:result[f'{number}:macro::{macro.group(1)}']=clean
    return result

def documents():
    manifest=shared.snapshot(CHECKOUT,PIN,['src','testing'],'https://github.com/rust-osdev/x86_64')
    if set(manifest['files'])!=set(MODULES):raise ValueError('module catalog differs from exact pinned Rust files: '+str(set(manifest['files'])^set(MODULES)))
    modules={'format':'cathedral-x86-module-map-v1','revision':PIN,'stage':'inventoried','categories':sorted(CATEGORIES),'modules':{}}
    supplements={'format':'cathedral-x86-lexical-supplement-v1','revision':PIN,'files':{}}
    counts=Counter()
    for path,entry in manifest['files'].items():
        categories,reason,targets=MODULES[path]
        for target in targets:
            file=ROOT/target['path']
            if not file.is_file() or target['anchor'] not in file.read_text():raise ValueError('missing local comparison target '+str(target))
        source=(CHECKOUT/path).read_text();lines=source.splitlines()
        entry.update(disposition='pending',reason='Source reviewed and fully module-classified; translation/integration remains staged work. '+reason,classification=categories,comparison_targets=targets)
        for key,row in entry['symbols'].items():
            category,why=classify(path,key,row['anchor'],lines)
            row.update(classification=category,disposition='omitted' if category in {R,E} else 'pending',reason=why)
            number=int(key.split(':',1)[0])
            if path=='src/instructions/port.rs' and number in {23,34,54,63}:
                row.update(disposition='blocked',blocker='omega:x86-port-widths',reason='PORT-BLOCKED[omega:x86-port-widths]: current checked in/out contracts admit only u8 payloads; u16/u32 operations need exact width contracts and realization. Source probe confirms u16 destination rejection.',targets=[{'path':'tools/ports/x86_64/catalog-probes/port16.omg','anchor':'PORT-BLOCKED[omega:x86-port-widths]'}])
            if path=='src/instructions/tlb.rs' and number==17:
                row.update(disposition='blocked',blocker='omega:x86-tlb-instructions',reason='PORT-BLOCKED[omega:x86-tlb-instructions]: invlpg has no known instruction contract in Omega eaa7993; source probe rejects the mnemonic. Pure TLB request arithmetic remains ordinary work.',targets=[{'path':'tools/ports/x86_64/catalog-probes/invlpg.omg','anchor':'PORT-BLOCKED[omega:x86-tlb-instructions]'}])
            counts[category]+=1
        extra=supplement(source)
        supplemental={}
        for key,anchor in extra.items():
            supplemental[key]={'anchor':anchor,'classification':T if path.startswith('testing/') else R if ':macro::' in key else F,'reason':'Source-defined Rust macro retained as review input; expansion is not a public Omega API.' if ':macro::' in key else 'Explicit named enum variant, including implicit discriminant; parent enum/module classification controls future translation.'}
        supplements['files'][path]={'sha256':entry['sha256'],'symbols':supplemental}
        modules['modules'][path]={'sha256':entry['sha256'],'line_count':len(lines),'categories':categories,'review':reason,'comparison_targets':targets,'anchor_count':len(entry['symbols']),'supplemental_anchor_count':len(supplemental)}
    modules['lexical_anchor_classifications']=dict(sorted(counts.items()))
    modules['limitations']='All41 Rust files are classified. Lexical anchors plus enum/macro supplements are source-bound review indexes, not Rust semantic expansion or a compilation claim. Pending means classified implementation work, not an unreviewed module. Related Cathedral targets indicate overlap/subset only.'
    tests={'format':'cathedral-x86-test-index-v1','revision':PIN,'execution':'not run; retained source scenarios only','tests':[]}
    for path in modules['modules']:
        source=(CHECKOUT/path).read_text()
        for marker in re.finditer(r'#\[(?:test|test_case|kani::proof)\]',source):
            tail=source[marker.end():]
            function=re.search(r'\bfn\s+(\w+)',tail)
            if function is None:raise ValueError('test marker without following function')
            position=marker.end()+function.start()
            tests['tests'].append({'path':path,'line':source[:position].count('\n')+1,'name':function.group(1),'kind':'kani-proof' if 'kani::proof' in marker.group() else 'hardware-integration' if path.startswith('testing/') or path.startswith('src/instructions/') or path in ('src/registers/rflags.rs',) or path=='src/registers/mxcsr.rs' else 'pure-or-detached-fixture','status':'not translated/executed in X86-000'})
    tests['integration_roots']=[]
    for path in modules['modules']:
        if path.startswith('testing/tests/'):
            source=(CHECKOUT/path).read_text()
            match=re.search(r'\bfn\s+_start',source)
            if match:tests['integration_roots'].append({'path':path,'line':source[:match.start()].count('\n')+1,'name':'_start','kind':'hardware-integration-root','status':'Rust boot harness deliberately not ported; retain scenario against Cathedral admitted roots'})
    return manifest,modules,supplements,tests

def render(modules):
    rows=['# Complete pinned module map','',f'Pin `{PIN}`. Every tracked Rust file under `src/` and `testing/` is listed.',
          'Categories may overlap. Existing targets indicate a representation subset or authority precedent, not whole-module implementation.',
          'This is a reviewed source inventory; pending symbol dispositions mean future implementation, not missing classification.','',
          '| Upstream module | Categories | Reconciliation / next work | Existing Cathedral comparison |','| --- | --- | --- | --- |']
    for path,row in modules['modules'].items():
        refs=', '.join('['+Path(t['path']).name+'](../../../'+t['path']+')' for t in row['comparison_targets']) or 'none'
        rows.append('| `'+path+'` | '+', '.join(row['categories'])+' | '+row['review'].replace('|','\\|')+' | '+refs+' |')
    rows+=['','No module is promoted to translated/typechecked/tested by this table. Macro bodies, private fields and feature-dependent branches remain bound by whole-file hashes.','']
    return '\n'.join(rows)

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--write',action='store_true');args=parser.parse_args()
    manifest,modules,supplements,tests=documents()
    output={'inventory.json':json.dumps(manifest,indent=2)+'\n','module-map.json':json.dumps(modules,indent=2)+'\n','lexical-supplement.json':json.dumps(supplements,indent=2)+'\n','MODULES.md':render(modules),'tests.json':json.dumps(tests,indent=2)+'\n'}
    for name,content in output.items():
        p=DEST/name
        if args.write:p.write_text(content)
        elif not p.exists() or p.read_text()!=content:raise SystemExit(name+' differs from reviewed catalog/pinned sources; inspect changes before --write')
    shared.check(manifest,CHECKOUT)
    print(json.dumps({'files':len(modules['modules']),'src_files':sum(p.startswith('src/') for p in modules['modules']),'lexical_anchors':sum(r['anchor_count'] for r in modules['modules'].values()),'supplemental_anchors':sum(r['supplemental_anchor_count'] for r in modules['modules'].values()),'classifications':modules['lexical_anchor_classifications'],'test_scenarios':len(tests['tests']),'integration_roots':len(tests['integration_roots']),'scope':'inventoried only; all pending rows explicitly classified implementation work'},indent=2))
if __name__=='__main__':main()
