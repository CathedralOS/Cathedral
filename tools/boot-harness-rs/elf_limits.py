"""Report compiler-produced ELF page demand before bundling or installing it.

This is a build diagnostic, not the kernel's geometry, permission or authority
validator. Count each PT_LOAD memory extent independently, including zero-fill.
"""
from pathlib import Path
import re
import struct

ROOT = Path(__file__).resolve().parents[2]
GEOMETRY = ROOT / 'source-rs/kernel/core/user_tasks/address_space/address_space.rs'
PAGE = 4096


def image_limit():
    declaration = re.search(r'^pub const MAX_IMAGE_PAGES: usize = ([0-9_]+);$',
                            GEOMETRY.read_text(encoding='utf-8'), re.MULTILINE)
    if declaration is None:
        raise ValueError(f'Cannot read kernel image-page limit from {GEOMETRY}')
    return int(declaration[1].replace('_', ''))


def load_pages(data):
    if len(data) < 64 or data[:6] != b'\x7fELF\x02\x01':
        raise ValueError('expected a little-endian ELF64 image')
    offset, = struct.unpack_from('<Q', data, 32)
    size, count = struct.unpack_from('<HH', data, 54)
    if size != 56 or count == 0 or offset + size * count > len(data):
        raise ValueError('invalid ELF program-header table')
    pages = 0
    loads = 0
    for index in range(count):
        kind, _, _, _, _, _, memory, _ = struct.unpack_from('<IIQQQQQQ', data, offset + index * size)
        if kind == 1:
            loads += 1
            pages += (memory + PAGE - 1) // PAGE
    if not loads:
        raise ValueError('ELF has no loadable segments')
    return pages


def check(source):
    source = Path(source)
    try:
        pages = load_pages(source.read_bytes())
    except ValueError as error:
        raise ValueError(f'{source}: {error}') from error
    limit = image_limit()
    if pages > limit:
        raise ValueError(f'{source}: image needs {pages} load pages, kernel limit is {limit}; '
                         'reduce the image before boot (PT_LOAD includes BSS)')
    print(f'image {source.name}: {pages}/{limit} load pages ({limit - pages} free)', flush=True)
    return pages
