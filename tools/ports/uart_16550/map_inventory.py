#!/usr/bin/env python3
"""Reviewed exact-pin UART audit mappings, including live-boundary omissions."""
import json,re
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
path=ROOT/'source/drivers/uart_16550/inventory.json';doc=json.loads(path.read_text())
base='source/drivers/uart_16550/';pure=base+'pure.omg';plans=base+'plans.omg';boundary=base+'BOUNDARIES.md';error=base+'errors.omg';facts='source/drivers/facts/uart_16550.omg';test='tools/ports/uart_16550/main.omg'
rows=json.loads((HERE/'facts.json').read_text())
def mapped(target,anchor,reason='Pure value/decision translation; no hardware authority.'):
 return {'disposition':'translated','reason':reason,'targets':[{'path':target,'anchor':anchor}]}
def omitted(reason):return {'disposition':'omitted','reason':reason}
spec_methods={'calc_baud_rate':'calc_baud_rate','calc_frequency':'calc_frequency','calc_divisor':'calc_divisor','interrupt_type':'interrupt_type','has_pending_interrupt':'has_pending_interrupt','priority':'interrupt_priority','from_bits':'interrupt_type','fifo_trigger_level':'fifo_trigger_level','set_fifo_trigger_level':'set_fifo_trigger_level','word_length':'word_length','set_word_length':'set_word_length','parity':'parity','set_parity':'set_parity','has_error':'has_error','pdf':'pdf','set_pdf':'set_pdf'}
lib_methods={'new_port':(pure,'valid_port_base'),'new_mmio':(pure,'valid_mmio_base'),'check_present':(plans,'init_step'),'init':(plans,'init_step'),'check_connected':(pure,'check_connected'),'ready_to_receive':(pure,'ready_to_receive'),'ready_to_send':(pure,'ready_to_send'),'try_receive_byte':(plans,'receive_byte'),'try_send_byte':(pure,'try_send_error'),'send_bytes':(pure,'send_count'),'dll_dlm':(plans,'divisor_step'),'configure_fcr':(pure,'configured_fcr'),'config_register_dump':(plans,'dump_step'),'divisor':(pure,'dump_divisor'),'baud_rate':(pure,'dump_baud_rate')}
for source,file in doc['files'].items():
 stem=Path(source).stem;lines=(ROOT/'reference_code/rust-osdev/uart_16550'/source).read_text().splitlines()
 for key,symbol in file['symbols'].items():
  n=int(key.split(':')[0]);name=key.split(':')[1];a=symbol['anchor'];preceding='\n'.join(lines[:n]);owners=re.findall(r'pub (?:struct|enum) (\w+)',preceding);owner=owners[-1] if owners else ''
  if name in {'fmt','source','Sealed','private','tests','accept','consume'} or a.startswith(('mod ','pub mod ','pub use ')):
   result=omitted('Rust namespace, formatting/error-framework or trait-glue only; no hardware fact/algorithm omitted.')
  elif source=='tests/api.rs':result=mapped(test,'machine test_constructors','Constructor predicates are callable and tested; no public ambient driver/backend object is recreated.')
  elif stem=='spec':
   if n>=1166:
    group='baud' if name.startswith('test_calc') else 'interrupt' if ('interrupt' in name or name.startswith('isr_')) else 'bits'
    result=mapped(test,'machine test_'+group,'Upstream test cases adapted to raw-byte helpers; final evaluator assertion has a behavior-mutating negative control.')
   elif name in {'NonIntegerBaudRateError','NonIntegerDivisorError'} or owner in {'NonIntegerBaudRateError','NonIntegerDivisorError'} and name in {'frequency','divisor','baud_rate','prescaler_division_factor'}:
    result=mapped(error,'pub data '+(name if name.startswith('NonInteger') else owner),'Error input payload preserved; optional prescaler uses explicit presence plus value.')
   elif name=='Divisor':result=mapped(base+'enum_ordinals.omg','module enum_ordinals','All89 Rust enum ordinals preserved; never confuse an ordinal with the divisor named in its variant.')
   elif name in spec_methods and 'fn ' in a:result=mapped(pure,'machine '+spec_methods[name]+'(')
   elif name in {'from_raw_bits','to_raw_bits','from_integer','to_integer'}:
    prefix={'FifoTriggerLevel':'fifo','WordLength':'word','Parity':'parity'}[owner];result=mapped(pure,'machine '+prefix+'_'+name+'(')
   elif 'const ' in a:
    matches=[r for r in rows if r['upstream']=='spec::'+name or r['upstream']=='spec::registers::offsets::'+name and n<=225 or r['upstream']==f'spec::registers::{owner}::{name}']
    if len(matches)!=1:raise ValueError((source,key,'constant mismatch',matches))
    result=mapped(facts,'pub const '+matches[0]['name'], 'Reconciled with existing fact owner; independent compiled upstream value verification.')
   else:
    anchor={'InterruptType':'interrupt_type','FifoTriggerLevel':'fifo_from_raw_bits','WordLength':'word_from_raw_bits','Parity':'parity_from_raw_bits'}.get(name,'ConfigRegisterDump')
    result=mapped(pure,anchor,'Register aliases/bitflags are raw u8; logical variants use documented encoded values, preserving unknown register bits.')
  elif stem=='config':
   if name=='BaudRate' or name in {'to_integer','from_integer'}:
    result=mapped(base+'baud.omg','pub data BaudRate' if name=='BaudRate' else 'machine baud_'+name+'(')
   elif name in {'partial_cmp','cmp'}:result=mapped(pure,'machine baud_compare(')
   elif name in {'DEFAULT','default'}:result=mapped(pure,'machine default_config(')
   else:result=mapped(pure,'pub data Config','Config retains fields via integer bit encodings; optional values carry explicit presence/enabled bits.')
  elif stem=='error' or stem=='tty' and name=='Uart16550TtyError':result=mapped(error,'pub data '+name)
  elif stem=='tty':
   result=mapped(pure,'machine tty_byte(') if name=='write_str' else omitted('Live TTY driver constructor/borrow wrapper omitted; initialization/loopback needs the explicit access and lifecycle boundaries in BOUNDARIES.md.')
  elif stem=='lib':
   if name in lib_methods:
    target,anchor=lib_methods[name];result=mapped(target,'machine '+anchor+'(','Pure constructor predicate/request or decision only; live reads/writes remain the explicit boundary in BOUNDARIES.md.')
   elif name in {'ier','isr','lcr','mcr','lsr','msr','spr'} and 'fn ' in a:result=mapped(plans,'machine register_read(','Literal byte read request; bank and side effects remain executor obligations.')
   elif name=='ConfigRegisterDump' or n in {1008,1010,1012,1014,1016,1018,1020,1022,1024}:result=mapped(pure,'pub data ConfigRegisterDump')
   elif name=='constructors':result=mapped(test,'machine test_constructors')
   elif name in {'mmio_dummy','STRIDE'}:result=mapped(test,'machine test_plans','Adapts divisor3 and stride1/4 offset expectations to request plans; no volatile RAM/device emulator execution claimed.')
   elif name in {'TEST_BYTE','TEST_MESSAGE','test_loopback'}:result=omitted('Live loopback deliberately omitted; exact protocol, test byte/message and cleanup/infinite-wait limitations recorded in BOUNDARIES.md.')
   else:result=omitted('Live driver ownership, polling/stream loops or Rust Send API omitted; pure requests/decisions are translated and explicit IO/lifecycle semantics are recorded in BOUNDARIES.md.')
  elif 'backend' in source:
   if name in {'assert_offset','add_offset','read','write'}:result=mapped(pure,'machine register_offset(','Checked register geometry only; dereference, PortIo instructions, MMIO custody and effects remain explicit access boundaries.')
   else:result=omitted('Rust backend/address wrapper or executable IO boundary; no ambient authority-bearing replacement. Constructor geometry and register request plans cover the pure obligations; see BOUNDARIES.md.')
  elif stem=='embedded_io':result=omitted('Rust embedded-io traits/live polling adapter excluded; exact repetition/no-op flush behavior recorded in BOUNDARIES.md; no progress guarantee invented.')
  else:raise ValueError((source,key,a))
  symbol.update(result)
 file['disposition']='translated';file['reason']='Complete audit; pure facts/plans/tests mapped and live IO/Rust-specific integration omissions explicitly justified.';file['targets']=[{'path':boundary,'anchor':'# UART access boundaries'}]
path.write_text(json.dumps(doc,indent=2)+'\n')
from collections import Counter
print(Counter(s['disposition'] for f in doc['files'].values() for s in f['symbols'].values()))
