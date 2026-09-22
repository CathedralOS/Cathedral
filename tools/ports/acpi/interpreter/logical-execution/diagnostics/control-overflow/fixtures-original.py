"""Encoded logical programs with independent Python primary-policy expectations."""
import importlib.util
import itertools
import json
from pathlib import Path

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
def load(label,path):
    spec=importlib.util.spec_from_file_location(label,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
mid=load('logical_mid_fixtures',HERE.parent/'mid-execution/fixtures.py')
ns=mid.ns
NAMED,GENERIC=mid.NAMED,mid.GENERIC
integer,pkg,ret,op,name=mid.integer,mid.pkg,mid.ret,mid.op,mid.name
MAX=(1<<64)-1
OPS={'and':0x90,'or':0x91,'not':0x92,'eq':0x93,'gt':0x94,'lt':0x95,'ne':0x9293,'le':0x9294,'ge':0x9295}

def encoded(value):
    kind,data=value
    if kind=='integer':return integer(data)
    if kind=='string':return b'\x0d'+data+b'\0'
    return pkg(0x11,integer(len(data))+data)

def expression(operation,*operands):
    opcode=OPS[operation]
    prefix=bytes([opcode]) if opcode<256 else opcode.to_bytes(2,'big')
    return prefix+b''.join(operands)

def implicit(value,target,bits):
    kind,data=value
    if kind=='integer':data&=(1<<bits)-1
    if kind=='string' and any(b==0 or b>=128 for b in data):raise ValueError('Encoding')
    if target=='integer':
        if kind=='integer':return data
        if not data:raise ValueError('Empty')
        if kind=='buffer':return int.from_bytes(data[:bits//8],'little')
        digits=''
        for byte in data[:bits//4]:
            ch=chr(byte)
            if ch not in '0123456789abcdefABCDEF':break
            digits+=ch
        return int(digits,16) if digits else 0
    if target=='buffer':
        output=data.to_bytes(bits//8,'little') if kind=='integer' else data+(b'\0' if kind=='string' and data else b'')
    elif kind=='integer':output=f'{data:0{bits//4}X}'.encode()
    elif kind=='buffer':output=b' '.join(f'{byte:02X}'.encode() for byte in data)
    else:output=data
    if len(output)>256:raise ValueError('Capacity')
    return output

def expected(operation,left,right,bits):
    try:
        target='integer' if operation in ('and','or','not') else left[0]
        a=implicit(left,target,bits)
        if operation=='not':yes=a==0
        else:
            b=implicit(right,target,bits)
            yes={'and':lambda:bool(a) and bool(b),'or':lambda:bool(a) or bool(b),
                 'eq':lambda:a==b,'ne':lambda:a!=b,'gt':lambda:a>b,'lt':lambda:a<b,
                 'le':lambda:a<=b,'ge':lambda:a>=b}[operation]()
        return ('Success',((1<<bits)-1) if yes else 0)
    except ValueError as error:return (str(error),0)

def execution_cases():
    rows=[]
    def add(label,operation,left,right=('integer',0),bits=64,body=None,methods=b'',setup=None,error=None,number=None,after=None,count=3):
        want_error,want_number=expected(operation,left,right,bits)
        operands=[name('LEFT')] if operation=='not' else [name('LEFT'),name('RGHT')]
        body=ret(expression(operation,*operands)) if body is None else body
        table=b'\x08'+name('LEFT')+encoded(left)+b'\x08'+name('RGHT')+encoded(right)+pkg(0x14,name('MAIN')+b'\0'+body)+methods
        rows.append(dict(name=label,table=table.hex(),kind=1,number=want_number if number is None else number,bytes='',bits=bits,
            after=after or {},object_count=count,error=want_error if error is None else error,note='',setup=setup or [],mutate_source=False))
    lefts=[('integer',0x100000001),('string',b'10'),('buffer',b'\x02\0\0\0\x01')]
    rights=[('integer',2),('string',b'2'),('buffer',b'\x03')]
    for bits in (32,64):
        for operation in OPS:
            pairs=[(left,('integer',0)) for left in lefts] if operation=='not' else itertools.product(lefts,rights)
            for left,right in pairs:
                add(f'{operation}_{left[0]}_{right[0]}_{bits}',operation,left,right,bits)
        for label,operation,left,right in [
            ('and_false','and',('integer',0),('integer',7)),
            ('or_false','or',('integer',0),('integer',0)),
            ('or_right_true','or',('integer',0),('string',b'1')),
            ('width_truth','and',('buffer',b'\0\0\0\0\x01'),('integer',1)),
            ('width_not','not',('buffer',b'\0\0\0\0\x01'),('integer',0)),
            ('hex_prefix','not',('string',b'0x12'),('integer',0)),
            ('numeric_prefix','and',('string',b'12Z'),('integer',1)),
            ('and_right_empty','and',('buffer',b'\0'),('buffer',b'')),
            ('or_right_empty','or',('buffer',b'\x01'),('buffer',b'')),
            ('unary_empty_buffer','not',('buffer',b''),('integer',0)),
            ('unary_empty_string','not',('string',b''),('integer',0)),
            ('lexical_buffer','lt',('buffer',b'\xff'),('buffer',b'\0\0')),
            ('prefix_buffer','lt',('buffer',b'A'),('buffer',b'AB')),
            ('empty_buffers','eq',('buffer',b''),('buffer',b'')),
            ('empty_strings','eq',('string',b''),('string',b'')),
            ('full_strings','eq',('string',b'A'*256),('string',b'A'*256)),
            ('full_buffers','gt',('buffer',b'A'*256),('buffer',b'A'*255)),
            ('right_string_capacity','eq',('buffer',b''),('string',b'A'*256)),
            ('right_buffer_capacity','eq',('string',b''),('buffer',b'A'*86)),
            ('unsigned','gt',('integer',MAX),('integer',1)),
            ('normalize_equal','eq',('integer',0x100000001),('integer',1))]:
            add(f'{label}_{bits}',operation,left,right,bits)
        add(f'inline_pair_{bits}','lt',('integer',7),('integer',9),bits,body=ret(expression('lt',integer(7),integer(9))))
        add(f'inline_right_string_{bits}','eq',('string',f'{1:0{bits//4}X}'.encode()),('integer',1),bits,body=ret(expression('eq',name('LEFT'),integer(1))))
        add(f'inline_right_buffer_{bits}','eq',('buffer',(1).to_bytes(bits//8,'little')),('integer',1),bits,body=ret(expression('eq',name('LEFT'),integer(1))))
        add(f'nested_{bits}','not',('integer',1),bits=bits,body=ret(expression('not',expression('eq',integer(9),integer(9)))))
        add(f'local_scalar_{bits}','eq',('integer',7),('integer',7),bits,body=op(0x9d,integer(7),b'\x60')+ret(expression('eq',b'\x60',integer(7))))
        add(f'no_parent_{bits}','or',('integer',1),('integer',0),bits,body=expression('or',name('LEFT'),name('RGHT'))+ret(integer(0)),number=0)
        for operation,first,want in [('and',0,0),('or',1,(1<<bits)-1)]:
            add(f'{operation}_evaluates_callee_{bits}',operation,('integer',first),('integer',0),bits,
                body=ret(expression(operation,integer(first),name('CAL'))),
                methods=pkg(0x14,name('CAL')+b'\0'+op(0x70,integer(1),name('RGHT'))+ret(integer(1))),
                after={'RGHT':1},number=want,count=4)
        add(f'callee_argument_{bits}','not',('buffer',b'\0\0\0\0\x01'),bits=bits,
            body=ret(name('CAL')+name('LEFT')),methods=pkg(0x14,name('CAL')+b'\x01'+ret(expression('not',b'\x68'))),count=4)
        add(f'late_left_identity_{bits}','gt',('buffer',b'\0'),('string',b'F'),bits,
            body=ret(expression('gt',name('LEFT'),name('CAL'))),
            methods=pkg(0x14,name('CAL')+b'\0'+op(0x9d,name('RGHT'),name('LEFT'))+ret(integer(15))),number=(1<<bits)-1,count=4)
    for kind in ['Named','Local','Arg']:
        add('transparent_'+kind.lower(),'eq',('integer',0),('buffer',b'A'),
            setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'],number=MAX)
    for kind in ['RefOf','Index']:
        add('explicit_'+kind.lower(),'eq',('integer',0),('integer',0),
            setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'],error='UnsupportedValue',number=0)
    add('source_cycle','eq',('integer',0),setup=['program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};'],error='InvalidState')
    add('full_arena_inline','eq',('integer',1),('integer',1),setup=['program.store.space.object_count=64;'],count=64)
    add('full_arena_bytes','eq',('buffer',b'A'),('buffer',b'A'),setup=['program.store.space.object_count=64;'],count=64)
    add('bad_owned_source','eq',('buffer',b'A'),('buffer',b'A'),setup=['program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};'],error='InvalidState',number=0)
    return rows

def bridge_module():return load('logical_bridge_fixtures',HERE/'bridge_fixtures.py')
def rows(group):
    if group=='execution':return execution_cases()
    if group=='bridge':return bridge_module().cases()
    return mid.rows({'mid':'execution','mid_bridge':'bridge'}.get(group,group))
def render_rows(group,selected):
    if group=='execution':return ns.base.render(selected)
    if group=='bridge':return bridge_module().render(selected)
    return mid.render_rows({'mid':'execution','mid_bridge':'bridge'}.get(group,group),selected)

if __name__=='__main__':
    selected=execution_cases()
    assert len({r['name'] for r in selected})==len(selected)
    (HERE/'execution-cases.json').write_text(json.dumps(selected,indent=2)+'\n')
    print('execution',len(selected),'authored behavior/control pairs; not execution proof')
