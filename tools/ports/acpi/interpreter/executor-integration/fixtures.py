"""Compose unchanged component cases with explicit expanded-Frame coverage."""
import importlib.util
from pathlib import Path

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
def load(label,path):
    spec=importlib.util.spec_from_file_location(label,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

mid=load('executor_integration_mid',HERE.parent/'mid-execution/fixtures.py')
logical=load('executor_integration_logical',HERE.parent/'logical-execution/fixtures.py')
to_string=load('executor_integration_to_string',HERE.parent/'to-string-execution/fixtures.py')
concat=load('executor_integration_concat',HERE.parent/'concat-execution/fixtures.py')
write_retirement=load('executor_integration_write_retirement',HERE.parent/'field-write-execution/fixtures.py')
write_pipeline=load('executor_integration_write_pipeline',HERE.parent.parent/'pipeline/field-writes/fixtures.py')
mixed=load('executor_integration_mixed',HERE/'mixed_fixtures.py')

GROUPS=['mixed','frame','composed_writes','logical','logical_bridge','to_string','to_string_bridge',
        'concat','write_retirement','write_pipeline','mid','mid_bridge','integer','generic','pipeline','to_integer']
NAMED,GENERIC=mid.NAMED,mid.GENERIC

def frames():return load('executor_integration_frame_coverage',HERE/'frame_coverage.py')
def writes():return load('executor_integration_pipeline_fixtures',HERE/'pipeline_fixtures.py')
def rows(group):
    if group=='mixed':return mixed.cases()
    if group=='frame':return frames().control_cases()
    if group=='composed_writes':return writes().cases()
    if group.startswith('logical'):return logical.rows('bridge' if group.endswith('_bridge') else 'execution')
    if group.startswith('to_string'):return to_string.rows('bridge' if group.endswith('_bridge') else 'execution')
    if group in ('concat','write_retirement','write_pipeline'):return globals()[group].cases()
    return mid.rows({'mid':'execution','mid_bridge':'bridge'}.get(group,group))

def original_source(group,selected):
    if group=='mixed':return mixed.render(selected)
    if group=='frame':return frames().render_controls(selected)
    if group=='composed_writes':return writes().render(selected)
    if group.startswith('logical'):return logical.render_rows('bridge' if group.endswith('_bridge') else 'execution',selected)
    if group.startswith('to_string'):return to_string.render_rows('bridge' if group.endswith('_bridge') else 'execution',selected)
    if group in ('concat','write_retirement','write_pipeline'):
        actual,source,entries=globals()[group].render(','.join(row['name'] for row in selected))
        assert actual==selected
        return source,entries
    return mid.render_rows({'mid':'execution','mid_bridge':'bridge'}.get(group,group),selected)

def render_rows(group,selected):
    source,entries=original_source(group,selected)
    if 'machine ba_frame(' in source:source=frames().augment(source)
    if group in ('logical_bridge','to_string_bridge','mid_bridge','frame'):
        assert 'a.field_writes==b.field_writes' in source
        assert 'a.deferred_write,b.deferred_write' in source
    return source,entries
