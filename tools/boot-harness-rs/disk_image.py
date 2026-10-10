"""Dedicated data disk; never attach a host block device or the firmware ESP here."""
SIZE = 4 * 1024 * 1024

def prepare(path, reset=False):
    path.parent.mkdir(parents=True, exist_ok=True)
    if reset and path.name != "test-storage.raw":
        raise RuntimeError("Only the harness's disposable test-storage.raw may be reset")
    if path.is_symlink():
        raise RuntimeError(f"Lab disk image must not be a symlink: {path}")
    if path.exists() and not path.is_file():
        raise RuntimeError(f"Lab disk image must be a regular file: {path}")
    if reset or not path.exists():
        with path.open("wb" if reset else "xb") as image:
            image.truncate(SIZE)
    if not path.is_file() or path.stat().st_size != SIZE:
        raise RuntimeError(f"Expected a {SIZE}-byte lab disk image: {path}")

def arguments(path):
    filename = str(path.resolve()).replace(",", ",,")
    return ["-device", "isa-ide,id=cathedral-ata,iobase=0x170,iobase2=0x376,irq=15",
            "-drive", f"if=none,id=cathedral-data,format=raw,cache=writeback,file={filename}",
            "-device", "ide-hd,drive=cathedral-data,bus=cathedral-ata.0,unit=0,serial=CATHEDRAL-LAB-DATA"]
