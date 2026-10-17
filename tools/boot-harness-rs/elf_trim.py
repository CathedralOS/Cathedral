"""Reduce an ELF to what the kernel loads: the header, the program headers and
the loadable segments. Debug information and section headers are dropped, so
a program object in the store holds only the bytes that become pages."""
import struct
from pathlib import Path


def trim(source, destination):
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
    Path(destination).write_bytes(bytes(data[:end]))
    return end
