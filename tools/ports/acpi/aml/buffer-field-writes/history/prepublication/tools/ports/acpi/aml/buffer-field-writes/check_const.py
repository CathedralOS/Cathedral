#!/usr/bin/env python3
"""Evaluate representative actual writes; smaller backing checks fit the constant budget."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures,check
HERE=fixtures.HERE;ROOT=fixtures.ROOT;compiler=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def main():
 inputs=check.snapshot();binary=check.sha(compiler);selected=['write_9_1_False','string_nul_True','field_max'];rows={r['name']:r for r in fixtures.cases()};proofs=[]
 for name in selected:
  for control in [False,True]:
   with tempfile.TemporaryDirectory(prefix='cathedral-query-constant-')as d:
    work=Path(d);text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(rows[name],control,constant=True)+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
''';(work/'main.omg').write_text(text);(work/'build.omg').write_text('machine build(builder:&mut Build){builder.application("query-constant");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});builder.depend_as("integer_helpers",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/interpreter')+'"});}')
    run=subprocess.run([str(compiler),'--check',str(work/'main.omg')],capture_output=True,text=True);output=run.stdout+run.stderr
    if control:assert run.returncode and 'cannot prove requires contract'in output and'1 == 0'in output,output
    else:assert run.returncode==0,output
    proofs.append(dict(case=name,control=control,source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=output));print('PASS',name,'control',control,flush=True)
    assert inputs==check.snapshot() and binary==check.sha(compiler),'inputs changed during constant proof'
 (HERE/'const-verification.json').write_text(json.dumps(dict(stage='constant evaluation and checked requires; native/hardware not run',compiler_sha256=binary,omega_revision=check.PIN,input_sha256=inputs,cases=selected,proofs=proofs),indent=2,sort_keys=True)+'\n')
if __name__=='__main__':main()
