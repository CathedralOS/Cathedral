#!/usr/bin/env python3
"""Verify the selected nominal-owner migration, retaining old full receipts as history."""
import hashlib
import json
from pathlib import Path
import check
import fixtures
import vectors
NAMES=['bank_read_0_0','index_write_3_0','index_last_selector_overflow']
def main():
 record=json.loads((fixtures.HERE/'owner-migration-verification.json').read_text())
 assert record['omega_revision']==check.PIN and record['scope']=='selected'
 assert record['input_sha256']==check.snapshot()
 expected=check.build_text().replace(str(fixtures.ROOT),record['execution_root'])
 assert record['build_text']==expected and record['build_sha256']==hashlib.sha256(expected.encode()).hexdigest()
 assert record['runner_sha256']==check.sha(Path(record['runner_path']))
 rows=[row for row in vectors.cases()if row['name']in NAMES]
 assert len(rows)==3 and record['cases']==[row['name']for row in rows]
 assert record['positive_count']==record['control_count']==3
 consumed=[]
 for batch in record['batches']:
  selected=[row for row in rows if row['name']in batch['cases']]
  text,names=fixtures.render(selected)
  assert batch['cases']==[row['name']for row in selected]
  assert batch['source_sha256']==hashlib.sha256(text.encode()).hexdigest();check.validate(batch['output'],names);consumed+=selected
 assert consumed==rows
 print('PASS three retained checked owner-migration pairs; original 204/3 receipts remain historical.')
if __name__=='__main__':main()
