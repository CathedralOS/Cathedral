#!/usr/bin/env python3
"""Verify recovered failed-run evidence against the immutable original sources."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--require-binaries',action='store_true')
    args=parser.parse_args()
    record=json.loads((HERE/'failed-run.json').read_text())
    launch=json.loads((HERE/'launch-inputs.json').read_text())
    terminal=json.loads((HERE/'terminal-output.json').read_text())
    assert record['input_sha256']==launch and len(launch)==160
    assert sha(HERE/'terminal-output.json')==record['terminal_sha256']
    assert hashlib.sha256(terminal['output'].encode()).hexdigest()==record['terminal_output_sha256']
    assert terminal['exit_code']==record['checker_exit_code']==1
    assert record['runner_exit_code'] is None and record['source_unchanged_at_recovery']
    repo=subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=HERE,text=True).strip()
    with tempfile.TemporaryDirectory(prefix='cathedral-sizeof-original-audit-') as directory:
        root=Path(directory)
        for name,digest in launch.items():
            blob=subprocess.check_output(['git','show',record['original_revision']+':'+name],cwd=repo)
            assert hashlib.sha256(blob).hexdigest()==digest,name
            destination=root/name
            destination.parent.mkdir(parents=True,exist_ok=True)
            destination.write_bytes(blob)
        tools=root/'tools/ports/acpi/interpreter/sizeof-execution'
        sys.path.insert(0,str(tools))
        spec=importlib.util.spec_from_file_location('original_sizeof_check',tools/'check.py')
        check=importlib.util.module_from_spec(spec);spec.loader.exec_module(check)
        assert record['omega_revision']==check.PIN
        assert record['groups']==list(check.DIRECTORIES) and record['scope']=='full'
        assert check.snapshot(record['groups'])==launch
        toolchain=json.loads((tools/'toolchain.json').read_text())
        assert record['runner_sha256']==toolchain['sha256'][str(check.RUNNER)]
        if args.require_binaries:
            assert sha(Path(record['runner_path']))==record['runner_sha256']
        modules=[];entries=[];generated={}
        for group in record['groups']:
            fixture=check.module(group);rows=fixture.cases()
            source,names=check.module_source(group,rows,fixture)
            filename='authored_'+group+'.omg'
            assert (HERE/'generated'/filename).read_text()==source
            generated[filename]=check.text_sha(source)
            modules.append(dict(group=group,cases=[row['name']for row in rows],source_sha256=check.text_sha(source),selections=names))
            entries.extend(names)
        for filename,source in [('main.omg',check.driver_source(record['groups'])),
                                ('build.omg',check.build_text(root=Path(record['execution_root'])))]:
            assert (HERE/'generated'/filename).read_text()==source
            generated[filename]=check.text_sha(source)
        assert generated==record['generated_sha256']
        assert modules==record['modules'] and entries==record['selections']
        assert len(entries)==2*record['positive_count']==2*record['control_count']==724
    output=terminal['output']
    assert output.count('AssertionError: CHECKED authored package and dependency bodies; native publication NOT requested')==1
    observations=re.findall(r'^(PASS|FAIL) (\S+) expected=(\d+) observed=(-?\d+) error=(.*?) usage=',output,re.M)
    assert len(observations)==len(re.findall(r'^(?:PASS|FAIL) ',output,re.M))==724
    assert [name+'='+expected for status,name,expected,observed,error in observations]==entries
    assert all(error=='None'for status,name,expected,observed,error in observations)
    assert all(status==('PASS'if expected==observed else 'FAIL')for status,name,expected,observed,error in observations)
    failed=[name for status,name,expected,observed,error in observations if status=='FAIL']
    assert failed==record['failures']==['SizeOfExecutionSuite::size_malformed_package_positive']
    assert sum(row[0]=='PASS'for row in observations)==record['passed_entries']==723
    assert len(failed)==record['failed_entries']==1
    assert sum(all(row[0]=='PASS'for row in observations[index:index+2])for index in range(0,724,2))==record['complete_passing_pairs']==361
    counts={}
    for module in modules:
        names=set(module['selections'])
        selected=[row for row in observations if row[1]+'='+row[2] in names]
        counts[module['group']]=dict(pairs=len(module['cases']),passed_entries=sum(row[0]=='PASS'for row in selected),failed_entries=sum(row[0]=='FAIL'for row in selected))
    assert counts==record['group_results']
    assert max(map(int,re.findall(r'fuel_units: (\d+)',output)))==record['maximum_fuel']
    print('PASS failed-evidence audit: 160 original inputs, eight regenerated files, 724 exact entries; overall failure retained')


if __name__=='__main__':
    main()
