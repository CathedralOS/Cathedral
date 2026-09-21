#!/usr/bin/env python3
"""Exact pinned callback observations; primary repeated semantics are separate expectations."""
import argparse,collections,json
import reference,geometry_vectors
HERE=reference.HERE

def pin_expected(row):
 memory=bytearray((i*37+11)&255 for i in range(512));output=dict(load='ok',created_mutexes='1',forbidden_calls='0');source=row['source']
 if isinstance(source,dict)and'text'in source:return dict(output,result='error:ObjectNotOfExpectedType { expected: Integer, got: String }',accesses='',memory=memory.hex())
 raw=source.to_bytes(8,'little')if isinstance(source,int)else bytes.fromhex(source['buffer']);logical=int.from_bytes(raw,'little');plan=geometry_vectors.plan(row['offset'],row['length'],row['flags'],row['region']);accesses=[]
 for chunk in plan['chunks']:
  offset=chunk['offset'];width=chunk['width'];old=int.from_bytes(memory[offset:offset+width],'little')
  if chunk['partial']:accesses.append(f'read,{offset},{width},{old}')
  value=(logical>>chunk['field_bit'])&((1<<chunk['bit_count'])-1);word=((old&~chunk['mask'])|(value<<chunk['native_bit']))&((1<<(width*8))-1);accesses.append(f'write,{offset},{width},{word}');memory[offset:offset+width]=word.to_bytes(width,'little')
 return dict(output,result='integer:0',accesses=';'.join(accesses),memory=memory.hex())

def report():
 public=json.loads((HERE/'reference-verification.json').read_text());assert public['pin']==reference.PIN;cases=reference.cases();assert len(cases)==len(public['rows']);counts=collections.Counter();rows=[]
 for actual,row in zip(public['rows'],cases):
  assert all(actual[k]==v for k,v in row.items());assert actual['observed']==pin_expected(row),row['name']
  if isinstance(row['source'],int):category='integer_single_pass_agreement'
  elif 'text'in row['source']:category='string_entry_rejection_divergence'
  elif row['primary_total']>1:category='wider_buffer_single_pass_divergence'
  else:category='buffer_single_pass_agreement'
  counts[category]+=1;rows.append(dict(name=row['name'],category=category,primary_total=row['primary_total'],pin_field_passes=0 if category.startswith('string_')else 1))
 return dict(stage='public pinned callbacks and full memory versus exact one-pass oracle; primary sequence expectations explicitly separated',public_count=len(rows),counts=dict(sorted(counts.items())),rows=rows)
def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();r=report();file=HERE/'comparison.json'
 if a.write:file.write_text(json.dumps(r,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(file.read_text())==r
 print('PASS',r['public_count'],'public source comparisons',r['counts'])
if __name__=='__main__':main()
