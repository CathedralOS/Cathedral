#!/usr/bin/env python3
"""Check actual public callbacks against independent interval/bit arithmetic.

Pin-only differences are recorded, never enabled as a production compatibility mode.
"""
import argparse,collections,json
from pathlib import Path
import reference,vectors
HERE=reference.HERE
INITIAL=bytes((11+37*i)&255 for i in range(512))
SOURCE=0x123456789abcdef0

def expected(row):
 memory=bytearray(INITIAL);log=[];flags=row['flags'];code=flags&15;rule=(flags>>5)&3
 if code>5:return dict(load='ok',result='error:InvalidFieldFlags',accesses='',memory=memory.hex(),created_mutexes='1',forbidden_calls='0')
 width=[1,1,2,4,8,1][code];bits=width*8;offset=row['offset'];length=row['length'];lead=offset%bits
 count=(lead+length+bits-1)//bits;first=(offset//bits)*width;result=0;written=0;panic=False
 for index in range(count):
  byte=first+index*width;native=lead if index==0 else 0;size=min(length-written,bits-native);mask=((1<<size)-1)<<native
  old=int.from_bytes(memory[byte:byte+width],'little')
  if row['write']:
   partial=native>0 or length-written<bits
   if partial and rule==3:panic=True;break
   base=old if partial and rule==0 else (1<<bits)-1 if partial and rule==1 else 0
   if partial and rule==0:log.append(f'read,{byte},{width},{old}')
   word=((base&~mask)|(((SOURCE>>written)&((1<<size)-1))<<native))&((1<<bits)-1)
   log.append(f'write,{byte},{width},{word}');memory[byte:byte+width]=word.to_bytes(width,'little')
  else:
   log.append(f'read,{byte},{width},{old}');result|=((old>>native)&((1<<size)-1))<<written
  written+=size
 output=dict(load='ok',accesses=';'.join(log),memory=memory.hex(),created_mutexes='1',forbidden_calls='0')
 if panic:output['panic']='true'
 elif row['write']:output['result']='integer:0'
 elif length<= (32 if row['revision']==1 else 64):output['result']='integer:'+str(result)
 else:
  byte_count=((length+7)//8)*8;output['result']=f'buffer:{byte_count}:'+result.to_bytes(byte_count,'little').hex()
 return output

def report():
 public=json.loads((HERE/'reference-verification.json').read_text());assert public['pin']==reference.PIN
 assert len(public['rows'])==len(reference.cases());counts=collections.Counter();rows=[]
 for observed,case in zip(public['rows'],reference.cases()):
  assert all(observed[key]==value for key,value in case.items())
  assert observed['observed']==expected(case),(case['name'],observed['observed'],expected(case))
  strict=case['strict_expected'];category='strict_geometry_agrees'
  if 'error'in strict:category='strict_rejects_'+strict['error']
  elif not case['write']and strict['shape']=='Buffer':category='strict_buffer_size_correction'
  counts[category]+=1
  rows.append(dict(name=case['name'],category=category,access_count=len(observed['observed']['accesses'].split(';'))if observed['observed']['accesses']else 0))
 return dict(stage='independent interval/bit arithmetic checked against actual public callback tuples and complete memory images',public_count=len(rows),counts=dict(sorted(counts.items())),rows=rows)

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();out=report();path=HERE/'comparison.json'
 if a.write:path.write_text(json.dumps(out,indent=2,sort_keys=True)+'\n')
 else:assert json.loads(path.read_text())==out
 print('PASS',out['public_count'],'public arithmetic/access comparisons',out['counts'])
if __name__=='__main__':main()
