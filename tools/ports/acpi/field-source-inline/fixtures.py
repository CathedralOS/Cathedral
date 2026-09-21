#!/usr/bin/env python3
"""Inline Integer payloads with independent unsigned-bit expectations."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
BASE=HERE.parent/'field-sources'
sys.path.insert(0,str(BASE))
spec=importlib.util.spec_from_file_location('previous_fixtures',BASE/'fixtures.py')
previous=importlib.util.module_from_spec(spec)
spec.loader.exec_module(previous)
from fixture_common import initialized
MAX=(1<<64)-1


def make(number,size,width,ordinal):
    if width==0:
        expected=dict(error='Bounds')
    elif width>2048:
        expected=dict(error='Capacity')
    elif ordinal:
        expected=dict(end=True,total=1)
    else:
        value=number & ((1<<(size*8))-1) & ((1<<width)-1)
        length=(width+7)//8
        expected=dict(total=1,ordinal=0,length=length,bytes=list(value.to_bytes(length,'little'))+[0]*(256-length))
    return dict(name=f'integer_{size}_{width}_{number}_{ordinal}',number=number,size=size,width=width,ordinal=ordinal,expected=expected)


def cases():
    rows=[]
    for size in [4,8]:
        for width in [1,7,8,9,31,32,33,63,64,65,255,256,257,2047,2048]:
            for number in [0,1,0x8000000100000001,MAX]:
                rows.append(make(number,size,width,0))
        for width in [1,65,2048]:
            for ordinal in [1,2,1<<63,MAX]:
                rows.append(make(MAX,size,width,ordinal))
        for width in [0,2049,1<<63,MAX]:
            for ordinal in [0,1,MAX]:
                rows.append(make(MAX,size,width,ordinal))
    assert len(rows)==len({row['name'] for row in rows})
    return rows


HEAD=previous.HEAD+'use sources::sequence::integer_payload_at;\n'


def body(row,control=False):
    size='FourBytes' if row['size']==4 else 'EightBytes'
    source=f'let value:PayloadResult=integer_payload_at({row["number"]},IntegerSize::{size},{row["width"]},{row["ordinal"]});'
    expected=row['expected']
    if 'error' in expected:
        error=expected['error'] if not control else 'Capacity' if expected['error']=='Bounds' else 'Bounds'
        source+=f'let good:bool=expected_failure(value,ConversionFailure::{error});'
    elif expected.get('end'):
        source+=f'let good:bool=expected_end(value,{1^int(control)});'
    else:
        out=list(expected['bytes'])
        out[-1]^=int(control)
        source+=initialized('expected',out)+f'let good:bool=expected_payload(value,1,0,{expected["length"]},&expected);'
    return source+'transition good {true -> (0) _ -> (1)}'


def render(rows):
    source=HEAD+'data Suite {}\n'
    entries=[]
    for row in rows:
        for control in [False,True]:
            name='Suite::'+row['name']+('_control' if control else '_positive')
            source+='machine '+name+'(&mut self)->i32 {'+body(row,control)+'}\n'
            entries.append(name+'='+str(int(control)))
    return source,entries


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write',action='store_true')
    args=parser.parse_args()
    rows=cases()
    if args.write:
        (HERE/'cases.json').write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n')
    else:
        assert json.loads((HERE/'cases.json').read_text())==rows
    print('PASS',len(rows),'inline Integer fixture pairs')
