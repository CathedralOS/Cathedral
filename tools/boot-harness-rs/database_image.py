"""Persistent database desktop media. Only a missing image may be formatted;
the profile's programs are installed into it as objects, and reinstalled as
new revisions whenever their bytes change. What was installed is recorded in
a sidecar beside the image, so an unchanged set never reopens the image."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

VOLUME_BYTES = 4 * 1024 * 1024
OFFSET = 4096
IMAGE_BYTES = OFFSET + VOLUME_BYTES + 4096


def oracle():
    """Build the host storage lab once and return its executable."""
    root = Path(__file__).resolve().parents[2]
    lab = root / 'tools/storage-lab'
    subprocess.run(['cargo', 'build', '--locked', '--release', '--features', 'engines'], cwd=lab, check=True)
    return lab / 'target/release' / ('storage-layout.exe' if os.name == 'nt' else 'storage-layout')


def prepare(path, programs=(), label_migration=None, app_close=False):
    """Create the image if missing, then install `programs`: (parent, file) pairs
    that become objects 1.. of root 0 in order, each under an earlier object or
    the root. An existing image keeps its data; only changed programs are
    republished. The stock desktop may also request its explicit readable-label
    migration, which upgrades recognized copied launchers and labels old apps.
    The migration saves a backup and leaves customized code and user data alone.
    app_close additionally refreshes recognized stock Notes/Colors code."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise RuntimeError(f'Database image must be a regular file: {path}')
    executable = None
    sidecar = path.with_name(path.name + '.programs')
    if path.exists():
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
        installed = {'programs': installed, 'readable_labels': hashlib.sha256(Path(label_migration).read_bytes()).hexdigest(), 'app_close': app_close}
    if sidecar.exists() and json.loads(sidecar.read_text(encoding='utf-8')) == installed:
        return
    if executable is None:
        executable = oracle()
    lab = executable.parents[2]
    command = [str(executable), 'engines', 'volume', 'seed', str(path), str(OFFSET), str(VOLUME_BYTES)]
    for parent, file in programs:
        command += [str(parent), str(file)]
    subprocess.run(command, cwd=lab, check=True)
    if label_migration is not None:
        subprocess.run([str(executable), 'engines', 'volume', 'upgrade-app-close' if app_close else 'upgrade-labels', str(path),
                        str(OFFSET), str(VOLUME_BYTES), str(Path(label_migration).resolve())], cwd=lab, check=True)
    sidecar.write_text(json.dumps(installed), encoding='utf-8')
