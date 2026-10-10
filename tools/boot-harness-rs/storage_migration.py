"""Load the previous format, edit it through the real app, then verify another reboot."""
import binascii
import struct
import capture
import disk_image


def run(boot_with, output):
    path = output / "test-storage.raw"
    disk_image.prepare(path, reset=True)
    body, commit = bytearray(512), bytearray(512)
    body[:8] = b"CTOBJ001"
    struct.pack_into("<QQQ", body, 8, 7, 3, 1)
    body[32:35] = bytes((1, 1, 5))
    struct.pack_into("<I", body, 508, binascii.crc32(body[:508]))
    commit[:8] = b"CTCOM001"
    struct.pack_into("<Q", commit, 8, 7)
    commit[16:20] = body[508:]
    struct.pack_into("<I", commit, 508, binascii.crc32(commit[:508]))
    with path.open("r+b") as image:
        image.write(body + commit)

    def convert(stream, folder, serial, process, timeout):
        capture.wait_log(serial, process, 0, ["Cathedral: selection=1 toggles=5",
                         "Cathedral: storage=00 saved=0007"], timeout)
        capture.snapshot(stream, folder, 1, 5, saved=7, legacy=True)
        offset = len(serial.read_bytes())
        capture.key_event(stream, "right")
        capture.wait_log(serial, process, offset, ["Cathedral: selection=2 toggles=5",
                         "Cathedral: storage=00 saved=0008"], timeout)
        capture.snapshot(stream, folder, 2, 5, saved=8)

    def verify(stream, folder, serial, process, timeout):
        capture.wait_log(serial, process, 0, ["Cathedral: selection=2 toggles=5",
                         "Cathedral: storage=00 saved=0008"], timeout)
        capture.snapshot(stream, folder, 2, 5, saved=8)

    boot_with(convert)
    boot_with(verify)
    print("PASS: legacy saved state converted atomically and survived reboot", flush=True)
