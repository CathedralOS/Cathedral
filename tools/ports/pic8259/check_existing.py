#!/usr/bin/env python3
"""Check exact existing PIC source in a modern isolated package, never execute I/O."""
from pathlib import Path
import re
import subprocess
import sys
import tempfile
ROOT = Path(__file__).resolve().parents[3]
OMEGA = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT.parent / 'Omega/target/release/omega'
facts = (ROOT / 'source/platform/drivers/facts/pic_8259.omg').read_text()
constants = {name: int(value, 0) for name, value in re.findall(r'public const (\w+):\s*u(?:8|16)\s*=\s*(0x[0-9a-fA-F]+|\d+);', facts)}
source = (ROOT / 'source/kernel/arch/board/pc/timer/pic.omg').read_text()
def number(value):
    return constants[value] if value in constants else int(value, 0)
def body(declaration):
    return source.split(declaration, 1)[1].split('\n}', 1)[0]
expected = {
    'public machine Pic8259::remap_masked(': [(0x20,0x11),(0xa0,0x11),(0x21,0x20),(0xa1,0x28),(0x21,4),(0xa1,2),(0x21,1),(0xa1,1),(0x21,255),(0xa1,255)],
    'public machine Pic8259::unmask_timer(': [(0x21,0xfe),(0xa1,255)],
    'machine complete(': [(0x20,0x20)],
}
for name, writes in expected.items():
    text = body(name)
    actual = [(number(port), number(value)) for port, value in re.findall(r'\bout\s+\|pic_8259\|::(\w+)\s*,\s*(?:\|pic_8259\|::)?(\w+)',text)]
    if actual != writes or 'reaches |assembly|::PortIo' not in text:
        raise SystemExit(name + ': existing ordered write sequence or explicit PortIo changed')
ack = body('machine complete(')
if ('requires acknowledgement in |interrupt|::InterruptAcknowledgement::Pending' not in ack
        or 'ensures timer_acknowledgement_completed(acknowledgement)' not in ack
        or 'asm ensures timer_acknowledgement_completed(acknowledgement)' not in ack):
    raise SystemExit('PIC acknowledgement lifecycle/order changed')
print('PASS source sequence/PortIo/Pending settlement regression checks (not compiler artifact validation)',flush=True)
with tempfile.TemporaryDirectory(prefix='cathedral-pic-source-') as directory:
    project = Path(directory)
    (project / 'board/pc/timer').mkdir(parents=True)
    (project / 'board/pc/timer/pic.omg').write_bytes((ROOT / 'source/kernel/arch/board/pc/timer/pic.omg').read_bytes())
    (project / 'main.omg').write_text('use board/pc/timer/pic.omg;\ndata Main {}\nmachine Main::main(&mut self) {}\n')
    (project / 'build.omg').write_text('machine build(builder: &mut Build) { builder.application("pic-existing-source-check"); builder.freestanding = true; builder.depend_as("facts", Source::Path { location: "' + str(ROOT / 'source/platform/drivers/facts') + '" }); }\n')
    subprocess.run([str(OMEGA),'--check',str(project/'main.omg')],cwd=ROOT,check=True)
print('PASS exact existing PC PIC source with actual facts dependency; no PortIo performed',flush=True)
