#!/usr/bin/env python3
"""Run the exact pinned ConcatRes result block as an explicit private-body mirror."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;UP=ROOT/'reference_code/rust-osdev/acpi'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==fixtures.PIN
source=(UP/'src/aml/mod.rs').read_text();assert source==subprocess.check_output(['git','show',fixtures.PIN+':src/aml/mod.rs'],cwd=UP,text=True)
start=source.index('let result = {',source.index('Opcode::ConcatRes => {'));brace=source.index('{',start);end=brace+1;depth=1
while depth:depth+=(source[end]=='{')-(source[end]=='}');end+=1
body=source[brace+1:end-1];assert body.count('Object::Buffer(buffer).wrap()')==1;body=body.replace('Object::Buffer(buffer).wrap()','buffer')
mirror='// SPDX-License-Identifier: MIT OR Apache-2.0\n// Exact private ConcatRes result block; only Object wrapping -> returned Vec.\nfn concat_mirror(source1:&[u8],source2:&[u8])->Vec<u8>{\n'+body+'\n}\n'
p=HERE/'pinned.rs'
if p.exists():assert p.read_text()==mirror,'Pinned result-block extraction changed; review before updating'
else:p.write_text(mirror)
statements=[];expected={};omitted=[]
for r in fixtures.cases():
 if r['a']!=len(r['left']) or r['b']!=len(r['right']):omitted.append(r['name']);continue
 args=['&['+','.join(map(str,r[k]))+']'for k in ['left','right']]
 statements.append(f'let value=concat_mirror({args[0]},{args[1]});println!("{r["name"]}\\t{{}}",value.iter().map(|b|format!("{{b:02x}}")).collect::<String>());')
 parts=[r[k][:-2]if len(r[k])>=2 and r[k][-2]==121 else r[k]for k in ['left','right']];expected[r['name']]=bytes(parts[0]+parts[1]+[121,0]).hex()
with tempfile.TemporaryDirectory(prefix='cathedral-resource-composition-reference-')as name:
 work=Path(name);(work/'main.rs').write_text(mirror+'fn main(){\n'+'\n'.join(statements)+'\n}')
 compiled=subprocess.run(['rustc','--edition=2021',str(work/'main.rs'),'-o',str(work/'reference')],capture_output=True,text=True);assert compiled.returncode==0,compiled.stderr
 output=subprocess.check_output([str(work/'reference')],text=True);observed=dict(line.split('\t')for line in output.splitlines());assert observed==expected
 agreements=[r['name']for r in fixtures.cases()if r['result']=='Composed'and observed[r['name']]==bytes(r['output']).hex()]
 record=dict(stage='exact private ConcatRes result-block mirror only; no actual public interpreter/opcode/Store call',pin=fixtures.PIN,upstream_sha256=hashlib.sha256(source.encode()).hexdigest(),private_mirror_calls=len(observed),supported_byte_agreements=agreements,omitted_impossible_logical_extents=omitted,observed=observed,source_sha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [HERE/'reference.py',HERE/'fixtures.py',HERE/'pinned.rs',HERE/'cases.json']})
 (HERE/'reference-verification.json').write_text(json.dumps(record,indent=2)+'\n');print('PASS',len(observed),'labelled private result-block mirrors;',len(agreements),'supported byte agreements')
