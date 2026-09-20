#!/usr/bin/env python3
"""Original fixed ACPI byte scenarios and semantic assertions; no firmware inputs."""
import argparse
import json
import re
from header_fixtures import sdt, fix
from fixed_model import HERE, ROOT, records, rust_measure, MADT_KINDS
from fixed_generate import carrier, snake

IMPORTS={
 'gas':['RawGenericAddress','GenericAddress','RawGasResult','GenericAddressResult','OptionalRawGas','GasSelection','parse_gas','gas_from_raw','gas_is_empty','standard_access_size','gas_decode_access_bit_width'],
 'fixed_bytes':['U8Result','U64Result','OptionalU64'],
 'fadt':['parse_fadt','fadt_minimum_length','preferred_address','facs_address','dsdt_address','select_register','pm_timer_block','reset_register','sleep_control_register','sleep_status_register','power_profile','PowerProfile'],
 'hpet':['parse_hpet','hpet_info','HpetInfo','PageProtection','page_protection'],
 'mcfg':['parse_mcfg','mcfg_entry','McfgView'],
 'madt':['parse_madt','madt_next','madt_entry_byte','MadtStep','MadtEntry','parse_mps_inti_flags','IntiResult','wakeup_command','WakeupCommandResult','supports_8259','Polarity','TriggerMode','MpProtectedModeWakeupCommand'],
 'madt_search':['get_mpwk_mailbox_addr'],
 'fixed_types':['Fadt','FadtResult','HpetTable','HpetTableResult','McfgEntryResult','Madt','MadtResult'],
}


def cases():
    rows=[];model=records();measure=rust_measure(model)
    def add(name,body,checks,data=b''):
        rows.append(dict(name=name,bytes=bytes(data).hex(),body=body,checks=checks))
    def parse(name,module,typ,data,error=0,length=None,extra=None):
        add(name,[f'let result: {typ} = parse_{module}(&input, {len(data) if length is None else length});'],
            [f'result.error == {error}']+(extra or []),data)
    raw=bytes([128,64,3,4])+bytes.fromhex('1032547698badcfe')
    for offset in [0,4084]:
        add('gas_offset_'+str(offset),[f'let result: RawGasResult = parse_gas(&input, {offset+12}, {offset});'],
            ['result.error == 0','result.value.address_space == 128','result.value.bit_width == 64','result.value.bit_offset == 3','result.value.access_size == 4','result.value.address == 18364758544493064720'],bytes(offset)+raw)
    for length,offset in [(11,0),(4097,0),(12,1),(4096,4085),(12,18446744073709551615)]:
        add(f'gas_bounds_{length}_{offset}',[f'let result: RawGasResult = parse_gas(&input, {length}, {offset});'],['result.error == 2'],raw)
    for space in list(range(13))+[126,127,128,191,192,255]:
        add('gas_space_'+str(space),[f'let result: GenericAddressResult = gas_from_raw(RawGenericAddress {{ address_space: {space}, address: 18446744073709551615 }});'],
            [f'result.error == {0 if space<=11 or space>=127 else 11}']+(['result.value.address == 18446744073709551615',f'result.value.address_space == {space}']if space<=11 or space>=127 else []))
    for value in [0,1,2,3,4,5,255]:
        add('gas_access_'+str(value),[f'let result: U8Result = standard_access_size({value});'],[f'result.error == {0 if value<=4 else 11}']+([f'result.value == {value}']if value<=4 else []))
    for width,offset,access,expected,error in [(8,0,255,8,0),(16,0,4,16,0),(32,0,0,32,0),(64,0,1,64,0),(0,0,0,8,0),(7,1,0,8,0),(8,1,0,16,0),(15,1,0,16,0),(16,1,0,32,0),(31,1,0,32,0),(32,1,0,64,0),(255,255,0,64,0),(3,1,1,8,0),(3,1,2,16,0),(3,1,3,32,0),(3,1,4,64,0),(3,1,5,0,11)]:
        add(f'gas_width_{width}_{offset}_{access}',[f'let result: U8Result = gas_decode_access_bit_width(GenericAddress {{ bit_width: {width}, bit_offset: {offset}, access_size: {access} }});'],[f'result.error == {error}',f'result.value == {expected}'])
    add('gas_empty',['let empty: bool = gas_is_empty(RawGenericAddress {});','let occupied: bool = gas_is_empty(RawGenericAddress { access_size: 1 });'],['empty','!occupied'])
    # Each raw field receives a distinctive representable value at its pinned offset.
    def record(name):
        data=bytearray(measure[name+'.size']);checks=[]
        for index,field in enumerate(model[name]['fields']):
            fieldname=field['name'];typ=field['type'];omega=fieldname.lstrip('_');pos=measure[name+'.'+fieldname]
            if fieldname=='header' or not carrier(field,name):continue
            array=re.fullmatch(r'\[u8; (\d+)\]',typ)
            inner=re.match(r'ExtendedField<([^,]+),',typ)
            wire=inner[1]if inner else typ
            wire={'FixedFeatureFlags':'u32','IaPcBootArchFlags':'u16','ArmBootArchFlags':'u16'}.get(wire,wire)
            optional=carrier(field,name).startswith('Optional')
            path=omega+'.value'if optional else omega
            if array:
                for i in range(int(array[1])):
                    value=(index*11+i+1)&255;data[pos+i]=value;checks.append((f'{path}[{i}]',value))
            elif wire=='RawGenericAddress':
                address=0xfedcba9800000000+index;data[pos:pos+12]=bytes([1,32,0,3])+address.to_bytes(8,'little')
                for f,v in [('address_space',1),('bit_width',32),('bit_offset',0),('access_size',3),('address',address)]:checks.append((path+'.'+f,v))
            else:
                bits=int(wire[1:]);value=(1<<(bits-1))+index*3+1;data[pos:pos+bits//8]=value.to_bytes(bits//8,'little');checks.append((path,value))
            if optional:checks.append((omega+'.present',True))
        return data,checks
    for revision,length in [(1,116),(2,132),(3,244),(4,244),(5,268),(6,276),(255,280)]:
        data=sdt(b'FACP',bytes(length-36),revision)
        parse('fadt_revision_'+str(revision),'fadt','FadtResult',data,extra=[f'result.value.header.length == {length}',f'result.value.has_reset == {str(revision>=2).lower()}',f'result.value.x_dsdt_address.present == {str(revision>=3).lower()}'])
        parse('fadt_short_revision_'+str(revision),'fadt','FadtResult',sdt(b'FACP',bytes(length-37),revision),5 if revision!=255 else 0)
    parse('fadt_revision_zero','fadt','FadtResult',sdt(b'FACP',bytes(240),0),4)
    data,fields=record('Fadt');data=sdt(b'FACP',data[36:],6)
    parse('fadt_all_fields','fadt','FadtResult',data,extra=[f'result.value.{key} == {str(value).lower()}' for key,value in fields])
    for name,sig,length,typ in [('fadt',b'FACP',276,'FadtResult'),('hpet',b'HPET',56,'HpetTableResult'),('madt',b'APIC',44,'MadtResult')]:
        data=sdt(sig,bytes(length-36),6 if name=='fadt' else 1)
        bad=bytearray(data);bad[-1]^=1;parse(name+'_bad_checksum',name,typ,bad,6)
        bad=bytearray(data);bad[0]^=1;fix(bad,9);parse(name+'_bad_signature',name,typ,bad,3)
        parse(name+'_truncated',name,typ,data,2,length=length-1)
        parse(name+'_capacity',name,typ,data,1,length=4097)
    for present,extended,legacy,error,expected in [(False,9,7,0,7),(True,9,7,0,9),(True,0,7,0,7),(False,0,0,12,0),(True,0xffffffffffffffff,7,0,0xffffffffffffffff)]:
        add(f'preferred_{len(rows)}',[f'let result: U64Result = preferred_address(OptionalU64 {{ present: {str(present).lower()}, value: {extended} }}, {legacy});'],[f'result.error == {error}',f'result.value == {expected}'])
    for length in [0,4,31,32,255]:
        add('register_width_'+str(length),[f'let result: GasSelection = select_register(OptionalRawGas {{}}, 1234, {length}, true);'],[f'result.error == {0 if length<=31 else 11}']+([f'result.value.bit_width == {length*8}','result.value.address == 1234','result.value.address_space == 1']if length<=31 else []))
    add('register_optional_zero',['let result: GasSelection = select_register(OptionalRawGas {}, 0, 4, false);'],['result.error == 0','!result.present'])
    add('register_required_zero',['let result: GasSelection = select_register(OptionalRawGas {}, 0, 4, true);'],['result.error == 0','result.present','result.value.address == 0'])
    for space,error in [(1,0),(128,0),(12,11)]:
        add('register_extended_'+str(space),[f'let result: GasSelection = select_register(OptionalRawGas {{ present: true, value: RawGenericAddress {{ address_space: {space}, address: 18446744073709551615, bit_width: 64 }} }}, 1234, 255, false);'],[f'result.error == {error}']+(['result.value.address == 18446744073709551615','result.value.bit_width == 64']if not error else []))
    add('timer_bad_length',['let result: GasSelection = pm_timer_block(Fadt { pm_timer_length: 8, x_pm_timer_block: OptionalRawGas { present: true, value: RawGenericAddress { address: 1234 } } });'],['!result.present','result.error == 0'])
    for method in ['reset_register','sleep_control_register','sleep_status_register']:
        add(method+'_absent',[f'let result: GasSelection = {method}(Fadt {{}});'],['!result.present'])
    source=(ROOT/'source/libraries/acpi/fadt.omg').read_text()
    for name,typ,bit in re.findall(r'pub machine (\w+)\(flags: (u\d+)\) -> bool \{ \(flags & (\d+)\) != 0 \}',source):
        IMPORTS['fadt'].append(name)
        add('flag_'+name,[f'let set: bool = {name}({bit});',f'let clear: bool = {name}({(1<<int(typ[1:]))-1-int(bit)});'],['set','!clear'])
    data,fields=record('HpetTable');data=sdt(b'HPET',data[36:])
    parse('hpet_all_fields','hpet','HpetTableResult',data,extra=[f'result.value.{key} == {str(value).lower()}'for key,value in fields])
    parse('hpet_short','hpet','HpetTableResult',sdt(b'HPET',bytes(19)),5)
    parse('hpet_revision','hpet','HpetTableResult',sdt(b'HPET',bytes(20),2),4)
    for encoded in [0,2,31]:
        block=0x12340000|0xa000|encoded<<8|0x81
        add('hpet_count_'+str(encoded),[f'let result: HpetInfo = hpet_info(HpetTable {{ event_timer_block_id: {block}, base_address: RawGenericAddress {{ address_space: 1, address: 18446744073709551615 }}, hpet_number: 255, clock_tick_unit: 65535, page_protection_and_oem: 242 }});'],[f'result.num_comparators == {encoded+1}',f'result.comparator_count_encoded == {encoded}','result.hardware_rev == 129','result.main_counter_is_64bits','result.legacy_irq_capable','result.pci_vendor_id == 4660','result.base_address == 18446744073709551615','result.address_space == 1','result.hpet_number == 255','result.clock_tick_unit == 65535','result.oem_attributes == 15'])
    # Entry tables: framing errors and all fields of all known variants.
    def mcfg(name,payload,tail=False,error=0,index=None,checks=None):
        data=sdt(b'MCFG',bytes(8)+payload)
        call=f'parse_mcfg(&input, {len(data)}, {str(tail).lower()})'if index is None else f'mcfg_entry(&input, {len(data)}, {str(tail).lower()}, {index})'
        add(name,[f'let result: {"McfgView"if index is None else "McfgEntryResult"} = {call};'],[f'result.error == {error}']+(checks or []),data)
    mcfg('mcfg_empty',b'',checks=['result.entry_count == 0'])
    entry=0xfedcba9876543210.to_bytes(8,'little')+bytes.fromhex('341280ffabcdef01')
    mcfg('mcfg_entry',entry,index=0,checks=['result.value.base_address == 18364758544493064720','result.value.pci_segment_group == 4660','result.value.bus_number_start == 128','result.value.bus_number_end == 255','result.value.reserved == 32492971'])
    mcfg('mcfg_index',entry,index=1,error=10);mcfg('mcfg_huge_index',entry,index=18446744073709551615,error=10)
    bad=bytearray(entry);bad[10:12]=bytes([255,128]);mcfg('mcfg_inverted',bad,index=0,error=13)
    for tail in range(1,16):
        mcfg('mcfg_tail_strict_'+str(tail),entry+bytes(tail),error=9)
        mcfg('mcfg_tail_quirk_'+str(tail),entry+bytes(tail),tail=True,checks=['result.entry_count == 1',f'result.ignored_tail_bytes == {tail}'])
    for name,kind in MADT_KINDS.items():
        entry,fields=record(name);entry[0]=kind;entry[1]=len(entry)
        data=sdt(b'APIC',bytes(8)+entry,6)
        body=[f'let step: MadtStep = madt_next(&input, {len(data)}, 44);',f'let fields: bool = fields_{name}(step.entry);']
        add('madt_'+name,body,['step.error == 0','!step.done',f'step.next_offset == {len(data)}','fields'],data)
        rows[-1]['variant']={'record':name,'checks':fields}
        short=entry[:-1];short[1]=len(short);data=sdt(b'APIC',bytes(8)+short,6)
        add('madt_short_'+name,[f'let result: MadtStep = madt_next(&input, {len(data)}, 44);'],['result.error == 14'],data)
    for name,entry,error in [('zero',bytes([128,0]),14),('one',bytes([128,1]),14),('header_short',bytes([128]),2),('overrun',bytes([128,3]),2),('unknown',bytes([128,2]),0)]:
        data=sdt(b'APIC',bytes(8)+entry)
        add('madt_'+name,[f'let result: MadtStep = madt_next(&input, {len(data)}, 44);'],[f'result.error == {error}']+(['result.next_offset == 46','!result.done']if not error else []),data)
    for cursor,error in [(44,0),(43,10),(45,10),(18446744073709551615,10)]:
        add('madt_cursor_'+str(cursor),[f'let result: MadtStep = madt_next(&input, 44, {cursor});'],[f'result.error == {error}']+(['result.done']if not error else []),sdt(b'APIC',bytes(8)))
    for index,error in [(2,0),(3,10),(18446744073709551615,10)]:
        add('madt_unknown_byte_'+str(index),[f'let result: U8Result = madt_entry_byte(&input, 47, 44, {index});'],[f'result.error == {error}']+(['result.value == 255']if not error else []),sdt(b'APIC',bytes(8)+bytes([128,3,255])))
    for flags in range(16):
        error=15 if flags&3==2 else 16 if flags>>2==2 else 0
        add('madt_inti_'+str(flags),[f'let result: IntiResult = parse_mps_inti_flags({flags|0xfff0});'],[f'result.error == {error}'])
    for value in [0,1,2,3,4,65535]:
        add('madt_wakeup_'+str(value),[f'let result: WakeupCommandResult = wakeup_command({value});'],[f'result.error == {0 if value<=3 else 17}'])
    for flags in range(16):
        if flags&3==2 or flags>>2==2:continue
        add('inti_values_'+str(flags),[f'let result: IntiResult = parse_mps_inti_flags({flags});','let p: u8 = polarity_code(result.polarity);','let t: u8 = trigger_code(result.trigger_mode);'],[f'p == {flags&3}',f't == {flags>>2}'])
    for value in range(4):
        add('wakeup_value_'+str(value),[f'let result: WakeupCommandResult = wakeup_command({value});','let code: u16 = wake_code(result.value);'],[f'code == {value}'])
    for value in [0,1,2,3,15,16,17,18,255]:
        add('hpet_page_'+str(value),[f'let value: PageProtection = page_protection({value});','let code: u8 = page_code(value);'],[f'code == {value&15 if value&15<=2 else 3}'])
    for value in list(range(9))+[9,255]:
        add('power_profile_'+str(value),[f'let value: PowerProfile = power_profile(Fadt {{ preferred_pm_profile: {value} }});','let code: u8 = profile_code(value);'],[f'code == {value}'])
    add('fadt_address_helpers',['let table: Fadt = Fadt { firmware_ctrl: 123, dsdt_address: 456, x_firmware_ctrl: OptionalU64 { present: true, value: 18446744073709551615 }, x_dsdt_address: OptionalU64 { present: true, value: 0 } };','let facs: U64Result = facs_address(table);','let dsdt: U64Result = dsdt_address(table);'],['facs.error == 0','facs.value == 18446744073709551615','dsdt.error == 0','dsdt.value == 456'])
    for flags in [0,1,2,0xffffffff]:
        add('madt_8259_'+str(flags),[f'let result: bool = supports_8259(Madt {{ flags: {flags} }});'],[f'result == {str(bool(flags&1)).lower()}'])
    entry,fields=record('GiccEntry');entry[0]=11;entry[1]=80
    data=sdt(b'APIC',bytes(8)+entry[:80],5)
    add('madt_gicc_legacy80',[f'let step: MadtStep = madt_next(&input, {len(data)}, 44);','let fields: bool = fields_GiccEntry(step.entry);'],['step.error == 0','fields'],data)
    rows[-1]['variant']={'record':'GiccEntry','checks':[(key,False if key=='trbe_interrupt.present'else 0 if key=='trbe_interrupt.value'else value)for key,value in fields]}
    wake=bytes([16,16,0,0,0,0,0,0])+0xfedcba9876543210.to_bytes(8,'little')
    for name,payload,error,address in [('empty',b'',18,0),('direct',wake,0,0xfedcba9876543210),('after_unknown',bytes([128,2])+wake,0,0xfedcba9876543210),('malformed',bytes([128,0])+wake,14,0)]:
        data=sdt(b'APIC',bytes(8)+payload)
        add('mailbox_'+name,[f'let result: U64Result = get_mpwk_mailbox_addr(&input, {len(data)});'],[f'result.error == {error}',f'result.value == {address}'],data)
    for i,(method,length_field) in enumerate([('pm1a_event_block','pm1_event_length'),('pm1b_event_block','pm1_event_length'),('pm1a_control_block','pm1_control_length'),('pm1b_control_block','pm1_control_length'),('pm2_control_block','pm2_control_length'),('pm_timer_block','pm_timer_length'),('gpe0_block','gpe0_block_length'),('gpe1_block','gpe1_block_length')]):
        IMPORTS['fadt'].append(method)
        add('wrapper_'+method,[f'let result: GasSelection = {method}(Fadt {{ {method}: {4096+i}, {length_field}: 4 }});'],['result.error == 0','result.present',f'result.value.address == {4096+i}','result.value.bit_width == 32','result.value.address_space == 1'])
    for method,field in [('reset_register','reset_reg'),('sleep_control_register','sleep_control_reg'),('sleep_status_register','sleep_status_reg')]:
        add('wrapper_'+method,[f'let result: GasSelection = {method}(Fadt {{ {field}: OptionalRawGas {{ present: true, value: RawGenericAddress {{ address_space: 128, address: 18446744073709551615, bit_width: 8 }} }} }});'],['result.error == 0','result.present','result.value.address == 18446744073709551615','result.value.address_space == 128','result.value.bit_width == 8'])
    return rows


def render(rows,evaluate=True):
    out=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Original synthetic scenarios generated by fixed_fixtures.py.']
    for module,names in IMPORTS.items():out.extend(f'use acpi::{module}::{name};'for name in dict.fromkeys(names))
    helpers={
      'polarity_code':'machine polarity_code(value: Polarity) -> u8 { transition value { Polarity::SameAsBus -> (0) Polarity::ActiveHigh -> (1) Polarity::ActiveLow -> (3) } }',
      'trigger_code':'machine trigger_code(value: TriggerMode) -> u8 { transition value { TriggerMode::SameAsBus -> (0) TriggerMode::Edge -> (1) TriggerMode::Level -> (3) } }',
      'wake_code':'machine wake_code(value: MpProtectedModeWakeupCommand) -> u16 { transition value { MpProtectedModeWakeupCommand::Noop -> (0) MpProtectedModeWakeupCommand::Wakeup -> (1) MpProtectedModeWakeupCommand::Sleep -> (2) MpProtectedModeWakeupCommand::AcceptPages -> (3) } }',
      'page_code':'machine page_code(value: PageProtection) -> u8 { transition value { PageProtection::None -> (0) PageProtection::Protected4K -> (1) PageProtection::Protected64K -> (2) PageProtection::Other -> (3) } }',
      'profile_code':'machine profile_code(value: PowerProfile) -> u8 { transition value { PowerProfile::Unspecified -> (0) PowerProfile::Desktop -> (1) PowerProfile::Mobile -> (2) PowerProfile::Workstation -> (3) PowerProfile::EnterpriseServer -> (4) PowerProfile::SohoServer -> (5) PowerProfile::AppliancePc -> (6) PowerProfile::PerformanceServer -> (7) PowerProfile::Tablet -> (8) PowerProfile::Reserved { value } -> (value) } }',
    }
    rowtext=str(rows)
    for name,code in helpers.items():
        if name in rowtext:out.append(code)
    for row in rows:
        variant=row.get('variant')
        if variant:
            name=variant['record'];checks=['decoded_'+snake(name)+'.'+key+' == '+str(value).lower()for key,value in variant['checks']]
            out += [f'machine fields_{name}_{row["name"]}(entry: MadtEntry) -> bool {{',f'    transition entry {{ MadtEntry::{name.removesuffix("Entry")} {{ decoded_{snake(name)} }} -> ('+' && '.join(checks)+') _ -> (false) }','}']
        out += [f'machine test_{row["name"]}() -> bool {{','    let mut input: [u8; 4096];']
        out += [f'    input[{i}] = {b};'for i,b in enumerate(bytes.fromhex(row['bytes']))if b]
        out += ['    '+(line.replace('fields_'+variant['record']+'(', 'fields_'+variant['record']+'_'+row['name']+'(')if variant else line) for line in row['body']]+['    '+' &&\n    '.join(row['checks']),'}']
    if evaluate:
        out += ['machine test_result() -> i32 {','    transition '+' &&\n        '.join('test_'+r['name']+'()'for r in rows)+' { true -> (0) _ -> (1) }','}',
                'const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }']
    else:out += ['data Main {}','machine Main::main(&mut self) {}']
    body='\n'.join(line for line in out if not line.startswith('use '))
    out=[line for line in out if not line.startswith('use ') or re.search(r'\b'+line.rsplit('::',1)[1].rstrip(';')+r'\b',body)]
    return '\n'.join(out)+'\n'


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    rows=cases()
    outputs={HERE/'fixed-cases.json':json.dumps({'format':'cathedral-acpi-fixed-cases-v1','provenance':'Original synthetic byte records. No firmware capture or upstream fixture content.','cases':rows},indent=2)+'\n',HERE/'fixed_main.omg':render(rows,False)}
    for path,text in outputs.items():
        if args.check:
            if not path.exists() or path.read_text()!=text:raise SystemExit('Fixture differs: '+str(path))
        else:path.write_text(text)
    print(len(rows),'fixed scenarios '+('verified'if args.check else 'generated'))

if __name__=='__main__':main()
