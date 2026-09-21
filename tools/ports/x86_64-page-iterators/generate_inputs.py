#!/usr/bin/env python3
"""Public-API observations around both canonical edges and iterator updates."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
LOW=2**47;HIGH=2**64-2**47;MAX=2**64-1
rows=[]
for size in [4096,2**21,2**30]:
 low=LOW-size;top=2**64-size
 intervals=[('zero',0,0),('short',0,3*size),('reversed',3*size,size),('low-singleton',low,low),('low-pair',low-size,low),('high-singleton',HIGH,HIGH),('high-pair',HIGH,HIGH+size),('top-singleton',top,top),('top-pair',top-size,top),('gap-pair',low,HIGH),('gap-wide',low-2*size,HIGH+3*size),('zero-to-high',0,HIGH+size),('low-to-top',low,top),('whole',0,top),('reversed-gap',HIGH,low)]
 for name,start,end in intervals:
  distance=(end-start)//size if end>=start else 0
  for inclusive in [False,True]:
   count=distance+int(inclusive)if end>=start else 0
   for reverse in [False,True]:
    boundary=((end&(2**48-1))-((LOW-size)&(2**48-1)))//size if reverse and end>=LOW-size else (LOW-start)//size if not reverse and start<=LOW else 0
    indices=sorted({0,1,2,3,4,max(0,count-1),count,count+1,MAX,max(0,boundary-1),boundary,boundary+1})
    for index in indices:
     rows.append(dict(size=size,start=start,end=end,inclusive=inclusive,index=index,reverse=reverse,direct=False,family=name))
    rows.append(dict(size=size,start=start,end=end,inclusive=inclusive,index=0,reverse=reverse,direct=True,family=name))
 # Added input policy, compared to actual public typed construction failure.
 for start,end in [(1,0),(LOW,0),(0,HIGH-4096),(MAX,0)]:
  rows.append(dict(size=size,start=start,end=end,inclusive=True,index=0,reverse=False,direct=False,family='invalid-address'))
def write(path,text):
 if '--check' in sys.argv:assert path.read_text()==text,('stale',path)
 else:path.write_text(text)
checks=[]
for n,row in enumerate(rows):
 size={4096:'Size4KiB',2**21:'Size2MiB',2**30:'Size1GiB'}[row['size']]
 checks.append('emit('+str(n)+',observe::<'+size+'>('+','.join(str(row[k]).lower()for k in ['start','end','inclusive','index','reverse','direct'])+'));')
write(HERE/'inputs.json',json.dumps(rows,indent=2)+'\n')
write(HERE/'src/main.rs',(HERE/'probe.rs.in').read_text().replace('// GENERATED_CHECKS','\n'.join(checks)))
print(len(rows),'actual public iterator or typed-input observations')
