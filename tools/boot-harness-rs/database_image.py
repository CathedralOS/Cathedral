"""Persistent database desktop media. Only a missing image may be formatted;
the profile's programs are installed into it as objects, and reinstalled as
new revisions whenever their bytes change. What was installed is recorded in
a sidecar beside the image, so an unchanged set never reopens the image."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

VOLUME_BYTES = 4 * 1024 * 1024
OFFSET = 4096
IMAGE_BYTES = OFFSET + VOLUME_BYTES + 4096


def require_available(path):
    """Diagnose Windows sharing conflicts before building or changing boot files.

    This read-only probe is not a reservation; the installer and QEMU still
    enforce their own locks when they open the image.
    """
    if os.name != 'nt':
        return
    import ctypes
    from ctypes import wintypes
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                                  wintypes.LPVOID, wintypes.DWORD, wintypes.DWORD,
                                  wintypes.HANDLE]
    kernel.CreateFileW.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    # GENERIC_READ, no sharing, OPEN_EXISTING. Never creates or writes the disk.
    handle = kernel.CreateFileW(str(Path(path).resolve()), 0x80000000, 0, None, 3, 0x80, None)
    if handle == wintypes.HANDLE(-1).value:
        error = ctypes.get_last_error()
        if error in (2, 3):  # A missing image is created during installation.
            return
        if error in (32, 33):
            raise RuntimeError(
                f'Cathedral disk is already in use: {path}\n'
                'Close the existing Cathedral QEMU window (save any work first), '
                'or press Ctrl+C in its launch terminal, then press F5 again. '
                'The saved disk has not been changed by this launch.')
        raise ctypes.WinError(error)
    kernel.CloseHandle(handle)


def oracle():
    """Build the host storage lab once and return its executable."""
    root = Path(__file__).resolve().parents[2]
    lab = root / 'tools/storage-lab'
    subprocess.run(['cargo', 'build', '--locked', '--release', '--features', 'engines'], cwd=lab, check=True)
    return lab / 'target/release' / ('storage-layout.exe' if os.name == 'nt' else 'storage-layout')


def prepare(path, programs=(), label_migration=None, app_close=False, realm_programs=()):
    """Create the image if missing, then install `programs`: (parent, file) pairs
    that become objects 1.. of root 0 in order, each under an earlier object or
    the root. An existing image keeps its data; only changed programs are
    republished. The stock desktop may also request its explicit readable-label
    migration, which upgrades recognized copied launchers and labels old apps.
    The migration saves a backup and leaves customized code and user data alone.
    app_close refreshes recognized stock Notes/Colors and matrix code. The stock
    migration also installs hierarchy metadata and matrix configuration."""
    if realm_programs and label_migration is None:
        raise ValueError('named realm additions require the explicit stock hierarchy migration')
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise RuntimeError(f'Database image must be a regular file: {path}')
    executable = None
    sidecar = path.with_name(path.name + '.programs')
    existing = path.exists()
    if existing:
        if path.stat().st_size != IMAGE_BYTES:
            raise RuntimeError(f'Unexpected database image size; refusing to replace {path}')
    else:
        executable = oracle()
        lab = executable.parents[2]
        sidecar.unlink(missing_ok=True)  # A fresh image holds nothing yet, whatever was recorded.
        with tempfile.TemporaryDirectory(prefix='database-format-', dir=path.parent) as directory:
            volume = Path(directory) / 'volume.raw'
            subprocess.run([str(executable), 'engines', 'volume', 'format', str(volume), str(VOLUME_BYTES), '1'], cwd=lab, check=True)
            data = volume.read_bytes()
            assert len(data) == VOLUME_BYTES
            # Exclusive creation cannot overwrite a concurrent creator or old media.
            with path.open('xb') as file:
                file.write(b'\x5a' * OFFSET + data + b'\xa5' * 4096)
                file.flush()
                os.fsync(file.fileno())
    if not programs:
        return
    installed = [[int(parent), hashlib.sha256(Path(file).read_bytes()).hexdigest()] for parent, file in programs]
    if label_migration is not None:
        installed = {'programs': installed, 'readable_labels': hashlib.sha256(Path(label_migration).read_bytes()).hexdigest(), 'app_close': app_close, 'hierarchy': 1, 'realm_programs': [[name, hashlib.sha256(Path(file).read_bytes()).hexdigest()] for name, file in realm_programs]}
    previous = json.loads(sidecar.read_text(encoding='utf-8')) if sidecar.exists() else {}
    fingerprint = {k: v for k, v in previous.items() if k != 'realm_manifests'} if isinstance(previous, dict) else previous
    if fingerprint == installed:
        return
    if executable is None:
        executable = oracle()
    lab = executable.parents[2]
    # A guest may have freed records without shrinking the engine. Release
    # that physical slack before an upgrade needs room for both old and new
    # programs. The host backs up the locked image and verifies all records.
    # Unchanged installations returned above without even opening the image.
    if existing:
        subprocess.run([str(executable), 'engines', 'volume', 'compact', str(path),
                        str(OFFSET), str(VOLUME_BYTES)], cwd=lab, check=True)
    command = [str(executable), 'engines', 'volume', 'seed', str(path), str(OFFSET), str(VOLUME_BYTES)]
    for parent, file in programs:
        command += [str(parent), str(file)]
    subprocess.run(command, cwd=lab, check=True)
    if label_migration is not None:
        subprocess.run([str(executable), 'engines', 'volume', 'upgrade-app-close' if app_close else 'upgrade-labels', str(path),
                        str(OFFSET), str(VOLUME_BYTES), str(Path(label_migration).resolve())], cwd=lab, check=True)
        known = previous.get('realm_manifests', {}) if isinstance(previous, dict) else {}
        additions = [str(item) for name, file in realm_programs for item in (name, file, known.get(name, '-'))]
        result = subprocess.run([str(executable), 'engines', 'volume', 'upgrade-hierarchy', str(path),
                                 str(OFFSET), str(VOLUME_BYTES), *additions], cwd=lab, check=False, capture_output=True, text=True)
        print(result.stdout, end='')
        if result.returncode:
            print(result.stderr, end='', file=sys.stderr)
            result.check_returncode()
        installed['realm_manifests'] = dict(line.removeprefix('REALM_PROGRAM ').rsplit(' ', 1)
                                            for line in result.stdout.splitlines() if line.startswith('REALM_PROGRAM '))
    sidecar.write_text(json.dumps(installed), encoding='utf-8')
