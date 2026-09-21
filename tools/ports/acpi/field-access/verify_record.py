#!/usr/bin/env python3
"""Validate current production/harness hashes and actual positive/control receipts."""
import argparse,hashlib,json
from pathlib import Path
import check,check_const,fixtures,reference,compare
HERE=fixtures.HERE

def main():
 p=argparse.ArgumentParser();p.add_argument('--checked-only',action='store_true');p.add_argument('--record',default='checked-verification.json');a=p.parse_args()
 record=json.loads((HERE/a.record).read_text());assert record['input_sha256']==check.snapshot();assert record['omega_revision']==check.PIN
 rows=[r for r in fixtures.cases()if r['name']in record['cases']];assert [r['name']for r in rows]==record['cases']
 if record['scope']=='full':assert rows==fixtures.cases()
 assert record['positive_count']==record['control_count']==len(rows)
 assert record['runner_sha256']==check.sha(Path(record['runner_path']))
 batch_cases=[]
 for batch in record['batches']:
  selected=[r for r in rows if r['name']in batch['cases']];assert [r['name']for r in selected]==batch['cases'];batch_cases+=selected
  text,names=fixtures.render(selected);assert batch['source_sha256']==hashlib.sha256(text.encode()).hexdigest();check.validate(batch['output'],names)
 assert batch_cases==rows
 assert json.loads((HERE/'comparison.json').read_text())==compare.report()
 public=json.loads((HERE/'reference-verification.json').read_text());assert public['pin']==reference.PIN
 assert public['source_sha256']=={name:check.sha(HERE/name)for name in public['source_sha256']}
 assert [{key:r[key]for key in reference.cases()[0]}for r in public['rows']]==reference.cases()
 if not a.checked_only:
  const=json.loads((HERE/'const-verification.json').read_text());assert const['input_sha256']==check.snapshot();assert const['omega_revision']==check.PIN
  assert const['positive_count']==const['control_count']==len(check_const.selected());assert len(const['proofs'])==2*len(check_const.selected());assert const['compiler_sha256']==check.sha(Path(const['compiler_path']))
  for row,proofs in zip(check_const.selected(),zip(const['proofs'][::2],const['proofs'][1::2])):
   for control,proof in zip([False,True],proofs):
    assert proof['case']==row['name'] and proof['control']==control
    assert proof['source_sha256']==hashlib.sha256(check_const.source(row,control).encode()).hexdigest()
    assert ('cannot prove requires contract'in proof['output'] and '1 == 0'in proof['output'])if control else 'compiled 'in proof['output']
 print('PASS',len(rows),'checked pairs;',len(record['input_sha256']),'current hashes;',len(public['rows']),'bound public observations;','checked only'if a.checked_only else'constant pairs verified')
if __name__=='__main__':main()
