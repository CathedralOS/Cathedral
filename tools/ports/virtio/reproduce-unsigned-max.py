#!/usr/bin/env python3
"""Compare inferred versus explicit u64 maximum arithmetic in the semantic evaluator."""
import argparse
from pathlib import Path
import subprocess
import tempfile
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--omega',type=Path,required=True)
args=parser.parse_args()
source='''data Span [copy] { offset: u64; length: u64; }
machine max_span_result() -> i32 {
    let value: Span = Span { offset: 4096, length: 32 };
    transition value.length <= (MAXIMUM - value.offset) { true -> (0) _ -> (1) }
}
const RESULT: i32 = max_span_result();
machine require_success(value: i32) requires value == 0; {}
data Main {}
machine Main::main(&mut self) { require_success(RESULT); }
'''
with tempfile.TemporaryDirectory(prefix='cathedral-unsigned-max-') as directory:
    folder=Path(directory)
    (folder/'build.omg').write_text('machine build(builder: &mut Build) { builder.application("unsigned-max"); builder.freestanding = true; }\n')
    for label,maximum in [('inferred','0xffffffffffffffff'),('explicit','(0xffffffffffffffff as u64)')]:
        (folder/'main.omg').write_text(source.replace('MAXIMUM',maximum))
        result=subprocess.run([str(args.omega.resolve()),'--check',str(folder/'main.omg')],capture_output=True,text=True)
        print(label,'exit',result.returncode,flush=True)
        print(result.stdout+result.stderr,flush=True)
