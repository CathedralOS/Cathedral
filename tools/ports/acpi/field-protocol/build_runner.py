#!/usr/bin/env python3
"""Build the retained checked runner and Omega CLI from an explicit audited checkout.

Requires Python 3, Git, and the checkout's pinned Rust toolchain on PATH.
Retain the Omega checkout after building: its bundled library is used at runtime.
"""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SHARED = HERE.parent / 'interpreter' / 'execution'
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def output(command, cwd):
    return subprocess.check_output(command, cwd=cwd, text=True).strip()


def require_pin(omega):
    revision = output(['git', 'rev-parse', 'HEAD'], omega)
    if revision != PIN:
        raise RuntimeError(f'Omega must be at audited revision {PIN}; found {revision}')
    if output(['git', 'status', '--porcelain', '--untracked-files=all'], omega):
        raise RuntimeError('Omega source must be clean')


def manifest(omega):
    lines = ['[package]', 'name="cathedral-acpi-checked-runner"',
             'version="0.1.0"', 'edition="2024"', '[dependencies]']
    for name, relative in {
        'checked-interpreter': 'psi/semantics/checked-interpreter',
        'package-manager': 'omega/packages/manager',
        'target': 'omega/representations/target',
    }.items():
        path = json.dumps((omega / 'omega-rust' / relative).as_posix())
        lines.append(f'{name} = {{ path = {path} }}')
    source = json.dumps((SHARED / 'checked_runner.rs').as_posix())
    lines += ['[[bin]]', 'name="cathedral-acpi-checked-runner"', f'path={source}']
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega-source', type=Path, required=True,
                        help='existing clean checkout at the audited Omega revision')
    parser.add_argument('--target-dir', type=Path, required=True,
                        help='dedicated build directory outside both source checkouts')
    parser.add_argument('--cargo', default=shutil.which('mbx') or 'cargo',
                        help='Cargo-compatible executable; defaults to mbx if found, otherwise cargo')
    parser.add_argument('--fetch', action='store_true',
                        help='allow dependency downloads; builds remain --locked')
    parser.add_argument('--record', type=Path,
                        help='write build provenance here; no record is written by default')
    args = parser.parse_args()
    omega, target = args.omega_source.resolve(), args.target_dir.resolve()
    for checkout in (omega, ROOT):
        if target == checkout or checkout in target.parents:
            parser.error('--target-dir must be outside the Omega and Cathedral checkouts')
    record_path = args.record.resolve() if args.record else None
    if record_path and (record_path == omega or omega in record_path.parents):
        parser.error('--record must be outside the Omega source checkout')
    cargo = shutil.which(args.cargo)
    if not cargo:
        parser.error(f'build executable not found: {args.cargo}')
    require_pin(omega)
    suffix = '.exe' if os.name == 'nt' else ''
    rustc = os.environ.get('RUSTC') or 'rustc'
    sibling_rustc = Path(cargo).parent / ('rustc' + suffix)
    if 'RUSTC' not in os.environ and sibling_rustc.is_file():
        rustc = str(sibling_rustc)
    rustc_version = output([rustc, '-Vv'], omega)
    environment = dict(os.environ, RUSTC=rustc)
    build_tool_version = output([cargo, '--version'], omega)
    paths = [Path(__file__).resolve(), SHARED / 'checked_runner.rs',
             SHARED / 'runner.Cargo.lock', omega / 'Cargo.lock',
             omega / 'rust-toolchain.toml']
    inputs = {str(path): sha(path) for path in paths}
    common = [cargo, 'build', '--locked', '--release', '--jobs', '4',
              '--target-dir', str(target)]
    if not args.fetch:
        common.append('--offline')
    manifest_text = manifest(omega)
    with tempfile.TemporaryDirectory(prefix='cathedral-field-protocol-runner-') as directory:
        work = Path(directory)
        (work / 'Cargo.toml').write_text(manifest_text, encoding='utf-8')
        shutil.copyfile(SHARED / 'runner.Cargo.lock', work / 'Cargo.lock')
        commands = [common + ['--manifest-path', str(work / 'Cargo.toml')],
                    common + ['--manifest-path', str(omega / 'Cargo.toml'), '-p', 'omega']]
        for command in commands:
            require_pin(omega)
            print(json.dumps(command), flush=True)
            subprocess.run(command, cwd=omega, env=environment, check=True)
        if sha(work / 'Cargo.lock') != inputs[str(SHARED / 'runner.Cargo.lock')]:
            raise RuntimeError('Runner lockfile changed during build')
    require_pin(omega)
    if inputs != {str(path): sha(path) for path in paths}:
        raise RuntimeError('Build inputs changed during compilation')
    artifacts = {
        name: target / 'release' / (name + suffix)
        for name in ('cathedral-acpi-checked-runner', 'omega')
    }
    hashes = {name: {'path': str(path), 'sha256': sha(path)}
              for name, path in artifacts.items()}
    record = {
        'format': 'cathedral-field-protocol-toolchain-build-v1',
        'omega_revision': PIN, 'omega_source': str(omega), 'omega_clean': True,
        'target_dir': str(target), 'rustc': rustc_version, 'rustc_executable': rustc,
        'build_tool': cargo, 'build_tool_version': build_tool_version,
        'offline': not args.fetch, 'commands': commands,
        'runner_manifest_contents': manifest_text, 'input_sha256': inputs,
        'artifacts': hashes,
        'scope': 'build only; no field-protocol tests or native/hardware execution',
    }
    if record_path:
        record_path.parent.mkdir(parents=True, exist_ok=True)
        record_path.write_text(json.dumps(record, indent=2, sort_keys=True) + '\n',
                               encoding='utf-8')
        print('Build provenance:', record_path)
    for name, artifact in hashes.items():
        print(f'{name}: {artifact["path"]}\nSHA-256: {artifact["sha256"]}')
    print('Retain the Omega source checkout for its bundled runtime library.')


if __name__ == '__main__':
    try:
        main()
    except subprocess.CalledProcessError as error:
        raise SystemExit(error.returncode) from error
    except (OSError, RuntimeError) as error:
        raise SystemExit(str(error)) from error
