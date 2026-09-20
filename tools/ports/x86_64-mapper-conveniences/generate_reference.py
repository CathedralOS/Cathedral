#!/usr/bin/env python3
"""Reuse reviewed owned-table reference setup; exercise actual default Mapper methods."""
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
s=(ROOT/'tools/ports/x86_64-mapping-routes/probe.rs.in').read_text();s=s[:s.index('fn main(){')]
old='"map"=>match unsafe{mapper.map_to_with_table_flags(page,PhysFrame::<S>::from_start_address(PhysAddr::new(frame)).unwrap(),f,pf,&mut supply)}'
new='"map"|"identity"=>match unsafe{if op=="identity"{mapper.identity_map(PhysFrame::<S>::from_start_address(PhysAddr::new(frame)).unwrap(),f,&mut supply)}else{mapper.map_to(page,PhysFrame::<S>::from_start_address(PhysAddr::new(frame)).unwrap(),f,&mut supply)}}'
assert s.count(old)==1;s=s.replace(old,new).replace('let pf=PageTableFlags::from_bits_retain(parent);','let _pf=PageTableFlags::from_bits_retain(parent);')
s+='''pub fn verify(){
 assert_eq!(observe::<Size4KiB>("map",0,16384,[0;4],[4096,8192,12288],36864,7,0),Observation{kind:0,payload:36864,words:[4103,8199,12295,36871],calls:3,zero:14});
 assert_eq!(observe::<Size2MiB>("identity",0,4194304,[0;4],[4096,8192,0],4194304,7,0),Observation{kind:0,payload:4194304,words:[4103,8199,4194439,0],calls:2,zero:6});
 assert_eq!(observe::<Size1GiB>("map",0,2147483648,[0;4],[4096,0,0],3221225472,7,0),Observation{kind:0,payload:3221225472,words:[4103,3221225607,0,0],calls:1,zero:2});
 assert_eq!(observe::<Size4KiB>("translate",0,16384,[4097,8193,12289,36871],[0;3],0,0,0),Observation{kind:0,payload:36864,words:[4097,8193,12289,36871],calls:0,zero:0});
}
'''
p=HERE/'src/route_reference.rs'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=s:raise SystemExit('owned-table reference mirror drift')
else:p.write_text(s)
