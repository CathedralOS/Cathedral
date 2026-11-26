#!/usr/bin/env python3
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
prefix='''// SPDX-License-Identifier: MIT OR Apache-2.0
use {values}/recursive_cleanup.omg;
use {values}/cleanup_ranges.omg;
'''
arrays='let words:[u64;4]=[0,0,0,0];let ids:[u64;4]=[16384,0,0,0];let others:[bool;4]=[true,false,false,false];\n'
# Expected-behavior changes live inside each body; require_ok remains unchanged.
cases={
 'invalid-index':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(0,0,2,512);let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&words,&ids,&others);', 'result.cursor.range.status in |cleanup_ranges|::RangeStatus::InvalidRange && !result.has_plan','|cleanup_ranges|::RangeStatus::InvalidRange','|cleanup_ranges|::RangeStatus::Done'),
 'invalid-address':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(140737488355328,140737488355328,2,0);let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&words,&ids,&others);', 'result.cursor.range.status in |cleanup_ranges|::RangeStatus::InvalidRange && !result.has_plan','|cleanup_ranges|::RangeStatus::InvalidRange','|cleanup_ranges|::RangeStatus::Done'),
 'forged-active':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::RecursiveCursor{range:|cleanup_ranges|::Cursor{next_page:1,last_page:4096,budget:2,status:|cleanup_ranges|::RangeStatus::Active},recursive_index:0};let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&words,&ids,&others);', 'result.cursor.range.status in |cleanup_ranges|::RangeStatus::InvalidRange && !result.has_plan','|cleanup_ranges|::RangeStatus::InvalidRange','|cleanup_ranges|::RangeStatus::Done'),
 'reversed':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(4096,0,2,0);let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&words,&ids,&others);','result.cursor.range.status in |cleanup_ranges|::RangeStatus::Done && !result.has_plan','|cleanup_ranges|::RangeStatus::Done','|cleanup_ranges|::RangeStatus::Active'),
 'zero-resume':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(0,0,0,0);let paused:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&words,&ids,&others);let resumed:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::resume(paused.cursor,1);let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(resumed,&words,&ids,&others);','paused.cursor.range.status in |cleanup_ranges|::RangeStatus::Exhausted && !paused.has_plan && result.cursor.range.status in |cleanup_ranges|::RangeStatus::Done && !result.has_plan && result.cursor.recursive_index==0','result.cursor.recursive_index==0','result.cursor.recursive_index==1'),
 'skip-resume-high':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(140187732541440,18446603336221196288,1,255);let first:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&words,&ids,&others);let resumed:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::resume(first.cursor,2);let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(resumed,&words,&ids,&others);let next:u64=first.cursor.range.next_page;','first.cursor.range.status in |cleanup_ranges|::RangeStatus::Exhausted && !first.has_plan && next==18446603336221196288 && result.cursor.range.status in |cleanup_ranges|::RangeStatus::Done && result.has_plan && result.cursor.recursive_index==255 && result.cursor.range.budget==1','result.cursor.recursive_index==255','result.cursor.recursive_index==254'),
 'inactive-resume':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(0,0,2,0);let result:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::resume(c,99);','result.range.status in |cleanup_ranges|::RangeStatus::Active && result.range.budget==2 && result.recursive_index==0','result.range.budget==2','result.range.budget==99'),
 'capture-mismatch':('let bad_words:[u64;4]=[4097,0,0,0];let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(0,0,2,1);let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&bad_words,&ids,&others);','result.cursor.range.status in |cleanup_ranges|::RangeStatus::CaptureMismatch && result.has_plan && result.cursor.range.budget==2 && result.branch.retire_count==0','result.cursor.range.budget==2','result.cursor.range.budget==1'),
 'high-bit-budget':('let c:|recursive_cleanup|::RecursiveCursor=|recursive_cleanup|::begin(0,549755813888,9223372036854775808,0);let result:|recursive_cleanup|::RecursiveStep=|recursive_cleanup|::step(c,&words,&ids,&others);let budget:u64=result.cursor.range.budget;','result.cursor.range.status in |cleanup_ranges|::RangeStatus::Active && !result.has_plan && budget==9223372036854775807','budget==9223372036854775807','budget==9223372036854775806'),
}
metadata={}
for name,(setup,condition,old,new)in cases.items():
 source=prefix+'machine test()->i32 {\n'+arrays+setup+'\ntransition '+condition+' {true -> (0) _ -> (1)}\n}\nconst RESULT:i32=test();machine require_ok(value:i32) requires value==0;{}data Main{}machine Main::main(&mut self){require_ok(RESULT);}\n'
 path=HERE/'cases'/f'extra-{name}.omg'
 if '--check'in sys.argv:assert path.read_text()==source,path
 else:path.write_text(source)
 metadata['extra-'+name]=[old,new]
path=HERE/'extras.json';text=json.dumps(metadata,indent=2)+'\n'
if '--check'in sys.argv:assert path.read_text()==text
else:path.write_text(text)
