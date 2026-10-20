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
