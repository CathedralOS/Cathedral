#!/usr/bin/env python3
"""Check authored corpus and source scope; this does not compile or run Omega."""
import argparse
import ast
import json
import re
import subprocess
import check

BASE='3f33097fa1988b5873f145babda286b55d782071'
PREFIX='source/libraries/acpi/'
CHANGED={PREFIX+'interpreter/execution/'+name+'.omg' for name in
         ['numeric_string_execution','operator_specs','retire']}


def git(*args):
    return subprocess.check_output(['git',*args],cwd=check.ROOT)


def run():
    before=check.snapshot()
    for path in check.HERE.glob('*.py'):
        ast.parse(path.read_text(),filename=str(path))
    original=set(git('ls-tree','-r','--name-only',BASE,PREFIX).decode().splitlines())
    original={path for path in original if path.endswith('.omg')}
    current={str(path.relative_to(check.ROOT)) for path in (check.ROOT/PREFIX).rglob('*.omg')}
    assert current==original|CHANGED
    differences={path for path in current if path not in original or
                 (check.ROOT/path).read_bytes()!=git('show',BASE+':'+path)}
    assert differences==CHANGED
    # Borrowed generators and full-state comparators retain their base bytes.
    borrowed=[path for path in before if path.startswith('tools/') and
              not path.startswith(str(check.HERE.relative_to(check.ROOT))+'/')]
    for path in borrowed:
        assert (check.ROOT/path).read_bytes()==git('show',BASE+':'+path)
    source=(check.ROOT/PREFIX/'interpreter/execution/numeric_string_execution.omg').read_text()
    scopes=[]
    for match in re.finditer(r'(?:pub )?machine (\w+)\(',source):
        start=source.index('{',match.end());depth=1;end=start+1
        while depth:
            depth+=(source[end]=='{')-(source[end]=='}');end+=1
        body=source[start:end]
        states=set(re.findall(r'\bstate\s+(\w+)\(',body))
        calls=set(re.findall(r'->\s*(\w+)\(',body))
        assert calls<=states|{match[1]},(match[1],calls-states)
        scopes.append(dict(machine=match[1],states=sorted(states),transition_calls=sorted(calls)))
    all_source='\n'.join((check.ROOT/path).read_text() for path in sorted(current))
    for name in ['NumericStringPrepared','NumericStringStage']:
        assert len(re.findall(r'\bdata\s+'+name+r'\b',all_source))==1
    groups=[]
    for group in check.GROUPS:
        rows=check.rows(group)
        assert len(rows)==len({row['name'] for row in rows})
        if group in ['execution','bridge']:
            assert rows==json.loads((check.HERE/(group+'-cases.json')).read_text())
        else:
            inherited=check.fixtures.mid.render_rows({'mid':'execution','mid_bridge':'bridge'}.get(group,group),rows)
            assert check.fixtures.render_rows(group,rows)==inherited
        rendered,entries=check.module_source(group,rows)
        assert len(entries)==len(set(entries))==2*len(rows)
        for number in re.findall(r'(?<![\w])(?:0[xX][0-9a-fA-F]+|\d+)(?![\w])',rendered):
            assert int(number,16 if number.lower().startswith('0x') else 10)<1<<64
        groups.append(dict(group=group,count=len(rows),cases=[row['name'] for row in rows],
                           source_sha256=check.text_sha(rendered),selections=entries))
    assert sum(group['count'] for group in groups)==646
    assert before==check.snapshot()
    return dict(stage='source scope and host generation only; no Omega execution',
                execution_validation=False,source_unchanged=True,base_revision=BASE,
                input_sha256=before,changed_production_files=sorted(CHANGED),
                unchanged_production_files=sorted(current-CHANGED),
                unchanged_borrowed_inputs=borrowed,transition_scopes=scopes,
                groups=groups,positive_count=646,control_count=646)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record',type=check.Path)
    args=parser.parse_args()
    record=run()
    if args.record:args.record.write_text(json.dumps(record,indent=2)+'\n')
    print('PASS source/generation audit:',record['positive_count'],'pairs;',len(record['input_sha256']),
          'bound inputs; no compilation or execution claim')


if __name__=='__main__':main()
