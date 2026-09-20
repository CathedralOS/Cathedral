#!/usr/bin/env python3
"""Observe actual pinned packed layout and bitfield encodings, without hardware."""
import hashlib
from pathlib import Path
import subprocess
import sys
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parent))
import inventory, rust_layout, vectors
inventory.check(inventory.read_json(ROOT/'source/libraries/virtio/inventory.json'),
                ROOT/'reference_code/rust-osdev/virtio-spec-rs', ROOT)
expected = vectors.validate(inventory.read_json(ROOT/'source/libraries/virtio/packed.vectors.json'))
measured, artifact = rust_layout.measure(HERE/'Cargo.toml', HERE/'src/packed_probe.rs', 'CATHEDRAL_VIRTIO_PACKED', 'cathedral_virtio_layout_probe')
if set(measured) != set(expected['measurements']): raise ValueError('packed measurement coverage differs')
for key,value in measured.items():
    if expected['measurements'][key]['value'] != value: raise ValueError(f'{key}: target geometry differs')
print(f'{len(measured)} actual Rust packed size/alignment/offset measurements agree. LLVM SHA-256: '+hashlib.sha256(artifact).hexdigest(), flush=True)
subprocess.run(['cargo','run','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,check=True)
print('Omega storage ABI and DMA/device access NOT RUN.')
