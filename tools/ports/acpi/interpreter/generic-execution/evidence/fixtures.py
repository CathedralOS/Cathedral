"""Reproduce final authored fixture bytes without invoking the compiler."""
import importlib.util
import sys
from pathlib import Path

def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    original = sys.path[:]
    try:
        sys.path.insert(0, str(path.parent))
        spec.loader.exec_module(module)
    finally:
        sys.path[:] = original
    return module

def reproduce(root, kind, receipt):
    here = root / 'tools/ports/acpi/interpreter/generic-execution'
    if kind == 'generic':
        fixtures = load(here / 'fixtures.py', 'evidence_generic_fixtures')
        rows = [row for row in fixtures.cases() if row['name'] in receipt['cases']]
        assert [row['name'] for row in rows] == receipt['cases']
        source, selections = fixtures.render(rows)
        assert source == (here / 'main.omg').read_text()
        return [source], selections
    if kind == 'focused':
        directory = here / 'focused' / receipt['group']
        return [(directory / 'main.omg').read_text()], (directory / 'selections.txt').read_text().splitlines()
    if kind == 'integer':
        fixtures = load(root / 'tools/ports/acpi/interpreter/execution/fixtures.py', 'evidence_integer_fixtures')
        rows = [row for row in fixtures.cases() if row['name'] in receipt['cases']]
        assert [row['name'] for row in rows] == receipt['cases']
        prefix = None
        bodies, selections = [], []
        for row in rows:
            for negative in [False, True]:
                source = fixtures.render(row, False)
                if negative:
                    numeric = row['expected'] is not None and row['error'] == 'Success'
                    old = f'result.value.number == {row["expected"]}' if numeric else f'result.outcome == ExecutionOutcome::{row["error"]}'
                    new = f'result.value.number == {(row["expected"] + 1) & ((1 << row["bits"]) - 1)}' if numeric else f'result.outcome == ExecutionOutcome::{"BadEncoding" if row["error"] == "Success" else "Success"}'
                    assert source.count(old) == 1
                    source = source.replace(old, new)
                if prefix is None:
                    prefix = source[:source.index('machine test_result()')] + 'data Suite {}\n'
                machine = 'Suite::' + row['name'] + ('_control' if negative else '_positive')
                body = source[source.index('machine test_result()'):source.index('data Main {}')]
                bodies.append(body.replace('machine test_result()', 'machine ' + machine + '(&mut self)', 1))
                selections.append(machine + '=' + str(int(negative)))
        return [prefix + ''.join(bodies)], selections
    if kind == 'pipeline':
        fixtures = load(root / 'tools/ports/acpi/pipeline/fixtures.py', 'evidence_pipeline_fixtures')
        rows = [row for row in fixtures.cases() if row['name'] in receipt['cases']]
        assert [row['name'] for row in rows] == receipt['cases']
        source, selections = [fixtures.IMPORTS, 'data PipelineSuite {}\n'], []
        for row in rows:
            for control in [False, True]:
                machine = 'PipelineSuite::' + row['name'] + ('_control' if control else '_positive')
                source.append(fixtures.render(row, control, machine))
                selections.append(machine + '=' + str(int(control)))
        return [''.join(source)], selections
    assert kind == 'kernel_const'
    source = (here / 'kernels.omg').read_text()
    return [source + f'\nconst TEST_RESULT:i32=gk_check(0,{number});\nmachine require_success(value:i32) requires value==0; {{}}\ndata Main {{}}\nmachine Main::main(&mut self) {{require_success(TEST_RESULT);}}\n' for number in [65, 66]], []
