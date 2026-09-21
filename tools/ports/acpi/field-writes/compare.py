#!/usr/bin/env python3
"""Compare actual public one-pass writes with detached records and exact callback logs."""
import argparse,collections,json
import reference
HERE=reference.HERE

def pin_expected(row):
 memory=bytearray((i*37+11)&255 for i in range(512));accesses=[];expected=row['strict_expected'];update=(row['flags']>>5)&3
 assert 'error'not in expected
 for record in expected['records']:
  offset=record['offset'];width=record['width'];left=max(row['offset'],offset*8);right=min(row['offset']+row['length'],(offset+width)*8)
  if right-left<width*8 and update==0:accesses.append(f'read,{offset},{width},{int.from_bytes(memory[offset:offset+width],"little")}')
  value=record['value'];accesses.append(f'write,{offset},{width},{value}');memory[offset:offset+width]=value.to_bytes(width,'little')
 return dict(load='ok',result='integer:0',accesses=';'.join(accesses),memory=memory.hex(),created_mutexes='1',forbidden_calls='0')

def report():
 public=json.loads((HERE/'reference-verification.json').read_text());assert public['pin']==reference.PIN;cases=reference.cases();assert len(public['rows'])==len(cases);counts=collections.Counter();rows=[]
 for row,case in zip(public['rows'],cases):
  assert all(row[k]==v for k,v in case.items());assert row['observed']==pin_expected(case),case['name']
  category='integer_single_pass'if isinstance(case['source'],int)else'buffer_exact_byte_aligned_single_pass';counts[category]+=1;rows.append(dict(name=case['name'],category=category))
 return dict(stage='actual public Store-to-Field callbacks compared to detached one-pass records; no conversion or hardware authority claim',public_count=len(rows),counts=dict(sorted(counts.items())),rows=rows)

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();result=report();path=HERE/'comparison.json'
 if a.write:path.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==result
 print('PASS',result['public_count'],'public write comparisons',result['counts'])
if __name__=='__main__':main()
