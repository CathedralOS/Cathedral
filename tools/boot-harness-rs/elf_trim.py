"""Reduce an ELF to what the kernel loads: the header, the program headers and
the loadable segments. Debug information and section headers are dropped.
An optional distribution display title follows the ELF; it is not loaded code
or authority metadata. The chooser reads this 32-byte footer through storage."""
import struct
from pathlib import Path


def title_record(title):
    if title is None:
        return b''
    if not isinstance(title, str) or not 1 <= len(title) <= 13 or any(not 32 <= ord(c) <= 126 for c in title) or title.strip() != title:
        raise ValueError('display_title must be 1..13 printable ASCII characters without outer spaces')
    return b'CTHTIT01' + title.encode('ascii').ljust(24, b'\0')


def trim(source, destination, title=None):
    footer = title_record(title)
    data = bytearray(Path(source).read_bytes())
    if data[:4] != b'\x7fELF' or data[4] != 2:
        raise ValueError(f'{source} is not an ELF64 file')
    phoff, = struct.unpack_from('<Q', data, 0x20)
    phentsize, phnum = struct.unpack_from('<HH', data, 0x36)
    if phoff + phnum * phentsize > 4096:
        raise ValueError(f'{source}: program headers must sit in the first page')
    end = phoff + phnum * phentsize
    for index in range(phnum):
        kind, _flags, offset, _vaddr, _paddr, filesz = struct.unpack_from('<IIQQQQ', data, phoff + index * phentsize)
        if kind == 1:
            end = max(end, offset + filesz)
    if end > len(data):
        raise ValueError(f'{source}: a loadable segment reaches past the file')
    # No section headers: offset, count, size and string index all zero.
    struct.pack_into('<Q', data, 0x28, 0)
    struct.pack_into('<HHH', data, 0x3a, 0, 0, 0)
    Path(destination).parent.mkdir(parents=True, exist_ok=True)
    Path(destination).write_bytes(bytes(data[:end]) + footer)
    return end + len(footer)
