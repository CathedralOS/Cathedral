"""Read the restricted, relocation-free Omega component used by the UI adapter.

The compiler has already applied internal PC-relative relocations. Preserve the
text-to-data spacing while discarding ELF lookup tables; the result is linked as
ordinary application text. Mutable state is supplied by the caller. This is
packaging of compiled code, not a source compiler or a component trust verdict.
"""
import struct


class Elf:
    def __init__(self, data, kind):
        self.data = data
        self.require(len(data) >= 64 and data[:8] == b'\x7fELF\x02\x01\x01\0', 'ELF64 little-endian System V header')
        self.require(self.unpack('<HH', 16) == (kind, 62), 'x86-64 ELF container type')
        self.require(self.unpack('<H', 52)[0] == 64, 'ELF header size')
        phoff, shoff = self.unpack('<QQ', 32)
        phsize, count, shsize, shcount, strings = self.unpack('<HHHHH', 54)
        self.require(phsize == 56 and 0 < count <= 16, 'program header roster')
        self.loads = []
        self.dynamic = None
        for index in range(count):
            row = self.unpack('<IIQQQQQQ', phoff + index * 56)
            tag, flags, offset, address, _, files, memory, alignment = row
            self.require(files <= memory and address + memory < 1 << 64, 'load extent')
            self.bytes(offset, files)
            if tag == 1:
                self.require(flags & 4 and flags & 3 != 3, 'readable, non-W+X load')
                self.require(alignment > 0 and alignment & (alignment - 1) == 0, 'load alignment')
                self.require(offset % alignment == address % alignment, 'load address alignment')
                self.require(all(address + memory <= base or base + previous <= address
                                 for _, base, _, previous, _ in self.loads), 'overlapping loads')
                self.loads.append((offset, address, files, memory, flags))
            elif tag == 2:
                self.require(self.dynamic is None and files % 16 == 0, 'one dynamic table')
                self.dynamic = (offset, files)
        self.sections = []
        if shcount:
            self.require(shsize == 64 and shcount <= 128 and strings < shcount, 'section header roster')
            rows = [self.unpack('<IIQQQQIIQQ', shoff + index * 64) for index in range(shcount)]
            names = self.bytes(rows[strings][4], rows[strings][5])
            for row in rows:
                name = self.string(names, row[0])
                self.sections.append((name, row))

    @staticmethod
    def require(condition, reason):
        if not condition:
            raise ValueError(reason)

    def bytes(self, offset, count):
        self.require(0 <= offset <= len(self.data) and 0 <= count <= len(self.data) - offset, 'file range')
        return self.data[offset:offset + count]

    def unpack(self, form, offset):
        return struct.unpack(form, self.bytes(offset, struct.calcsize(form)))

    @staticmethod
    def string(table, offset):
        if not 0 <= offset < len(table):
            raise ValueError('string offset')
        end = table.find(b'\0', offset)
        if end < 0:
            raise ValueError('unterminated string')
        return table[offset:end]

    def virtual(self, address, count):
        for offset, base, files, _, _ in self.loads:
            if base <= address and address + count <= base + files:
                return self.bytes(offset + address - base, count)
        raise ValueError('unbacked virtual range')

    def needed(self):
        if self.dynamic is None:
            return []
        offset, size = self.dynamic
        values = [self.unpack('<QQ', offset + index) for index in range(0, size, 16)]
        self.require(any(tag == 0 for tag, _ in values), 'terminated dynamic table')
        strings = [value for tag, value in values if tag == 5]
        sizes = [value for tag, value in values if tag == 10]
        self.require(len(strings) == len(sizes) == 1, 'dynamic string table')
        table = self.virtual(strings[0], sizes[0])
        return [self.string(table, value) for tag, value in values if tag == 1]


def component(data):
    elf = Elf(data, 3)
    elf.require(not elf.needed(), 'UI component must have no external dependencies')
    text = [(name, row) for name, row in elf.sections if name == b'.text']
    elf.require(len(text) == 1, 'one text section')
    section = text[0][1]
    start = section[3]
    end = start + section[5]
    elf.require(section[5] > 0 and section[2] & 4, 'executable text')
    exports = []
    for section_name, row in elf.sections:
        _, tag, flags, address, offset, size, link, _, alignment, entry_size = row
        elf.require(not flags & 0x400, 'TLS is unsupported')
        elf.require(not (tag in (4, 9) and size), 'runtime relocations are unsupported')
        elf.require(not (tag == 8 and size), 'internal mutable state is unsupported')
        if tag == 11:
            elf.require(entry_size == 24 and size % 24 == 0 and link < len(elf.sections), 'dynamic symbols')
            names = elf.sections[link][1]
            table = elf.bytes(names[4], names[5])
            for at in range(offset, offset + size, 24):
                name, info, _, defined, value, length = elf.unpack('<IBBHQQ', at)
                elf.require(defined or not name, 'undefined component symbol')
                if info == 0x12 and defined:
                    elf.require(start <= value and value + length <= start + section[5] and length > 0, 'export text range')
                    exports.append((elf.string(table, name).decode('ascii'), value - start))
        if flags & 2 and tag == 1 and address >= start and size and section_name != b'.got.plt':
            elf.require(section_name in (b'.text', b'.rodata', b'.data'), 'unsupported allocated code/data section')
            elf.require(elf.virtual(address, size) == elf.bytes(offset, size), 'section/load disagreement')
            elf.require(alignment <= 16, 'component data alignment exceeds static adapter contract')
            end = max(end, address + size)
    elf.require(len(exports) == 1, 'one exact requirement export per component')
    elf.require(end - start <= 1024 * 1024, 'component exceeds adapter code budget')
    # Each initialized section must retain its original relative address.
    for _, address, files, memory, _ in elf.loads:
        elf.require(files == memory, 'component load has implicit initialization')
    body = elf.virtual(start, section[5])
    image = bytearray(end - start)
    image[:len(body)] = body
    for name, row in elf.sections:
        if name != b'.got.plt' and row[2] & 2 and row[1] == 1 and start <= row[3] < end and row[5]:
            image[row[3] - start:row[3] - start + row[5]] = elf.bytes(row[4], row[5])
    return bytes(image), exports[0]
