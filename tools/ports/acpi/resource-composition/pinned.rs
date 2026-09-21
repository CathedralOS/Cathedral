// SPDX-License-Identifier: MIT OR Apache-2.0
// Exact private ConcatRes result block; only Object wrapping -> returned Vec.
fn concat_mirror(source1:&[u8],source2:&[u8])->Vec<u8>{

                            fn strip_end_tag(buf: &[u8]) -> &[u8] {
                                if buf.len() >= 2 && buf[buf.len() - 2] == 0x79 {
                                    buf.split_at(buf.len() - 2).0
                                } else {
                                    buf
                                }
                            }
                            let mut buffer = Vec::from(strip_end_tag(source1));
                            buffer.extend_from_slice(strip_end_tag(source2));
                            // Add a new end-tag
                            buffer.push(0x79);
                            // Don't calculate the new real checksum - just use 0
                            buffer.push(0x00);
                            buffer
                        
}
