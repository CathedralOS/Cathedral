#!/usr/bin/env python3
"""Complete resource.rs anchor audit for the bounded pinned descriptor families."""
import json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
args=[sys.executable,str(ROOT/'tools/ports/inventory.py'),'snapshot','--checkout',str(ROOT/'reference_code/rust-osdev/acpi'),'--revision','257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5','--url','https://github.com/rust-osdev/acpi','src/aml/resource.rs']
data=json.loads(subprocess.check_output(args,text=True));data['slice']='bounded pin-supported descriptor families including GPIO and SerialBus/I2C; other defined descriptors remain framing-only Unsupported'
model={'Resource':'Resource','InterruptTrigger':'Trigger','InterruptPolarity':'Polarity','AddressSpaceResourceType':'AddressKind','AddressSpaceDecodeType':'Address','AddressSpaceDescriptor':'Address','MemoryRangeDescriptor':'FixedMemory','IrqDescriptor':'Irq','DMASupportedSpeed':'DmaSpeed','DMATransferTypePreference':'DmaWidth','DMADescriptor':'Dma','IOPortDescriptor':'Io','Irqs':'Interrupts'}
functions={'resource_descriptor_list':('resource_template.omg','pub machine validate_template('),'resource_descriptor':('resource_parse.omg','pub machine parse_one('),'fixed_memory_descriptor':('resource_parse.omg','machine decode_memory('),'address_space_descriptor':('resource_parse.omg','machine decode_address('),'irq_format_descriptor':('resource_parse.omg','machine decode_irq('),'dma_format_descriptor':('resource_parse.omg','machine decode_dma('),'io_port_descriptor':('resource_parse.omg','machine decode_io('),'extended_interrupt_descriptor':('resource_parse.omg','machine decode_extended('),'irqs_from_bit_mask':('resource_irq.omg','pub machine irq_at('),'UNSUPPORTED_LARGE_RESOURCE_DESCRIPTORS':('resource_parse.omg','pub machine parse_one('),'UNSUPPORTED_SMALL_RESOURCE_DESCRIPTORS':('resource_parse.omg','pub machine parse_one('),'unsupported_resource_descriptor_name':('resource_parse.omg','pub machine parse_one('),'skip_unsupported_resource':('resource_parse.omg','pub machine parse_one(')}
model.update({'ShareType':'I2c','ResourceUsage':'I2c','SlaveMode':'I2c','AddressingMode':'AddressingMode','SerialBusDescriptor':'I2c','SerialBus':'I2c','GPIOInterruptPolarity':'GpioPolarity','WakeCapability':'GpioConnection','IORestriction':'IoRestriction','PinConfiguration':'PinConfiguration','GPIOConnectionDescriptor':'Gpio','GPIOConnection':'GpioConnection'})
functions.update({'i2c_bus_descriptor':('resource_connections.omg','machine decode_serial_connection('),'serial_bus_descriptor':('resource_connections.omg','machine decode_serial_connection('),'gpio_connection_descriptor':('resource_connections.omg','machine decode_gpio_connection(')})
for file in data['files'].values():
 file['disposition']='translated';file['reason']='All pin-supported families mapped to bounded decoders; declared profile/spec corrections documented. Other defined families remain explicit Unsupported, as the pin omits their semantics.'
 file['targets']=[{'path':'source/libraries/acpi/resources/resource_parse.omg','anchor':'pub machine parse_one('},{'path':'source/libraries/acpi/resources/resource_connections.omg','anchor':'pub machine parse_connection('}]
 for key,row in file['symbols'].items():
  line,name=key.split(':',1);line=int(line);target=None
  row['disposition']='pending';row['reason']='Outside first descriptor slice. Preserve full envelope as Unsupported; semantic decoder and policy remain pending.'
  if name in model:target=('resource_model.omg','pub data '+model[name]+' ')
  elif name in functions:target=functions[name]
  elif 208<=line<=216:target=('resource_model.omg','pub data Address ')
  elif 349<=line<=354:target=('resource_model.omg','pub data Irq ')
  elif 463<=line<=466:target=('resource_model.omg','pub data Dma ')
  elif 519<=line<=522:target=('resource_model.omg','pub data Io ')
  elif 637<=line<=643:target=('resource_model.omg','pub data I2c ')
  elif 737<=line<=745:target=('resource_model.omg','pub data Gpio ')
  if target:
   row['disposition']='translated';row['reason']='Modified bounded data/decoder mapping; checked envelopes, semantic cases, retained source spans, strict template profile. See PORT.md for exact spec corrections and raw-fact boundary; full Rust allocation/object API is not reproduced.';row['targets']=[{'path':'source/libraries/acpi/resources/'+target[0],'anchor':target[1]}]
  if name in ['Resource','resource_descriptor_list','resource_descriptor']:
   row['reason']='All resource families implemented by this pin are represented: IRQ/DMA/IO/FixedMemory32/address/ExtendedIRQ/GPIO/SerialBus-I2C. Bounded spans replace allocations; strict envelope/source/EndTag and spec corrections documented. Other defined descriptors remain explicit Unsupported and are not claimed semantically decoded.'
   row['implemented_components']=['bounded descriptor envelope','strict template termination/checksum profile','IRQ','DMA','IO','FixedMemory32','Word/DWord/QWord address','ExtendedIRQ','GPIO interrupt and IO','SerialBus/I2C']
  if line==7:
   row['disposition']='omitted';row['reason']='smallvec allocator re-export is replaced by bounded mask/table-span views and checked index access; no dynamic container dependency.'
  if line==510:
   row['disposition']='omitted';row['reason']='Lexical scanner false positive inside unimplemented!("Reserved DMA transfer type preference") string, not a Rust declaration. Actual reserved code becomes BadEncoding in decode_dma.'
  if line>=826:
   row['disposition']='omitted';row['reason']='Upstream test/module body not copied. Original synthetic positive/malformed fixtures call the actual pinned public parser and actual Omega bodies; no firmware dump transcription.'
path=ROOT/'source/libraries/acpi/resources/inventory.json';text=json.dumps(data,indent=2,sort_keys=True)+'\n'
if '--check'in sys.argv:assert path.read_text()==text,'stale resources inventory'
else:path.write_text(text)
