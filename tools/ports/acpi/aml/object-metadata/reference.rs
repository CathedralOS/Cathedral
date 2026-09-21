// SPDX-License-Identifier: MIT OR Apache-2.0
// Original observations of actual public upstream APIs; no handler or interpreter.
use acpi::aml::object::{MethodFlags,DeviceStatus};
fn main() {
 for raw in 0..=255u8 {
  let value=MethodFlags(raw);
  println!("method\t{raw}\t{}\t{}\t{}",value.arg_count(),value.serialize(),value.sync_level());
 }
 for prefix in [0u64,32,1<<31,1<<32,1<<63,u64::MAX&!31] {
  for low in 0..32u64 {
   let raw=prefix|low; let value=DeviceStatus(raw);
   println!("status\t{raw}\t{}\t{}\t{}\t{}\t{}",value.present(),value.enabled(),value.show_in_ui(),value.functioning(),value.battery_present());
  }
 }
}
