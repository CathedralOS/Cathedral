# Retained rust-osdev license records

See [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) for the preserved
license choices, attributions, modification policy, and pin-update audit.

`sources.json` identifies exact upstream commits and file paths. Entries under
`copied_files` are verbatim copies; `evidence` entries hash inspected files kept
only in the gitignored reading room. pic8259's `supplied_files` explicitly
identify the other pinned project from which standard license terms were copied,
because pic8259 itself links the terms without carrying text files. The two
Markdown notice files are Cathedral collections of the attributed upstream
excerpts, not claims that those upstreams contain a NOTICE file.

To check source evidence and exact retained bytes from the repository root,
with all seven pinned reading-room checkouts present:

```sh
python3 - <<'PY'
import hashlib
import json
from pathlib import Path
import subprocess

records = json.loads(Path('licenses/rust-osdev/sources.json').read_text())
for project in records['projects']:
    checkout = Path('reference_code/rust-osdev') / project['project']
    if not checkout.is_dir():
        raise SystemExit(f'Missing optional upstream checkout: {checkout}')
    for record in project['evidence'] + project['copied_files']:
        data = subprocess.check_output([
            'git', '-C', str(checkout), 'show',
            project['revision'] + ':' + record['upstream_path'],
        ])
        assert hashlib.sha256(data).hexdigest() == record['sha256'], record
        if 'local_path' in record:
            assert Path(record['local_path']).read_bytes() == data, record
    for record in project.get('supplied_files', []):
        data = subprocess.check_output([
            'git', '-C', str(Path('reference_code/rust-osdev') / record['source_project']),
            'show', record['source_revision'] + ':' + record['upstream_path'],
        ])
        assert hashlib.sha256(data).hexdigest() == record['sha256'], record
        assert Path(record['local_path']).read_bytes() == data, record
print('Verified source evidence and retained license bytes for 7 pinned projects')
PY
```

This checks recorded provenance and bytes, not the licensing or completeness of
future source translations. Review each slice's own inputs and source map.
