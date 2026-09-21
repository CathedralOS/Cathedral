#!/usr/bin/env python3
"""Bind actual public read callbacks/results to independent normalized-field values."""
import argparse,collections,json
import reference,vectors
HERE=reference.HERE

def pin_expected(row):
 memory=bytes(vectors.byte_at(i,row['pattern'])for i in range(512));code=row['flags']&15
 output=dict(load='ok',memory=memory.hex(),created_mutexes='1',forbidden_calls='0')
 if code>5:output.update(result='error:InvalidFieldFlags',accesses='');return output
 width=[1,1,2,4,8,1][code];bits=width*8;first=(row['offset']//bits)*width;count=(row['length']+row['offset']%bits+bits-1)//bits
 accesses=[]
 for i in range(count):
  offset=first+i*width;value=int.from_bytes(memory[offset:offset+width],'little');accesses.append(f'read,{offset},{width},{value}')
 value=(int.from_bytes(memory,'little')>>row['offset'])&((1<<row['length'])-1)
 output['accesses']=';'.join(accesses)
 if row['length']<= (32 if row['revision']==1 else 64):output['result']='integer:'+str(value)
 else:
  length=(row['length']+7)//8*8;output['result']=f'buffer:{length}:'+value.to_bytes(length,'little').hex()
 return output

def report():
 public=json.loads((HERE/'reference-verification.json').read_text());assert public['pin']==reference.PIN
 actual=public['rows'];cases=reference.cases();assert len(actual)==len(cases);counts=collections.Counter();rows=[]
 for row,case in zip(actual,cases):
  assert all(row[k]==v for k,v in case.items());assert row['observed']==pin_expected(case),case['name']
  strict=case['strict_expected']
  if 'error'in strict:category='strict_geometry_rejection_'+strict['cause']
  elif strict['shape']=='Integer':
   assert row['observed']['result']=='integer:'+str(strict['value']);category='integer_agreement'
  else:
   _,length,hexbytes=row['observed']['result'].split(':');raw=bytes.fromhex(hexbytes);assert int(length)==strict['length']*8
   assert raw[:strict['length']]==bytes(strict['bytes'][:strict['length']]);assert all(b==0 for b in raw[strict['length']:]);category='buffer_value_agreement_size_correction'
  counts[category]+=1;rows.append(dict(name=case['name'],category=category))
 return dict(stage='actual public reads compared to independent logical-bit values and exact callback/memory observations',public_count=len(rows),counts=dict(sorted(counts.items())),rows=rows)

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();result=report();path=HERE/'comparison.json'
 if a.write:path.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==result
 print('PASS',result['public_count'],'public read comparisons',result['counts'])
if __name__=='__main__':main()
