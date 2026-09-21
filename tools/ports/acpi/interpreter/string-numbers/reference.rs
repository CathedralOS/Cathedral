// SPDX-License-Identifier: MIT OR Apache-2.0
// Exact pure result-construction mirror of private do_to_dec_hex_string at
// acpi 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5, src/aml/mod.rs:2100-2133.
// Object resolution, target Store, argument contribution and retirement omitted.
// This is NOT a call to the upstream private operation.
use acpi::aml::{IntegerSize, object::Object};
fn format_mirror(operand:&Object,hex:bool)->String {
    match operand {
        Object::String(value)=>value.clone(),
        Object::Integer(value)=>if hex {format!("{value:#X}")}else{value.to_string()},
        Object::Buffer(bytes)=>{
            if bytes.is_empty(){String::new()}else{
                let mut string=String::new();
                for byte in bytes {
                    let as_str=if hex {format!("{byte:#04X},")}else{format!("{byte},")};
                    string.push_str(&as_str);
                }
                if !string.is_empty(){string.pop();}
                string
            }
        }
        _=>panic!("Mirror only admits String/Integer/Buffer fixtures"),
    }
}
fn hexadecimal(value:&str)->String {value.as_bytes().iter().map(|byte|format!("{byte:02x}")).collect()}
