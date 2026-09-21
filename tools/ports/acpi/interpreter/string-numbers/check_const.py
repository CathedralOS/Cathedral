#!/usr/bin/env python3
"""Representative current helpers evaluated as consts, including changed-body rejection."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
compiler=Path('/tmp/cathedral-omega-eaa7993/release/omega')
selected=['parse_max64_64_StrictDecimalHex','parse_above32_32_StrictDecimalHex','integer_64_18446744073709551615_Hexadecimal','buffer_boundaries_Decimal']
rows={r['name']:r for r in fixtures.cases()};runs=[]
for name in selected:
 for control in [False,True]:
  with tempfile.TemporaryDirectory(prefix='cathedral-string-numbers-const-')as directory:
   work=Path(directory);source=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(rows[name],control)+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
'''
   (work/'main.omg').write_text(source);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../../../source/libraries/acpi/interpreter',str(ROOT/'source/libraries/acpi/interpreter')))
   result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],capture_output=True,text=True);output=result.stdout+result.stderr
   good=result.returncode!=0 and'cannot prove requires contract'in output and'1 == 0'in output if control else result.returncode==0
   if not good:raise SystemExit(name+': '+output)
   runs.append({'case':name,'control':control,'output':output});print(('CONTROL'if control else'PASS'),name,flush=True)
files=list((ROOT/'source/libraries/acpi/interpreter').glob('*.omg'))+[HERE/'fixtures.py',HERE/'cases.json',HERE/'build.omg',Path(__file__).resolve()]
record={'stage':'constant evaluation and checked requires; native/hardware not run','compiler_sha256':hashlib.sha256(compiler.read_bytes()).hexdigest(),'cases':selected,'runs':runs,'source_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(files)}}
(HERE/'const-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
