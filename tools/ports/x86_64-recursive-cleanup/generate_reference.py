#!/usr/bin/env python3
"""Extract the private recursive cleanup body with explicit owned-registry plumbing."""
import argparse
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args()
 source=(ROOT/'reference_code/rust-osdev/x86_64/src/structures/paging/mapper/recursive_page_table.rs').read_text()
 start=source.index('        fn clean_up(\n');brace=source.index('{',start);end=brace+1;depth=1
 while depth:depth+=(source[end]=='{')-(source[end]=='}');end+=1
 body=source[start:end]
 parameter='            frame_deallocator: &mut impl FrameDeallocator<Size4KiB>,\n'
 assert body.count(parameter)==1;body=body.replace(parameter,parameter+'            registry: &Registry,\n')
 pointer='''                        let page_table =
                            [p1_ptr, p2_ptr, p3_ptr][level as usize - 2](start, recursive_index);'''
 assert body.count(pointer)==1;body=body.replace(pointer,'                        let page_table = registry.resolve(frame);')
 recursive='''                            frame_deallocator,
                        )'''
 assert body.count(recursive)==1;body=body.replace(recursive,'                            frame_deallocator,\n                            registry,\n                        )')
 assert body.count('VirtAddr::forward_checked_impl(')==1;body=body.replace('VirtAddr::forward_checked_impl(','<VirtAddr as Step>::forward_checked(')
 addr=(ROOT/'reference_code/rust-osdev/x86_64/src/addr.rs').read_text()
 assert 'fn forward_checked(start: Self, count: usize) -> Option<Self> {\n        Self::forward_checked_impl(start, count)\n    }' in addr
 result='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Private clean_up body from x86_64 cc35c876d3badb57df54a66e22f7768a52be95f2.
// Only pointer resolution is substituted by a stable owned registry (with
// parameter plumbing), and the private step helper is reached via public Step.
// This is an extracted mirror, not a call to RecursivePageTable::clean_up.
'''+body+'\n'
 path=HERE/'src/private_cleanup.rs'
 if args.check:assert path.read_text()==result,'private cleanup mirror differs from pinned extraction'
 else:path.write_text(result)
 print('Pinned private cleanup body and same-body public Step substitution verified.')
if __name__=='__main__':main()
