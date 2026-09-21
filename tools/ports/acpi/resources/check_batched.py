#!/usr/bin/env python3
"""Reproduce the full checked-interpreter corpus in disjoint bounded suites."""
import argparse,concurrent.futures,hashlib,json,subprocess,sys,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--jobs',type=int,default=2);p.add_argument('--batch-size',type=int,default=40);p.add_argument('--record',type=Path,default=HERE/'checked-verification.json');a=p.parse_args()
 assert 1<=a.jobs<=3 and a.batch_size>=1
 cases=json.loads((HERE/'cases.json').read_text());names=sorted(cases);groups=[names[i:i+a.batch_size]for i in range(0,len(names),a.batch_size)];directory=HERE/'checked-batches';directory.mkdir(exist_ok=True)
 metadata_before=digest(HERE/'cases.json');started=time.monotonic()
 def run(item):
  i,selected=item;path=directory/f'batch-{i:02}.json';log=directory/f'batch-{i:02}.log';command=[sys.executable,str(HERE/'check_interpreted.py'),*sum([['--case',n]for n in selected],[]),'--record',str(path)]
  with log.open('w')as output:subprocess.run(command,cwd=ROOT,stdout=output,stderr=subprocess.STDOUT,check=True)
  record=json.loads(path.read_text());assert set(record['fixture_sha256'])==set(selected);assert record['scenario_count']==len(selected)==record['control_count']
  print(f'PASS batch {i}: {len(selected)} body/control pairs',flush=True);return path,record
 with concurrent.futures.ThreadPoolExecutor(max_workers=a.jobs)as pool:records=list(pool.map(run,enumerate(groups)))
 assert metadata_before==digest(HERE/'cases.json'),'metadata changed'
 union={};source=None
 for path,r in records:
  if source is None:source=r['source_sha256']
  assert source==r['source_sha256'],'batch source mismatch'
  for n,h in r['fixture_sha256'].items():assert n not in union;assert h==digest(HERE/'cases'/f'{n}.omg');union[n]=h
 assert set(union)==set(cases)
 for path,h in source.items():assert digest(ROOT/path)==h
 result={'format':'cathedral-resource-checked-batches-v1','stage':'actual checked-interpreter execution of every positive and changed-body control; no native ABI','scenario_count':len(union),'control_count':len(union),'source_sha256':source,'fixture_sha256':union,'omega_revision':records[0][1]['omega_revision'],'runner_sha256':records[0][1]['runner_sha256'],'batches':[{ 'path':str(path.relative_to(ROOT)), 'sha256':digest(path),'cases':r['scenario_count']}for path,r in records],'elapsed_seconds':round(time.monotonic()-started,3),'command':f'python3 tools/ports/acpi/resources/check_batched.py --jobs {a.jobs} --batch-size {a.batch_size}'}
 a.record.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');print(f'PASS complete union: {len(union)} positives + {len(union)} controls.',flush=True)
if __name__=='__main__':main()
