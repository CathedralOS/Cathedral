#!/usr/bin/env python3
"""Execute pinned upstream methods against recording ports; never issue x86 I/O."""
from pathlib import Path
import subprocess
import sys
import tempfile
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'tools/ports'))
import inventory
manifest = inventory.read_json(ROOT/'source/libraries/pic8259/inventory.json')
inventory.check(manifest, ROOT/'reference_code/rust-osdev/pic8259')
source = (ROOT/'reference_code/rust-osdev/pic8259/src/lib.rs').read_text()
# Only crate framing changes; every controller implementation byte is retained.
source = source.replace('#![no_std]', '').replace('//!','//')
mock = r'''
mod x86_64 { pub mod instructions { pub mod port {
    use std::sync::Mutex;
    pub static EVENTS: Mutex<Vec<(u8,u16,u8)>> = Mutex::new(Vec::new());
    pub struct Port<T> { address: u16, _kind: std::marker::PhantomData<T> }
    impl<T> Port<T> { pub const fn new(address:u16)->Self { Self{address,_kind:std::marker::PhantomData} } }
    impl Port<u8> {
        pub unsafe fn read(&mut self)->u8 {
            let value = match self.address {0x21=>0x5a,0xa1=>0xa5,_=>panic!("unexpected read")};
            EVENTS.lock().unwrap().push((0,self.address,value)); value
        }
        pub unsafe fn write(&mut self,value:u8) { EVENTS.lock().unwrap().push((1,self.address,value)); }
    }
} } }
fn take_events()->Vec<(u8,u16,u8)> {
    std::mem::take(&mut *x86_64::instructions::port::EVENTS.lock().unwrap())
}
fn main() {
    unsafe {
        let mut pics=ChainedPics::new_contiguous(32);
        pics.initialize();
        let mut expected=vec![(0,0x21,0x5a),(0,0xa1,0xa5)];
        for (port,value) in [(0x20,0x11),(0xa0,0x11),(0x21,32),(0xa1,40),(0x21,4),(0xa1,2),(0x21,1),(0xa1,1)] {
            expected.push((1,port,value)); expected.push((1,0x80,0));
        }
        expected.extend([(1,0x21,0x5a),(1,0xa1,0xa5)]);
        assert_eq!(take_events(),expected);
        assert_eq!(pics.read_masks(),[0x5a,0xa5]);
        assert_eq!(take_events(),[(0,0x21,0x5a),(0,0xa1,0xa5)]);
        pics.write_masks(0,255);
        assert_eq!(take_events(),[(1,0x21,0),(1,0xa1,255)]);
        pics.disable();
        assert_eq!(take_events(),[(1,0x21,255),(1,0xa1,255)]);
        // Exhaust every vector for separated/reversed/contiguous ranges. Avoid
        // the documented upstream offset248 overflow instead of hiding it.
        for (master,slave) in [(0,8),(32,40),(64,32),(232,240)] {
            let mut pair=ChainedPics::new(master,slave);
            for vector in 0..=255u8 {
                let m=(master..master+8).contains(&vector);
                let s=(slave..slave+8).contains(&vector);
                assert_eq!(pair.handles_interrupt(vector),m||s);
                pair.notify_end_of_interrupt(vector);
                let expected=if s {vec![(1,0xa0,0x20),(1,0x20,0x20)]}
                    else if m {vec![(1,0x20,0x20)]} else {vec![]};
                assert_eq!(take_events(),expected);
            }
        }
    }
    println!("PASS pinned upstream init/read/write/disable and 1024 membership+EOI routes with recording ports; no hardware I/O");
}
'''
with tempfile.TemporaryDirectory(prefix='pic8259-upstream-mock-') as directory:
    path = Path(directory)
    (path/'probe.rs').write_text(source+'\n'+mock)
    subprocess.run(['rustc','--edition=2021',str(path/'probe.rs'),'-o',str(path/'probe')],check=True)
    subprocess.run([str(path/'probe')],check=True)
