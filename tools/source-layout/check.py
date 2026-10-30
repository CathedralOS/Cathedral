#!/usr/bin/env python3
"""Check Cathedral's source ownership boundaries and relocated build roots."""

import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]
LAYERS = ('kernel', 'platform', 'distribution', 'contracts', 'foundation')


def within(path, root):
    return path == root or path.startswith(root + '/')


def check_edge(source, target):
    """Paths name package roots; public contracts are not implementation imports."""
    owner, dependency = source.split('/')[0], target.split('/')[0]
    if owner not in LAYERS or dependency not in LAYERS:
        raise ValueError(f'Unknown source layer: {source} -> {target}')

    # Independent proof packages may compose their own native/Omega pieces,
    # never import the stock distribution's policy or contracts.
    independent = within(source, 'distribution/lab/matrix-composition')
    split = within(source, 'distribution/applications/split')
    same_proof = independent and within(target, 'distribution/lab/matrix-composition')
    if (independent or split) and dependency == 'distribution' and not same_proof:
        raise ValueError(f'Independent matrix proof imports distribution code: {source} -> {target}')

    kernel_contract = within(source, 'kernel/contracts')
    platform_contract = within(source, 'platform/contracts')
    distribution_contract = within(source, 'distribution/matrix/contracts')
    block_boundary = source == 'platform/libraries/block-device'
    if kernel_contract or platform_contract or distribution_contract or block_boundary:
        allowed = dependency == 'foundation'
        allowed |= kernel_contract and within(target, 'kernel/contracts')
        allowed |= platform_contract and (within(target, 'platform/contracts') or target == 'kernel/contracts')
        allowed |= distribution_contract and (within(target, 'platform/contracts') or target == 'kernel/contracts')
        if not allowed:
            raise ValueError(f'Public boundary imports implementation: {source} -> {target}')

    if dependency == 'distribution' and owner != 'distribution':
        raise ValueError(f'Reverse distribution dependency: {source} -> {target}')
    if owner in ('platform', 'distribution') and dependency == 'kernel' and target != 'kernel/contracts':
        raise ValueError(f'Userspace imports kernel implementation: {source} -> {target}')
    # Omega's existing external UEFI declarations still occupy its legacy root.
    if owner in ('contracts', 'foundation') and dependency not in ('contracts', 'foundation'):
        raise ValueError(f'Shared root imports implementation: {source} -> {target}')
    if owner == 'kernel' and dependency == 'platform':
        facts = target == 'platform/drivers/facts'
        boot_console = source == 'kernel/boot/uefi' and target == 'platform/drivers/uart_16550'
        if not (facts or boot_console):
            raise ValueError(f'Kernel imports platform implementation: {source} -> {target}')


def check_profiles():
    for tree_name in ('source', 'source-rs'):
        tree = ROOT / tree_name
        for old in ('boot', 'core', 'arch', 'drivers', 'libraries', 'services', 'applications', 'distributions'):
            if (tree / old).exists():
                raise ValueError(f'Obsolete source root: {tree_name}/{old}')
        profile = json.loads((tree / 'distribution/profile.json').read_text(encoding='utf-8'))
        entry = (tree / profile['boot_entry']).resolve()
        if not entry.is_relative_to(tree / 'kernel/boot') or not entry.is_file():
            raise ValueError(f'Invalid boot entry in {tree_name}/distribution/profile.json')
        if not profile['name'] or not profile['target']:
            raise ValueError(f'Incomplete distribution profile: {tree_name}')


def check_omega():
    tree = ROOT / 'source'
    count = 0
    for layer in LAYERS:
        for manifest in (tree / layer).rglob('build.omg'):
            text = re.sub(r'//[^\n]*', '', manifest.read_text(encoding='utf-8'))
            paths = re.findall(r'\b(?:path\(|location:\s*)"([^"]+)"', text)
            for relative in paths:
                target = (manifest.parent / relative).resolve()
                if not target.is_dir() or not target.is_relative_to(tree):
                    raise ValueError(f'Invalid Omega dependency: {manifest}: {relative}')
                check_edge(manifest.parent.relative_to(tree).as_posix(), target.relative_to(tree).as_posix())
                count += 1
    return count


def check_rust():
    tree = ROOT / 'source-rs'
    if (tree / 'contracts').exists():
        raise ValueError('Rust contracts must live with their kernel/platform owner')
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], cwd=tree))
    packages = {p['name']: p for p in metadata['packages']}
    count = 0
    for package in packages.values():
        source = Path(package['manifest_path']).parent.relative_to(tree).as_posix()
        for dependency in package['dependencies']:
            if dependency.get('path'):
                target = Path(dependency['path']).relative_to(tree).as_posix()
                check_edge(source, target)
                count += 1
    profile = json.loads((tree / 'distribution/profile.json').read_text(encoding='utf-8'))
    package = packages[profile['boot_package']]
    entries = [Path(t['src_path']).resolve() for t in package['targets'] if 'bin' in t['kind']]
    if (tree / profile['boot_entry']).resolve() not in entries:
        raise ValueError('Rust profile boot entry does not match its Cargo package')
    startup = profile['startup']
    users = [*profile['user_programs'].values(), startup['initial']]
    users.extend(startup.get('launches', [startup['launch']] if startup.get('launch') else []))
    users.extend(startup.get('executables', []))
    # Realm entries without a package are installer directives, not executables.
    users.extend(entry for entry in startup.get('realm_programs', []) if 'package' in entry)
    users.extend(program for program in startup.get('programs', []) if 'package' in program)
    for path in (ROOT / 'tools/boot-harness-rs/profiles').glob('*.json'):
        lab = json.loads(path.read_text(encoding='utf-8'))['startup']
        users.append(lab['initial'])
        users.extend(lab.get('launches', [lab['launch']] if lab.get('launch') else []))
        users.extend(lab.get('executables', []))
        users.extend(program for program in lab.get('programs', []) if 'package' in program)
    for user in users:
        if Path(user['entry']).suffix == '.omg':
            entry = (ROOT / 'source' / user['entry']).resolve()
            if not any(entry.is_relative_to(ROOT / 'source' / owner)
                       for owner in ('distribution', 'platform/services')) or entry.name != 'main.omg' \
                    or not entry.is_file() or not entry.with_name('build.omg').is_file():
                raise ValueError('Omega profile user entry must name a distribution or platform service package')
            if user['target'] != 'x86_64-unknown-none':
                raise ValueError('standalone Omega profile requires the implemented x86-64 ELF target')
            continue
        entry = (tree / user['entry']).resolve()
        package = packages[user['package']]
        entries = [Path(t['src_path']).resolve() for t in package['targets'] if 'bin' in t['kind']]
        if not (entry.is_relative_to(tree / 'distribution') or entry.is_relative_to(tree / 'platform/services')) or entry not in entries:
            raise ValueError('Rust profile user entry must name a distribution or platform service executable')
        if user['target'] != 'x86_64-unknown-none':
            raise ValueError('Rust user profile requires the implemented x86-64 ELF target')
    return count


def main():
    check_profiles()
    omega = check_omega()
    rust = check_rust()
    print(f'PASS: both distribution profiles; {omega} Omega and {rust} Rust package edges')


if __name__ == '__main__':
    main()
