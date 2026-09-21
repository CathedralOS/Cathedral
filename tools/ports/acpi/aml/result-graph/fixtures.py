"""Original reachable-object graph witnesses, with independent expected quotas."""
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
MAX=(1<<64)-1


def cases():
    rows=[]
    def add(name, setup='', objects=1, byte_count=0, visited=1, error='', root=0, count=1, object_budget=64, byte_budget=16384, length=4):
        rows.append(dict(name=name,setup=setup,objects=objects,bytes=byte_count,visited=visited,error=error,root=root,count=count,object_budget=object_budget,byte_budget=byte_budget,length=length))
    def owned(i,length,kind='Buffer',owner=None):
        owner=i if owner is None else owner
        member='buffer' if kind=='Buffer' else 'string'
        return f'store.space.objects[{i}].value=Value::{kind} {{{member}_storage:{kind}Storage::Owned {{{member}_owner:{owner}}}}};store.bytes.blocks[{i}]=ByteBlock {{initialized:true,length:{length}}};'
    def package(i,first,count):
        return f'store.space.objects[{i}].value=Value::Package {{first:{first},count:{count}}};'
    def ref(i,j,kind='RefOf'):
        return f'store.space.objects[{i}].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:{j}}};'
    add('integer')
    add('integer_zero_object_budget',object_budget=0,error='WorkLimit')
    add('uninitialized_inert','store.space.objects[0].value=Value::Uninitialized;')
    add('empty_package',package(0,MAX,0),object_budget=1,byte_budget=0)
    add('empty_owned_buffer',owned(0,0),object_budget=1,byte_budget=0)
    add('owned_buffer',owned(0,4),byte_count=4,byte_budget=4)
    add('byte_budget_one_short',owned(0,4),error='Capacity',byte_budget=3)
    add('owned_string',owned(0,2,'String')+'store.bytes.blocks[0].bytes[0]=65;store.bytes.blocks[0].bytes[1]=66;',byte_count=2)
    add('invalid_string_tail',owned(0,2,'String')+'store.bytes.blocks[0].bytes[0]=65;store.bytes.blocks[0].bytes[1]=255;',error='Encoding')
    add('source_buffer_padding','store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:7,buffer_initializer:Span {unit:9,end:4}}};',byte_count=7,byte_budget=7)
    add('source_string','store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:9,end:4}}};',byte_count=4)
    for kind in ['Named','Local','Arg','RefOf','Index','Unresolved']:
        add('reference_'+kind.lower(),ref(0,1,kind)+owned(1,3),objects=2,byte_count=3,visited=3,count=2,object_budget=2)
    add('reference_bad_owner',ref(0,1)+owned(1,3,owner=0),error='InvalidState',count=2)
    add('reference_dangling',ref(0,MAX),error='InvalidState')
    add('opaque_self_cycle',ref(0,0),object_budget=1)
    add('reference_cycle',ref(0,1)+ref(1,0),objects=2,visited=3,count=2,object_budget=2)
    links='store.space.objects[1].has_next=true;store.space.objects[1].next=2;'
    outer=package(0,1,2)+links
    add('nested_bytes',outer+owned(1,3)+owned(2,4),objects=3,byte_count=7,visited=7,count=3,object_budget=3,byte_budget=7)
    add('nested_byte_budget',outer+owned(1,3)+owned(2,4),error='Capacity',count=3,byte_budget=6)
    add('nested_object_budget',outer+owned(1,3)+owned(2,4),error='WorkLimit',count=3,object_budget=2)
    add('nested_bad_owner',outer+owned(1,3)+owned(2,4,owner=1),error='InvalidState',count=3)
    add('nested_package',package(0,1,1)+package(1,2,1)+owned(2,5),objects=3,byte_count=5,visited=7,count=3,object_budget=3)
    add('nested_package_bad_tail',package(0,1,1)+package(1,2,2),count=3,error='InvalidState')
    add('nested_package_oversize',package(0,1,1)+package(1,2,MAX),count=3,error='Capacity')
    add('shared_reference_bytes',outer+ref(1,3)+ref(2,3)+owned(3,5),objects=4,byte_count=5,visited=15,count=4,object_budget=4,byte_budget=5)
    add('shared_nested_package',outer+package(1,3,1)+package(2,3,1)+owned(3,5),objects=4,byte_count=5,visited=15,count=4,object_budget=4,byte_budget=5)
    add('package_self_graph',package(0,0,1),object_budget=1)
    add('sibling_cycle',package(0,1,2)+'store.space.objects[1].has_next=true;store.space.objects[1].next=1;',count=2,error='ReferenceCycle')
    add('extra_tail',package(0,1,1)+'store.space.objects[1].has_next=true;store.space.objects[1].next=0;',count=2,error='InvalidState')
    add('root_last_slot','store.space.objects[63].value=Value::Integer {number:8};',root=63,count=64,visited=1<<63)
    add('root_max',root=MAX,error='InvalidState')
    add('count_max',count=MAX,error='Capacity')
    add('object_budget_max',object_budget=MAX,error='Capacity')
    add('byte_budget_max',byte_budget=MAX,error='Capacity')
    add('length_max',length=MAX,error='Capacity')
    add('owned_uninitialized','store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};',error='InvalidState')
    add('owned_capacity',owned(0,257),error='Capacity')
    add('source_wrong_unit','store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:0,buffer_initializer:Span {unit:8,end:4}}};',error='Bounds')
    add('source_bad_bounds','store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:9,start:5,end:4}}};',error='Bounds')
    add('lexical_forward','let mut name:Path=Path {count:1};name.segments[0]=1598314310;store.space.objects[0].value=Value::NameReference {name:name,scope:Path {absolute:true}};')
    add('lexical_bad_scope','store.space.objects[0].value=Value::NameReference {name:Path {},scope:Path {}};',error='InvalidState')
    add('unsupported_service','store.space.objects[0].value=Value::OperationRegion {};',error='UnsupportedValue')
    add('unrelated_bad_object','store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:MAX}};'.replace('MAX',str(MAX)),count=2)
    setup=package(0,1,63)
    for i in range(1,64):
        setup+=owned(i,256)
        if i<63:setup+=f'store.space.objects[{i}].has_next=true;store.space.objects[{i}].next={i+1};'
    add('full_graph',setup,objects=64,byte_count=63*256,visited=MAX,count=64,object_budget=64,byte_budget=63*256)
    add('full_graph_budget',setup,count=64,object_budget=63,error='WorkLimit')
    return rows


def render(rows):
    source='''module main;
use aml::model::ObjectStore;
use aml::model::Value;
use aml::model::ByteBlock;
use aml::model::BufferStorage;
use aml::model::StringStorage;
use aml::model::ReferenceKind;
use aml::model::Path;
use aml::model::Span;
use aml::result_graph::GraphResult;
use aml::result_graph::check_graph;
use aml::object_conversions::ConversionFailure;
data Suite {}
'''
    names=[]
    for row in rows:
        for negative in (False,True):
            name='Suite::'+row['name']+('_control' if negative else '_positive')
            names.append(name+'='+str(int(negative)))
            source+=f'machine {name}(&mut self)->i32 {{let mut input:[u8;1024];input[0]=65;input[1]=66;input[2]=67;input[3]=68;let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={row["count"]};store.space.objects[0].value=Value::Integer {{number:9}};'+row['setup']
            source+=f'let found:GraphResult=check_graph(&input,{row["length"]},9,&store,{row["root"]},{row["object_budget"]},{row["byte_budget"]});'
            if row['error']:
                error='Overflow' if negative else row['error']
                source+=f'transition found {{GraphResult::Failure {{reason}} -> verdict(reason==ConversionFailure::{error}) _ -> (1)}}'
            else:
                visited=row['visited']^(1<<63) if negative else row['visited']
                source+=f'transition found {{GraphResult::Admitted {{objects,bytes,visited}} -> verdict(objects=={row["objects"]} && bytes=={row["bytes"]} && visited=={visited}) _ -> (1)}}'
            source+='state verdict(good:bool)->i32 {transition good {true -> (0) _ -> (1)}}}\n'
    return source,names

if __name__=='__main__':
    (HERE/'cases.json').write_text(json.dumps(cases(),indent=2)+'\n')
    print(len(cases()),'graph cases')
