#!/usr/bin/env python3
"""Verify unchanged Mid inputs after unrelated package additions; no execution replay."""
import argparse, hashlib, json, re, subprocess, sys, tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
CORPUS=HERE.parent
ROOT=CORPUS.parents[4]

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record',type=Path)
    args=parser.parse_args()
    receipt=CORPUS/'checked-verification.json';record=json.loads(receipt.read_text())
    recorded=record['input_sha256']
    # Every originally captured input remains byte-identical, even outside the
    # actual dependency packages. This cannot excuse a changed recorded file.
    for path,wanted in recorded.items():assert sha(ROOT/path)==wanted,path
    pending=[ROOT/'source/libraries/acpi'/part for part in ['aml','interpreter','interpreter/execution','pipeline']]
    roots=set()
    while pending:
        root=pending.pop().resolve()
        if root in roots:continue
        assert root.is_relative_to(ROOT.resolve()),root
        roots.add(root);build=(root/'build.omg').read_text()
        # These recorded build files have only literal Source::Path dependencies.
        locations=re.findall(r'Source::Path\s*\{\s*location\s*:\s*"([^"]+)"\s*\}',build)
        assert len(locations)==len(re.findall(r'builder\.depend(?:_as)?\(',build)),root
        pending.extend((root/path).resolve() for path in locations)
    closure={str(path.relative_to(ROOT)):sha(path) for root in sorted(roots) for path in sorted(root.glob('*.omg'))}
    assert all(recorded.get(path)==digest for path,digest in closure.items())
    current={str(p.relative_to(ROOT)) for p in (ROOT/'source/libraries/acpi').rglob('*.omg')}
    added={path:sha(ROOT/path) for path in sorted(current-set(recorded))}
    assert not set(added)&set(closure)
    # Run the original, hash-verified verifier with precisely its original input
    # set, so its whole-tree snapshot cannot be silently relaxed. This performs
    # source/generator/entry/result/binary verification, not checked interpretation.
    with tempfile.TemporaryDirectory(prefix='cathedral-mid-receipt-inputs-') as directory:
        temporary=Path(directory)
        for path in recorded:
            target=temporary/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/path).read_bytes())
        verifier=temporary/'tools/ports/acpi/interpreter/mid-execution/verify_record.py'
        run=subprocess.run([sys.executable,str(verifier),str(receipt),'--require-binaries'],capture_output=True,text=True,check=True)
    result=dict(stage='unchanged dependency closure and original receipt verification; no execution replay',receipt_sha256=sha(receipt),original_input_count=len(recorded),package_roots=[str(p.relative_to(ROOT)) for p in sorted(roots)],dependency_files_sha256=closure,unrelated_added_files_sha256=added,original_verifier_output=run.stdout)
    if args.record:args.record.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(run.stdout,end='');print('PASS',len(closure),'unchanged dependency files;',len(added),'unrelated additions; no execution replay')
if __name__=='__main__':main()
