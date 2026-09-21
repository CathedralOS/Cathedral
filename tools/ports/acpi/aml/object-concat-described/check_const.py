#!/usr/bin/env python3
"""Evaluate representative direct description-aware concatenations as constant expressions."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
import fixtures,check
HERE=fixtures.HERE;ROOT=fixtures.ROOT;compiler=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def main():
 execution_root=str(ROOT.resolve());build=check.build_text('constant');build_sha256=check.text_sha(build)
 inputs=check.snapshot();binary=check.sha(compiler);selected=['desc_Device_buffer_left','desc_Package_integer_right','left_encoding_before_right_id_True'];rows={r['name']:r for r in fixtures.cases()};proofs=[]
 for name in selected:
  for control in [False,True]:
   with tempfile.TemporaryDirectory(prefix='cathedral-object-concat-described-constant-')as d:
    work=Path(d);text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(rows[name],control)+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
''';(work/'main.omg').write_text(text);(work/'build.omg').write_text(build);assert check.sha(work/'build.omg')==build_sha256
    run=subprocess.run([str(compiler),'--check',str(work/'main.omg')],capture_output=True,text=True);output=run.stdout+run.stderr
    if control:assert run.returncode and 'cannot prove requires contract'in output and'1 == 0'in output,output
    else:assert run.returncode==0,output
    proofs.append(dict(execution_root=execution_root,build_sha256=build_sha256,case=name,control=control,source_sha256=hashlib.sha256(text.encode()).hexdigest(),output=output));print('PASS',name,'control',control,flush=True)
    assert inputs==check.snapshot() and binary==check.sha(compiler) and check.sha(work/'build.omg')==build_sha256,'inputs changed during constant proof'
 (HERE/'const-verification.json').write_text(json.dumps(dict(execution_root=execution_root,build_text=build,build_sha256=build_sha256,stage='constant evaluation and checked requires; native/hardware not run',compiler_sha256=binary,omega_revision=check.PIN,input_sha256=inputs,cases=selected,proofs=proofs),indent=2,sort_keys=True)+'\n')
if __name__=='__main__':main()
