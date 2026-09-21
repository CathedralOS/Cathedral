#!/usr/bin/env python3
"""Authored Omega assertions for complete recipes and initialized array tails."""
import argparse
import hashlib
import json
from pathlib import Path

import vectors

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
HEAD = '''use protocol::protocol;
use protocol::model::Request;
use protocol::model::Register;
use protocol::model::Recipe;
use protocol::model::ProtocolResult;
use protocol::model::Action;
use protocol::model::Component;
use protocol::model::Direction;
use access::model::Plan;
use access::model::Chunk;
use access::model::ReadShape;
use access::model::LockRequirement;
use access::model::Error;
use fields::field_model::Field;
use fields::field_model::AccessResult;
use fields::field_model::AccessType;
use fields::field_model::UpdateRule;
use fields::field_model::DeclarationKind;
use fields::field_model::Connection;
use fields::flags;
use aml::model::Path;
use integers::integers::IntegerSize;
machine same_chunk(a:Chunk,b:Chunk)->bool {a.offset==b.offset && a.width==b.width && a.field_bit==b.field_bit && a.native_bit==b.native_bit && a.bit_count==b.bit_count && a.mask==b.mask && a.partial==b.partial}
machine same_chunks(a:&[Chunk;257],b:&[Chunk;257],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=chunk_at(a,b,index);transition index<count {true -> same_chunks(a,b,index+1,count,prior && good) _ -> (prior)}}
machine chunk_at(a:&[Chunk;257],b:&[Chunk;257],index:u64)->bool {transition index<257 {true -> compare(a[index],b[index]) _ -> (true)} state compare(a:Chunk,b:Chunk)->bool {let good:bool=same_chunk(a,b);good}}
machine same_shape(a:ReadShape,b:ReadShape)->bool {
 transition a {ReadShape::Integer {size} -> integer(size,b) ReadShape::Buffer {bytes} -> buffer(bytes,b)}
 state integer(expected:IntegerSize,other:ReadShape)->bool {transition other {ReadShape::Integer {size} -> (size==expected) _ -> (false)}}
 state buffer(expected:u64,other:ReadShape)->bool {transition other {ReadShape::Buffer {bytes} -> (bytes==expected) _ -> (false)}}
}
machine same_plan(plan:&Plan,expected:&Plan)->bool {
 let rows:bool=same_chunks(&plan.chunks,&expected.chunks,0,257,true);let shape:bool=same_shape(plan.shape,expected.shape);
 plan.access==expected.access && plan.update==expected.update && plan.lock==expected.lock && shape && plan.bit_offset==expected.bit_offset && plan.bit_length==expected.bit_length && plan.region_bytes==expected.region_bytes && plan.start==expected.start && plan.end==expected.end && plan.width==expected.width && plan.count==expected.count && plan.preserve_reads==expected.preserve_reads && rows
}
machine same_action(a:Action,b:Action)->bool {
 transition a {Action::None -> none(b) Action::Select {value} -> select(value,b) Action::ReadDatum {chunk} -> read(chunk,b) Action::WriteDatum {chunk} -> write(chunk,b)}
 state none(other:Action)->bool {transition other {Action::None -> (true) _ -> (false)}}
 state select(expected:u64,other:Action)->bool {transition other {Action::Select {value} -> (value==expected) _ -> (false)}}
 state read(expected:u64,other:Action)->bool {transition other {Action::ReadDatum {chunk} -> (chunk==expected) _ -> (false)}}
 state write(expected:u64,other:Action)->bool {transition other {Action::WriteDatum {chunk} -> (chunk==expected) _ -> (false)}}
}
machine same_actions(a:&[Action;1028],b:&[Action;1028],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=action_at(a,b,index);transition index<count {true -> same_actions(a,b,index+1,count,prior && good) _ -> (prior)}}
machine action_at(a:&[Action;1028],b:&[Action;1028],index:u64)->bool {transition index<1028 {true -> compare(a[index],b[index]) _ -> (true)} state compare(a:Action,b:Action)->bool {let good:bool=same_action(a,b);good}}
machine good_recipe(result:ProtocolResult,expected:&Recipe)->bool {
 transition result {ProtocolResult::Ready {recipe} -> compare(&recipe,expected) _ -> (false)}
 state compare(recipe:&Recipe,expected:&Recipe)->bool {
  let outer:bool=same_plan(&recipe.outer,&expected.outer);let selector:bool=same_plan(&recipe.selector,&expected.selector);let data:bool=same_plan(&recipe.data,&expected.data);
  let actions:bool=same_actions(&recipe.actions,&expected.actions,0,1028,true);
  recipe.kind==expected.kind && recipe.direction==expected.direction && recipe.bank_value==expected.bank_value && recipe.lock==expected.lock && recipe.count==expected.count && outer && selector && data && actions
 }
}
machine bad_recipe(result:ProtocolResult,wanted:Component,expected:Error)->bool {transition result {ProtocolResult::Failure {component,error} -> (component==wanted && error==expected) _ -> (false)}}
machine overflow_recipe(result:ProtocolResult)->bool {transition result {ProtocolResult::SelectorOverflow -> (true) _ -> (false)}}
'''


def field_text(name, row):
    text = f'let {name}_access:AccessResult=flags::decode_flags({row["flags"]});let mut {name}:Field=Field {{bit_offset:{row["offset"]},bit_length:{row["length"]},access:{name}_access.access}};{name}.access.flags={row["flags"]};'
    patches = {
        'connection': '.connection=Connection::Name {connection_name:Path {}};',
        'attribute': '.access.attribute=1;',
        'attribute_mode': '.access.attribute_mode=1;',
        'extended': '.access.extended=true;',
        'access_length': '.access.access_length=1;',
        'kind_mismatch': '.access.kind=AccessType::Buffer;',
        'update_mismatch': '.access.update=UpdateRule::WriteAsOnes;',
        'lock_mismatch': '.access.locked=true;',
    }
    if row.get('patch'):
        text += name + patches[row['patch']]
    return text


def setup(row):
    text = ''.join(field_text(name, row[name]) for name in ('field', 'selector', 'data'))
    size = 'FourBytes' if row['size'] == 4 else 'EightBytes'
    text += f'let request:Request=Request {{kind:DeclarationKind::{row["kind"]},direction:Direction::{row["direction"]},size:IntegerSize::{size},field:field,region_bytes:{row["region"]},bank_value:{row["bank_value"]},'
    for name in ('selector', 'data'):
        text += f'{name}:Register {{kind:DeclarationKind::{row[name].get("kind", "Field")},field:{name},region_bytes:{row[name+"_region"]}}},'
    return text + '};let result:ProtocolResult=protocol::plan(&request);'


def chunk_text(row):
    return 'Chunk {' + ','.join(key + ':' + str(value).lower() for key, value in row.items()) + '}'


def plan_text(name, field, region, size, expected):
    if expected is None:
        return ''
    integer = 'FourBytes' if size == 4 else 'EightBytes'
    shape = f'ReadShape::Integer {{size:IntegerSize::{integer}}}' if expected['shape'] == 'Integer' else f'ReadShape::Buffer {{bytes:{expected["bytes"]}}}'
    access = ['Any', 'Byte', 'Word', 'DWord', 'QWord'][field['flags'] & 15]
    update = ['Preserve', 'WriteAsOnes', 'WriteAsZeros'][(field['flags'] >> 5) & 3]
    lock = 'UnmetGlobalLock' if expected['locked'] else 'NotRequested'
    text = f'expected.{name}=Plan {{access:AccessType::{access},update:UpdateRule::{update},lock:LockRequirement::{lock},shape:{shape},bit_offset:{field["offset"]},bit_length:{field["length"]},region_bytes:{region},'
    text += ','.join(f'{key}:{expected[key]}' for key in ('start', 'end', 'width', 'count', 'preserve_reads')) + '};'
    for index, chunk in enumerate(expected['chunks']):
        text += f'expected.{name}.chunks[{index}]={chunk_text(chunk)};'
    return text


def body(row, control=False):
    text = setup(row)
    expected = row['expected']
    if 'error' in expected:
        error = expected['error']
        component = error['component']
        geometry = error['geometry']
        if control:
            if hashlib.sha256(row['name'].encode()).digest()[0] % 2:
                component = 'Selector' if component == 'Field' else 'Field'
            else:
                geometry = 'InvalidChunk'
        check = f'bad_recipe(result,Component::{component},Error::{geometry})'
    elif 'protocol' in expected:
        check = 'bad_recipe(result,Component::Selector,Error::InvalidChunk)' if control else 'overflow_recipe(result)'
    else:
        lock = 'UnmetGlobalLock' if expected['lock'] else 'NotRequested'
        text += f'let mut expected:Recipe=Recipe {{kind:DeclarationKind::{row["kind"]},direction:Direction::{row["direction"]},bank_value:{expected["bank_value"]},lock:LockRequirement::{lock},count:{len(expected["actions"])}}};'
        text += plan_text('outer', row['field'], row['region'] if row['kind'] == 'Bank' else (1 << 64)-1, row['size'], expected['outer'])
        for name in ('selector', 'data'):
            text += plan_text(name, row[name], row[name+'_region'], row['size'], expected[name])
        for index, action in enumerate(expected['actions']):
            value = f'Select {{value:{action["value"]}}}' if action['kind'] == 'Select' else f'{action["kind"]}Datum {{chunk:{action["index"]}}}'
            text += f'expected.actions[{index}]=Action::{value};'
        # Independently exercise live action fields, ordering tags, metadata,
        # component geometry and inactive tails across the checked corpus.
        if control:
            choice = hashlib.sha256(row['name'].encode()).digest()[0] % 9
            first = expected['actions'][0]['value']
            datum = expected['actions'][1]
            text += [
                'expected.actions[1027]=Action::Select {value:1};',
                f'expected.actions[0]=Action::Select {{value:{first ^ 1}}};',
                f'expected.actions[1]=Action::{datum["kind"]}Datum {{chunk:1}};',
                f'expected.actions[1]=Action::{"Write" if datum["kind"]=="Read" else "Read"}Datum {{chunk:0}};',
                f'expected.count={len(expected["actions"]) ^ 1};',
                f'expected.direction=Direction::{"Write" if row["direction"]=="Read" else "Read"};',
                f'expected.outer.chunks[0].mask={expected["outer"]["chunks"][0]["mask"] ^ 1};',
                f'expected.selector.chunks[0].mask={expected["selector"]["chunks"][0]["mask"] ^ 1};',
                'expected.data.chunks[256].mask=1;',
            ][choice]
        check = 'good_recipe(result,&expected)'
    return text + f'let good:bool={check};transition good {{true -> (0) _ -> (1)}}\n'


def render(rows):
    text = HEAD + 'data Suite {}\n'
    selections = []
    for row in rows:
        for control in (False, True):
            name = 'Suite::' + row['name'] + ('_control' if control else '_positive')
            text += f'machine {name}(&mut self)->i32 {{\n{body(row, control)}}}\n'
            selections.append(name + '=' + str(int(control)))
    return text, selections


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    text = json.dumps(vectors.cases(), indent=2, sort_keys=True) + '\n'
    path = HERE / 'cases.json'
    if args.check:
        assert path.read_text() == text
    else:
        path.write_text(text)
    print('PASS', len(vectors.cases()), 'deterministic field protocol cases')


if __name__ == '__main__':
    main()
