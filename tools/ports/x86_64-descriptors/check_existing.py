#!/usr/bin/env python3
"""Run the existing stack-policy source through modern isolated packaging."""
from pathlib import Path
import subprocess,sys,tempfile
ROOT=Path(__file__).resolve().parents[3]
omega=Path(sys.argv[1]).resolve()
baseline='--baseline' in sys.argv
with tempfile.TemporaryDirectory(prefix='cathedral-existing-ist-') as directory:
 p=Path(directory)
 facts_path=ROOT/'source/drivers/facts'
 if baseline:
  facts_path=p/'facts';facts_path.mkdir()
  for source_file in (ROOT/'source/drivers/facts').glob('*.omg'):(facts_path/source_file.name).write_bytes(source_file.read_bytes())
  original=subprocess.check_output(['git','show','cbbf9b6c9c9365728c24fcedafe6cb6fdd3bfdda:source/drivers/facts/x86_interrupt_stacks.omg'],cwd=ROOT)
  (facts_path/'x86_interrupt_stacks.omg').write_bytes(original)
 source=(ROOT/'source/core/x86_interrupt_profile.omg').read_text().replace('use omega::language::std::calling;', 'use std::calling;')
 (p/'x86_interrupt_profile.omg').write_text(source)
 (p/'main.omg').write_text('use x86_interrupt_profile;\ndata Main {}\nmachine Main::main(&mut self) {}\n')
 (p/'build.omg').write_text('machine build(builder: &mut Build) { builder.application("cathedral-existing-ist"); builder.freestanding = true; builder.depend_as("std", Source::Path { location: "'+str(ROOT.parent/'Omega/source/library/std')+'" }); builder.depend_as("facts", Source::Path { location: "'+str(facts_path)+'" }); }\n')
 result=subprocess.run([str(omega),'--check',str(p/'main.omg')],cwd=ROOT,capture_output=True,text=True)
 print(result.stdout+result.stderr,end='')
 sys.exit(result.returncode)
