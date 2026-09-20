#!/usr/bin/env python3
"""Pinned-source inventories for staged Omega ports; not a Rust semantic parser."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

REPOSITORY = Path(__file__).resolve().parents[2]
DISPOSITIONS = {"pending", "translated", "omitted", "blocked"}


def read_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate JSON key: {key}")
            result[key] = value
        return result
    return json.loads(Path(path).read_text(), object_pairs_hook=unique)


def relative_path(value):
    path = PurePosixPath(value)
    if not value or path.is_absolute() or '..' in path.parts or '\\' in value or str(path) != value:
        raise ValueError(f"expected normalized relative path: {value!r}")
    return path


def git(checkout, *arguments):
    return subprocess.check_output(['git', '-C', str(checkout), *arguments], stderr=subprocess.PIPE).decode()


def rust_anchors(source):
    """Conservative source review anchors, including macro flags/enum values.

    Entire file hashes also bind syntax this lexical index does not understand.
    Anchors deliberately include private operations: review may omit them with
    reasons, but must not silently discard a method when visibility is inherited.
    This does not expand macros, resolve reexports, or prove semantic coverage.
    """
    anchors = {}
    block_depth = 0
    for number, line in enumerate(source.splitlines(), 1):
        clean = ''
        index = 0
        while index < len(line):
            pair = line[index:index + 2]
            if pair == '/*':
                block_depth += 1
                index += 2
            elif block_depth and pair == '*/':
                block_depth -= 1
                index += 2
            elif block_depth:
                index += 1
            elif pair == '//':
                break
            else:
                clean += line[index]
                index += 1
        clean = clean.strip()
        declaration = re.search(r'\b(?:fn|struct|enum|union|trait|type|const(?!\s+fn\b)|static|mod)\s+([A-Za-z_][A-Za-z_0-9]*)', clean)
        field = re.match(r'pub(?:\([^)]*\))?\s+([A-Za-z_][A-Za-z_0-9]*)\s*:', clean)
        variant = re.match(r'([A-Z][A-Za-z_0-9]*)\s*=', clean)
        exported = re.match(r'pub\s+use\s+(.+)', clean)
        matched = declaration or field or variant or exported
        if matched:
            anchors[f'{number}:{matched.group(1)}'] = clean
    return anchors


def sources(checkout, revision, roots):
    if not checkout.is_dir() or not (checkout / '.git').exists():
        raise ValueError(f"upstream checkout absent: {checkout}; acquire the exact pinned source first")
    if not re.fullmatch('[0-9a-f]{40}', revision):
        raise ValueError('revision must be a full lowercase Git commit')
    if git(checkout, 'rev-parse', 'HEAD').strip() != revision:
        raise ValueError(f'upstream HEAD differs from pinned revision {revision}')
    for root in roots:
        relative_path(root)
    names = git(checkout, 'ls-tree', '-r', '--name-only', revision, '--', *roots).splitlines()
    if not names:
        raise ValueError('claimed source roots contain no tracked files')
    result = {}
    for name in names:
        if not name.endswith('.rs'):
            continue
        blob = subprocess.check_output(['git', '-C', str(checkout), 'show', f'{revision}:{name}'])
        if not (checkout / name).is_file() or (checkout / name).read_bytes() != blob:
            raise ValueError(f'upstream working source differs from pin: {name}')
        result[name] = (hashlib.sha256(blob).hexdigest(), rust_anchors(blob.decode()))
    if not result:
        raise ValueError('claimed source roots contain no Rust source')
    return result


def snapshot(checkout, revision, roots, url):
    result = {'format': 'cathedral-port-inventory-v1', 'upstream': {'url': url, 'revision': revision, 'roots': roots}, 'files': {}}
    for name, (digest, anchors) in sources(checkout, revision, roots).items():
        result['files'][name] = {'sha256': digest, 'disposition': 'pending', 'reason': 'Awaiting source review.', 'symbols': {
            key: {'anchor': anchor, 'disposition': 'pending', 'reason': 'Awaiting source review.'} for key, anchor in anchors.items()}}
    return result


def check_entry(entry, label, repository, require_transcribed):
    disposition = entry['disposition']
    if disposition not in DISPOSITIONS:
        raise ValueError(f'{label}: unknown disposition {disposition}')
    if disposition != 'translated' and not entry.get('reason', '').strip():
        raise ValueError(f'{label}: {disposition} needs a reason')
    if require_transcribed and disposition in {'pending', 'blocked'}:
        raise ValueError(f'{label}: not transcribed ({disposition})')
    targets = entry.get('targets', [])
    if disposition == 'translated' and not targets:
        raise ValueError(f'{label}: translated entry needs target mappings')
    for target in targets:
        path = repository / relative_path(target['path'])
        if not path.resolve().is_relative_to(repository.resolve()) or not path.is_file():
            raise ValueError(f'{label}: target missing or outside repository: {path}')
        anchor = target.get('anchor', '')
        if not anchor.strip() or anchor not in path.read_text():
            raise ValueError(f'{label}: target anchor missing: {path}: {anchor!r}')


def check(manifest, checkout, repository=REPOSITORY, require_transcribed=False):
    if manifest['format'] != 'cathedral-port-inventory-v1':
        raise ValueError('unsupported inventory format')
    upstream = manifest['upstream']
    actual = sources(checkout, upstream['revision'], upstream['roots'])
    if set(actual) != set(manifest['files']):
        raise ValueError(f"source file inventory differs: {sorted(set(actual) ^ set(manifest['files']))}")
    counts = {status: 0 for status in sorted(DISPOSITIONS)}
    for name, (digest, anchors) in actual.items():
        entry = manifest['files'][name]
        if entry['sha256'] != digest:
            raise ValueError(f'{name}: source hash differs')
        check_entry(entry, name, repository, require_transcribed)
        if set(entry['symbols']) != set(anchors):
            raise ValueError(f'{name}: symbol inventory differs: {sorted(set(entry["symbols"]) ^ set(anchors))}')
        for key, anchor in anchors.items():
            symbol = entry['symbols'][key]
            if symbol['anchor'] != anchor:
                raise ValueError(f'{name}:{key}: source anchor differs')
            check_entry(symbol, f'{name}:{key}', repository, require_transcribed)
            counts[symbol['disposition']] += 1
    return {'files': len(actual), 'symbols': counts, 'meaning': 'source coverage audit only; no Omega compilation or execution'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    create = commands.add_parser('snapshot', help='write an unreviewed inventory to stdout')
    create.add_argument('--checkout', type=Path, required=True)
    create.add_argument('--revision', required=True)
    create.add_argument('--url', required=True)
    create.add_argument('roots', nargs='+')
    verify = commands.add_parser('check')
    verify.add_argument('manifest', type=Path)
    verify.add_argument('--checkout', type=Path, required=True)
    verify.add_argument('--require-transcribed', action='store_true')
    arguments = parser.parse_args()
    try:
        if arguments.command == 'snapshot':
            result = snapshot(arguments.checkout, arguments.revision, arguments.roots, arguments.url)
        else:
            result = check(read_json(arguments.manifest), arguments.checkout, require_transcribed=arguments.require_transcribed)
        print(json.dumps(result, indent=2, sort_keys=True))
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'error: {error}\n')


if __name__ == '__main__':
    main()
