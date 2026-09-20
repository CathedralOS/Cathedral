#!/usr/bin/env python3
"""Review dispositions for pinned numeric page/frame files; exact-line overrides are intentional."""
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=ROOT/'source/libraries/x86_64/pages-inventory.json';doc=json.loads(p.read_text())
source='source/libraries/x86_64/pages.omg';fixture='tools/ports/x86_64-pages/main.omg';extras='tools/ports/x86_64-pages/extras.omg'
for path,file in doc['files'].items():
 frame=path.endswith('/frame.rs')
 for key,row in file['symbols'].items():
  number,name=key.split(':');number=int(number);disposition='translated';target=source;reason='Pure numeric geometry/operation retained with explicit checked inputs/results; page/frame ownership and native representation are not recreated.'
  if name in {'fmt','Output','Item','any','tests','proofs','test_is_hash','test_page_is_hash','PageSize','NotGiantPageSize','DEBUG_STR'} or 'unchecked' in name:
   disposition='omitted';reason='Rust formatting/trait/newtype/unchecked-construction machinery deliberately omitted. Explicit size validation and checked numeric operations replace it.';anchor=None
  elif (frame and number>=534) or (not frame and number>=952):
   disposition='omitted';reason='Kani proof infrastructure is retained as upstream audit metadata, not claimed as an Omega universal proof. Finite boundaries and iterator behavior have executable fixtures.';anchor=None
  elif (frame and number>=520) or (not frame and number>=691):
   target=extras if name in {'test_frame_range_len','test_page_ranges','test_page_range_inclusive_overflow','page_step_overflowing'} else fixture
   anchor='pub machine result(' if target==extras else 'machine test_result('
   reason='Adapted expectations: all stepping tables and overflow outcomes; representative positions of the 1000-element ranges, explicit checked gap-range deviations. Original host tests additionally execute unchanged.'
  else:
   mapping={'from_start_address':'from_start','containing_address':'containing','from_pfn':'frame_from_pfn','try_from_pfn':'frame_from_pfn','pfn':'frame_pfn','range':'range_count','range_inclusive':'range_count','is_empty':'range_count','len':'range_count','size_hint':'range_count','next':'range_at','nth':'range_at','next_back':'range_at','nth_back':'range_at','add':'arithmetic','add_assign':'arithmetic','sub_assign':'arithmetic','steps_between_u64':'steps_between','steps_between_impl':'steps_between','steps_between':'steps_between','forward_checked_impl':'step','forward_checked':'step','backward_checked':'step','forward_overflowing':'step_overflowing','backward_overflowing':'step_overflowing','from_page_table_indices':'from_indices','from_page_table_indices_2mib':'from_indices','from_page_table_indices_1gib':'from_indices','as_4kib_page_range':'range_count'}
   if name in mapping:anchor='machine '+mapping[name]+'('
   elif name=='sub':anchor='machine '+('difference' if number==(222 if frame else 300) else 'arithmetic')+'('
   elif name in {'Size4KiB','Size2MiB','Size1GiB'}:anchor='pub const '+{'Size4KiB':'SIZE_4K','Size2MiB':'SIZE_2M','Size1GiB':'SIZE_1G'}[name]+':'
   elif name in {'SIZE','size'}:anchor='machine '+('range_bytes' if (frame and number in {256,397}) or (not frame and number in {370,526}) else 'page_size_valid')+'('
   elif name in {'p1_index','p2_index','p3_index','p4_index','page_table_index'}:target='source/libraries/x86_64/addresses.omg';anchor='machine page_index('
   elif name in {'Page','PhysFrame','start_address','AddressNotAligned','PfnNotValid'}:anchor='machine from_start('
   elif name in {'PageRange','PageRangeInclusive','PhysFrameRange','PhysFrameRangeInclusive','start','end'}:anchor='machine range_count('
   else:raise ValueError((path,key))
  row.update(disposition=disposition,reason=reason)
  if anchor:row['targets']=[dict(path=target,anchor=anchor)]
 file.update(disposition='translated',reason='Reviewed numeric page/frame profile; iterator object/ABI/proof infrastructure adaptations and omissions are explicit per symbol and in pages.PORT.md.',targets=[dict(path=source,anchor='module pages;')])
p.write_text(json.dumps(doc,indent=2)+'\n')
