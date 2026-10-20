//! Periodic best-effort probe. Tick observations are not WCET or reservations.
//! Collect before printing so diagnostic serial traffic is outside the workload.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{Error, event, time, write, yield_now};
use core::fmt::Write;

pub fn periodic() -> u64 {
    let mut samples = [[0u64; 4]; 16];
    let mut release = time::after(8).unwrap();
    for sample in &mut samples {
        assert_eq!(event::wait(&[], release), Err(Error(abi::TIMED_OUT as i64)));
        let start = time::now().unwrap();
        assert!(time::reached(start, release));
        let cpu = time::cpu_us().unwrap();
        while time::cpu_us().unwrap() - cpu < 1_000 {
            core::hint::spin_loop();
        }
        let finish = time::now().unwrap();
        *sample = [release, start, finish, time::cpu_us().unwrap() - cpu];
        // Absolute releases expose lateness; delaying the next release until
        // completion would silently hide missed periods.
        release = release.wrapping_add(8);
    }
    for [release, start, finish, cpu] in samples {
        let mut record = Record {
            bytes: [0; 128],
            len: 0,
        };
        writeln!(
            record,
            "CPU_TIMING release={release} start={start} finish={finish} cpu_us={cpu}"
        )
        .unwrap();
        write(&record.bytes[..record.len]).unwrap();
    }
    0
}

pub fn noise(yielding: u64) -> u64 {
    if yielding >= 2 {
        return memory_pressure(if yielding == 2 { 1 } else { 128 });
    }
    let stop = time::after(240).unwrap();
    while !time::reached(time::now().unwrap(), stop) {
        if yielding == 0 {
            core::hint::spin_loop();
        } else {
            yield_now();
        }
    }
    0
}

fn memory_pressure(pages: usize) -> u64 {
    use cathedral_user_runtime::memory::Private;
    let stop = time::after(240).unwrap();
    let mut cycles = 0;
    while cycles == 0 || !time::reached(time::now().unwrap(), stop) {
        let mut private = Private::allocate(pages).unwrap();
        let bytes = private.bytes_mut();
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[bytes.len() - 1], 0);
        bytes[0] = 0x5a;
        let mut object = private.freeze().unwrap();
        object.unmap().unwrap();
        assert_eq!(object.map().unwrap()[0], 0x5a);
        drop(object);
        cycles += 1;
    }
    // Leave four owned objects, two mapped, for production task-exit cleanup.
    // The local mapping ledger admits two maximum-size objects beside the image.
    // No pointer escapes; the kernel owns reclamation after this process exits.
    for index in 0..4 {
        let private = Private::allocate(pages).unwrap();
        if index < 2 {
            let mut object = private.freeze().unwrap();
            object.unmap().unwrap();
            core::mem::forget(object);
        } else {
            core::mem::forget(private);
        }
    }
    0
}

struct Record {
    bytes: [u8; 128],
    len: usize,
}
impl Write for Record {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let end = self.len + text.len();
        let destination = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        destination.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
