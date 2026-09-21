#!/usr/bin/env python3
"""Ordinary capture/cursor validation outside the public Rust typed API."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
PREFIX='''// SPDX-License-Identifier: MIT OR Apache-2.0
use x86::memory_encryption;
use x86::memory_encryption::State;
use x86::memory_encryption::Configuration;
use x86::encrypted_cleanup_branch;
use x86::encrypted_cleanup_ranges;
use x86::encrypted_cleanup_recursive;
use x86::cleanup_branch::CleanupPlan;
use x86::cleanup_branch::CleanupOutcome;
use x86::cleanup_ranges::Cursor;
use x86::cleanup_ranges::RangeStep;
use x86::cleanup_ranges::RangeStatus;
use x86::recursive_cleanup::RecursiveCursor;
use x86::recursive_cleanup::RecursiveStep;
'''
PROFILE='''let profile:State=memory_encryption::initial();
_ = memory_encryption::configure(&mut profile,Configuration::EncryptedBit{position:47});
_ = memory_encryption::configure(&mut profile,Configuration::SharedBit{position:48});
let mark:u64=422212465065984;
'''
FOOT='\nconst TEST_RESULT:i32=test_result();\nmachine require_success(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_success(TEST_RESULT);}\n'
cases={
'invalid-page':PROFILE+'''let words:[u64;4]=[65537|mark,131073|mark,196609|mark,0];
let ids:[u64;4]=[262144,65536,131072,196608]; let others:[bool;4]=[false,false,false,false];
let value:CleanupPlan=encrypted_cleanup_branch::plan(profile,&words,&ids,&others,1);
transition value.outcome in CleanupOutcome::InvalidPage && value.retire_count==0 && value.write_mask==0 && value.visited==0 && value.words[0]==words[0] {true -> (0) _ -> (1)}''',
'capture-mismatch':PROFILE+'''let words:[u64;4]=[65537|mark,131073|mark,196609|mark,0];
let ids:[u64;4]=[262144,65536,131072,196608|140737488355328]; let others:[bool;4]=[false,false,false,false];
let value:CleanupPlan=encrypted_cleanup_branch::plan(profile,&words,&ids,&others,0);
transition value.outcome {
 CleanupOutcome::CaptureMismatch{expected,supplied} -> tested(&value,expected,supplied)
 _ -> (1)
}
state tested(value:&CleanupPlan,expected:u64,supplied:u64) {
 transition expected==196608 && supplied==140737488551936 && value.retire_count==0 && value.write_mask==0 && value.visited==3 && value.words[0]==422212465131521 && value.words[1]==422212465197057 && value.words[2]==422212465262593 {true -> (0) _ -> (1)}
}''',
'retained-prefix':PROFILE+'''let first_words:[u64;4]=[65537|mark,131073|mark,196609|mark,0];
let first_ids:[u64;4]=[262144,65536,131072,196608]; let first_others:[bool;4]=[true,false,true,false];
let cursor:Cursor=encrypted_cleanup_ranges::begin(0,2097152,9);
let first:RangeStep=encrypted_cleanup_ranges::step(profile,cursor,&first_words,&first_ids,&first_others);
let words:[u64;4]=[65537|mark,131073|mark,327681|mark,0];
let ids:[u64;4]=[262144,65536,131072,458752];let others:[bool;4]=[true,false,false,false];
let second:RangeStep=encrypted_cleanup_ranges::step(profile,first.cursor,&words,&ids,&others);
transition second.cursor.status {
 RangeStatus::CaptureMismatch{expected,supplied} -> checked(&first,&second,expected,supplied)
 _ -> (1)
}
state checked(first:&RangeStep,second:&RangeStep,expected:u64,supplied:u64) {
 let next:u64=second.cursor.next_page; let budget:u64=second.cursor.budget;
 transition expected==327680 && supplied==458752 && first.branch.retire_count==1 && first.branch.frames_to_retire[0]==196608 && first.branch.write_mask==4 && first.branch.words[2]==0 && second.branch.retire_count==0 && second.branch.write_mask==0 && next==2097152 && budget==8 && second.branch.words[2]==422212465393665 {true -> (0) _ -> (1)}
}''',
'resume-and-validation':PROFILE+'''let words:[u64;4]=[65537|mark,131073|mark,196609|mark,0];
let ids:[u64;4]=[262144,65536,131072,196608];let others:[bool;4]=[false,false,false,false];
let cursor:Cursor=encrypted_cleanup_ranges::begin(0,0,0);
let resumed:Cursor=encrypted_cleanup_ranges::resume(cursor,18446744073709551615);
let value:RangeStep=encrypted_cleanup_ranges::step(profile,resumed,&words,&ids,&others);
let forged:Cursor=Cursor{next_page:0,last_page:0,budget:0,status:RangeStatus::Active};
let stopped:RangeStep=encrypted_cleanup_ranges::step(profile,forged,&words,&ids,&others);
let invalid:Cursor=encrypted_cleanup_ranges::begin(1,0,9);
let preserved:Cursor=encrypted_cleanup_ranges::resume(invalid,9);
let budget:u64=value.cursor.budget;
transition value.cursor.status in RangeStatus::Done && budget==18446744073709551614 && value.branch.retire_count==3 && !stopped.has_plan && stopped.cursor.status in RangeStatus::Exhausted && preserved.status in RangeStatus::InvalidRange {true -> (0) _ -> (1)}''',
'recursive-validation':PROFILE+'''let words:[u64;4]=[65537|mark,131073|mark,196609|mark,0];
let ids:[u64;4]=[262144,65536,131072,196608];let others:[bool;4]=[true,false,false,false];
let cursor:RecursiveCursor=encrypted_cleanup_recursive::begin(0,0,0,511);
let resumed:RecursiveCursor=encrypted_cleanup_recursive::resume(cursor,18446744073709551615);
let value:RecursiveStep=encrypted_cleanup_recursive::step(profile,resumed,&words,&ids,&others);
let bad:RecursiveCursor=RecursiveCursor{range:Cursor{next_page:0,last_page:0,budget:9,status:RangeStatus::Active},recursive_index:512};
let invalid:RecursiveStep=encrypted_cleanup_recursive::step(profile,bad,&words,&ids,&others);
let kept:RecursiveCursor=encrypted_cleanup_recursive::resume(bad,3);
let budget:u64=value.cursor.range.budget;
transition value.cursor.range.status in RangeStatus::Done && budget==18446744073709551614 && value.branch.retire_count==3 && value.cursor.recursive_index==511 && !invalid.has_plan && invalid.cursor.range.status in RangeStatus::InvalidRange && kept.range.status in RangeStatus::InvalidRange {true -> (0) _ -> (1)}''',
'recursive-capture-mismatch':PROFILE+'''let words:[u64;4]=[65537|mark,131073|mark,196609|mark,0];
let ids:[u64;4]=[262144,65536|140737488355328,131072,196608];let others:[bool;4]=[true,false,false,false];
let cursor:RecursiveCursor=encrypted_cleanup_recursive::begin(0,4096,18446744073709551615,511);
let value:RecursiveStep=encrypted_cleanup_recursive::step(profile,cursor,&words,&ids,&others);
transition value.cursor.range.status {
 RangeStatus::CaptureMismatch{expected,supplied} -> checked(&value,expected,supplied)
 _ -> (1)
}
state checked(value:&RecursiveStep,expected:u64,supplied:u64) {
 let budget:u64=value.cursor.range.budget;
 transition expected==65536 && supplied==140737488420864 && budget==18446744073709551615 && value.branch.write_mask==0 && value.branch.retire_count==0 && value.branch.visited==1 && value.branch.words[0]==422212465131521 {true -> (0) _ -> (1)}
}''',
}
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.write_text(text)
extras=[]
for name,body in cases.items():
 path='cases/extra-'+name+'.omg';write(HERE/path,PREFIX+'machine test_result()->i32 {\n'+body+'\n}\n'+FOOT);extras.append(dict(path=path))
write(HERE/'extras.json',json.dumps(dict(extras=extras,controls=[dict(path='cases/extra-capture-mismatch.omg',family='capture-identity',old='expected==196608',new='expected==196609'),dict(path='cases/extra-retained-prefix.omg',family='retained-earlier-branch',old='first.branch.retire_count==1',new='first.branch.retire_count==0'),dict(path='cases/extra-resume-and-validation.omg',family='maximum-budget-resume',old='budget==18446744073709551614',new='budget==18446744073709551613')]),indent=2)+'\n')
print(len(cases),'ordinary capture/cursor policy bodies')
