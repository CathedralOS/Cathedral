#!/usr/bin/env python3
"""Classify actual public pin observations against the explicit ASL text profile."""
import collections,json
from pathlib import Path
import fixtures,reference
HERE=fixtures.HERE
def report():
 public=json.loads((HERE/'reference-verification.json').read_text());rows=[];counts=collections.Counter()
 for row in public['observations']:
  raw=bytes.fromhex(row['text_hex']);expected=fixtures.expectation(raw);parsed=row['observed'].get('parsed','panic');pin_ok=parsed.startswith('ok:');strict_ok='error'not in expected
  if pin_ok and strict_ok:
   assert parsed[3:]==expected['text_hex'],(raw,parsed,expected)
   category='accepted_text_agreement'
  elif not pin_ok and not strict_ok:category='both_reject'
  elif strict_ok:category='primary_acceptance_correction'
  else:category='strict_grammar_or_capacity_rejection'
  counts[category]+=1;rows.append(dict(kind=row['kind'],text_hex=row['text_hex'],category=category,pin_parsed=parsed,strict_expectation=expected))
 # Explicit public canaries prevent a self-consistent report from hiding pin drift.
 path={bytes.fromhex(r['text_hex']):r['observed']for r in public['observations']if r['kind']=='path'}
 assert path[b'\\']['parsed']=='ok:5c'
 assert not path[b'^'].get('parsed','panic').startswith('ok:')
 assert path[b'A.^B']['parsed'].startswith('ok:')
 assert not path[b'abcd']['parsed'].startswith('ok:')
 assert 'parsed'not in path[b'a'] and path[b'a']['panic']=='true'
 assert path[b'A.B']['parsed']=='ok:415f5f5f2e425f5f5f'
 return dict(stage='classification of actual public Rust observations; strict expected values are exercised separately by Omega fixtures',counts=dict(counts),rows=rows)
def main():
 import argparse
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();data=report();path=HERE/'comparison.json'
 if a.write:path.write_text(json.dumps(data,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==data
 print('PASS',len(data['rows']),'classified actual public observations',data['counts'])
if __name__=='__main__':main()
