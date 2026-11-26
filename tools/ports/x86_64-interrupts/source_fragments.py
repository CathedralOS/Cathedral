"""Append an Omega source fragment within the importing file's scope."""


def append_fragment(source, fragment, local_alias=None):
    imports = {line for line in source.splitlines() if line.startswith('use ')}
    if local_alias is not None:
        imports.add(f'use {local_alias}.omg;')
        fragment = fragment.replace(f'|{local_alias}|::', '')
    lines = (line for line in fragment.splitlines(keepends=True) if line.rstrip('\n') not in imports)
    return source + '\n' + ''.join(lines)
