#!/usr/bin/env python3
"""Execute actual loader-to-executor bodies and changed-result controls after checking."""
import argparse,hashlib,json,os,re,shutil,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
SHARED=ROOT/'tools/ports/acpi/interpreter/execution'

def parser_suite():
    directory=ROOT/'tools/ports/acpi/aml'
    metadata=json.loads((directory/'cases.json').read_text())
    imports=set();helpers={};rows=[]
    for name,item in metadata.items():
        source=(directory/'cases'/f'{name}.omg').read_text()
        start=source.index('machine test()');end=source.index('const TEST_RESULT')
        prefix=source[:start];body=source[start:end]
        imports.update(re.findall(r'^use [^;]+;',prefix,re.M))
        for found in re.finditer(r'^machine (\w+)\(',source,re.M):
            if found[1] in ['test','require_ok']:continue
            opening=source.index('{',found.start());depth=1;closing=opening+1
            while depth:
                depth+=(source[closing]=='{')-(source[closing]=='}');closing+=1
            text=source[found.start():closing]
            if found[1]in helpers:assert helpers[found[1]]==text,(name,found[1])
            else:helpers[found[1]]=text
        assert body.count(item['mutation'][0])==1,(name,'mutation must remain in original assertion body')
        rows.append({'name':'parser_'+name.replace('-','_'),'source':body,'mutation':item['mutation'],'original':str((directory/'cases'/f'{name}.omg').relative_to(ROOT))})
    def render(row,control,machine):
        source=row['source'].replace(*row['mutation'])if control else row['source']
        return source.replace('machine test()',f'machine {machine}(&mut self)',1)
    return rows,'\n'.join(sorted(imports))+'\n'+'\n'.join(helpers.values())+'\n',render

def source_snapshot(parser_regression,all_rows):
    sources=set()
    packages=['source/libraries/acpi/aml']if parser_regression else['source/libraries/acpi/aml','source/libraries/acpi/interpreter','source/libraries/acpi/pipeline']
    for package in packages:sources.update((ROOT/package).rglob('*.omg'))
    if parser_regression:
        sources.update(ROOT/row['original']for row in all_rows);sources.add(ROOT/'tools/ports/acpi/aml/cases.json')
    sources.update(path for path in HERE.iterdir()if path.suffix in ['.py','.omg','.json']and 'verification'not in path.name)
    sources.add(SHARED/'checked_runner.rs');sources.add(SHARED/'runner.Cargo.lock')
    return {str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest()for path in sorted(sources)}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--match',default='');p.add_argument('--parser-regression',action='store_true');p.add_argument('--record',type=Path);p.add_argument('--omega-source',type=Path,default=ROOT.parent/'Omega');p.add_argument('--target-dir',type=Path,default=Path('/tmp/cathedral-acpi-execution-checked'));a=p.parse_args();omega=a.omega_source.resolve();target=a.target_dir.resolve()
    revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip()
    assert revision=='eaa7993a23623cd8fabf45350340479c5c9c7879'
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip(),'Omega must be clean'
    print('Omega source:',revision,flush=True)
    if a.parser_regression:
        subprocess.run(['python3',str(ROOT/'tools/ports/acpi/aml/audit.py')],check=True)
        all_rows,prefix,render=parser_suite()
    else:
        subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
        all_rows,prefix,render=fixtures.cases(),fixtures.IMPORTS,fixtures.render
    rows=[row for row in all_rows if any(part in row['name']for part in a.match.split(','))]
    if not rows:raise SystemExit('No selected scenarios')
    source_hashes=source_snapshot(a.parser_regression,all_rows)
    with tempfile.TemporaryDirectory(prefix='cathedral-acpi-pipeline-')as directory:
        work=Path(directory)
        manifest=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
        for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():manifest.append(f'{name} = {{ path="{omega}/omega-rust/{path}" }}')
        manifest +=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{SHARED}/checked_runner.rs"']
        (work/'Cargo.toml').write_text('\n'.join(manifest)+'\n');(work/'Cargo.lock').write_bytes((SHARED/'runner.Cargo.lock').read_bytes())
        subprocess.run([shutil.which('mbx')or'cargo','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True,cwd=omega)
        runner=target/'release/cathedral-acpi-checked-runner';print('Harness SHA-256:',hashlib.sha256(runner.read_bytes()).hexdigest(),flush=True)
        build=((ROOT/'tools/ports/acpi/aml/build.omg')if a.parser_regression else(HERE/'build.omg')).read_text()
        for relative in ['../../../../source/libraries/acpi/pipeline','../../../../source/libraries/acpi/aml','../../../../source/libraries/acpi/interpreter/execution','../../../../source/libraries/acpi/interpreter']:build=build.replace(relative,str((HERE/relative).resolve()))
        (work/'build.omg').write_text(build)
        source=[prefix,'data PipelineSuite {}\n'];selections=[]
        for row in rows:
            for control in [False,True]:
                machine='PipelineSuite::'+row['name']+('_control'if control else'_positive')
                source.append(render(row,control,machine));selections.append(machine+'='+str(int(control)))
        (work/'main.omg').write_text(''.join(source))
        started=time.monotonic();command=[str(runner),str(work/'main.omg'),str(work/'build'),*selections]
        process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));lines=[]
        for line in process.stdout:print(line,end='',flush=True);lines.append(line)
        result=subprocess.CompletedProcess(command,process.wait(),''.join(lines));result.check_returncode()
        if a.record:
            assert source_hashes==source_snapshot(a.parser_regression,all_rows),'Source changed during run'
            value={'format':'cathedral-acpi-parser-regression-checked-v1'if a.parser_regression else'cathedral-acpi-pipeline-checked-v1','stage':'checked-interpreter execution; native/hardware not run','omega_revision':revision,'runner_sha256':hashlib.sha256(runner.read_bytes()).hexdigest(),'cargo_lock_sha256':hashlib.sha256((work/'Cargo.lock').read_bytes()).hexdigest(),'cases':[row['name']for row in rows],'scenario_count':len(rows),'control_count':len(rows),'elapsed_seconds':round(time.monotonic()-started,3),'evaluator_step_limit':10000000,'source_sha256':source_hashes,'output':result.stdout}
            a.record.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
    label='parser regression'if a.parser_regression else'pipeline'
    print(f'PASS {len(rows)} {label} scenarios + {len(rows)} changed-body controls. Native/hardware NOT RUN.',flush=True)
if __name__=='__main__':main()
