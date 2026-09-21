// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2018 Isaac Woods. Exact private copy_bits body below is a mirror;
// Object clone/read_buffer_field/to_integer calls use the real pinned crate.
// No forged ObjectToken, WrappedObject mutable access, AML opcode or live handler.
use acpi::aml::{IntegerSize,object::{Object,ReferenceKind}};
pub(crate) fn copy_bits(
    src: &[u8],
    mut src_index: usize,
    dst: &mut [u8],
    mut dst_index: usize,
    mut length: usize,
) {
    while length > 0 {
        let src_shift = src_index & 7;
        let mut src_bits = src.get(src_index / 8).unwrap_or(&0x00) >> src_shift;
        if src_shift > 0 && length > (8 - src_shift) {
            src_bits |= src.get(src_index / 8 + 1).unwrap_or(&0x00) << (8 - src_shift);
        }

        if length < 8 {
            src_bits &= (1 << length) - 1;
        }

        let dst_shift = dst_index & 7;
        let mut dst_mask: u16 = if length < 8 { ((1 << length) - 1) as u16 } else { 0xff_u16 } << dst_shift;
        dst[dst_index / 8] =
            (dst[dst_index / 8] & !(dst_mask as u8)) | ((src_bits << dst_shift) & (dst_mask as u8));

        if dst_shift > 0 && length > (8 - dst_shift) {
            dst_mask >>= 8;
            dst[dst_index / 8 + 1] &= !(dst_mask as u8);
            dst[dst_index / 8 + 1] |= (src_bits >> (8 - dst_shift)) & (dst_mask as u8);
        }

        if length < 8 {
            length = 0;
        } else {
            length -= 8;
            src_index += 8;
            dst_index += 8;
        }
    }
}

fn main(){
 let data:Vec<u8>=(0..256).map(|i|((i*29+7)&255)as u8).collect();
 for width in [4usize,8] {
  let size=if width==4 {IntegerSize::FourBytes}else{IntegerSize::EightBytes};
  for (label,start,count)in [("low",0,8),("cross",5,17),("native",0,width*8),("last",2040,8),("too_wide",0,width*8+1)] {
   let field=Object::BufferField {buffer:Object::Buffer(data.clone()).wrap(),offset:start,length:count};
   let read=field.read_buffer_field(size).unwrap();
   let kind=match &read {Object::Integer(_)=>"Integer",Object::Buffer(_)=>"Buffer",_=>panic!("kind")};
   let numeric=read.to_integer(size).unwrap();
   println!("{{\"case\":\"field_read_{width}_{label}\",\"stage\":\"actual public read_buffer_field then to_integer\",\"kind\":{kind:?},\"number\":{numeric}}}");
  }
  for (label,start,count,value)in [("cross",5,17,u64::MAX),("native",0,width*8,0x8877665544332211),("zero_pad",0,128,u64::MAX),("last",2040,8,0x51)] {
   let bytes=value.to_le_bytes();let mut output=data.clone();copy_bits(&bytes[..width],0,&mut output,start,count);
   println!("{{\"case\":\"field_write_{width}_{label}\",\"stage\":\"exact private copy_bits body mirror\",\"bytes\":{output:?}}}");
  }
 }
 let original=Object::Buffer(vec![65,66]);let mut copied=original.clone();
 if let Object::Buffer(ref mut bytes)=copied {bytes[0]=99;}else{panic!("copy type")};
 assert_eq!(original.as_buffer().unwrap(),[65,66]);assert_eq!(copied.as_buffer().unwrap(),[99,66]);
 println!("{{\"case\":\"public_buffer_deep_clone\",\"independent\":true}}");
 let original=Object::String(String::from("AB"));let mut copied=original.clone();
 if let Object::String(ref mut text)=copied {text.replace_range(0..1,"C");}else{panic!("copy type")};
 assert_eq!(original.to_buffer(IntegerSize::EightBytes).unwrap(),[65,66]);assert_eq!(copied.to_buffer(IntegerSize::EightBytes).unwrap(),[67,66]);
 println!("{{\"case\":\"public_string_deep_clone\",\"independent\":true}}");
 let backing=Object::Buffer(vec![65,66]).wrap();let alias=backing.clone();assert!(core::ptr::eq(&*alias,&*backing));
 let field=Object::BufferField {buffer:backing.clone(),offset:0,length:8};let copied=field.clone();
 if let Object::BufferField{buffer,..}=copied {assert!(core::ptr::eq(&*buffer,&*backing));}else{panic!("field clone")};
 let reference=Object::Reference {kind:ReferenceKind::RefOf,inner:backing.clone()};
 if let Object::Reference{inner,..}=reference.clone(){assert!(core::ptr::eq(&*inner,&*backing));}else{panic!("ref clone")};
 let package=Object::Package(vec![backing.clone()]);if let Object::Package(items)=package.clone(){assert!(core::ptr::eq(&*items[0],&*backing));}else{panic!("package clone")};
 println!("{{\"case\":\"public_shared_identity_clones\",\"wrapped_reference_field_package_shared\":true}}");
}
