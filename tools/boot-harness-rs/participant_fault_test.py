"""Real IPC interruption evidence from disposable variants of the installed apps."""
from pathlib import Path
import re


def exercise(output, click, key, scan, width, height, resumed, mode):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w,d,x,y): return d[(y*w+x)*3:(y*w+x+1)*3]
    def text(w,d,x,y,value):
        for i,ch in enumerate(value + ' '):
            for row,bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,d,x+(i%15)*12+col*2,y+(i//15)*16+row*2)==b'\xff\xff\xff') != bool(bits & (1<<(4-col))): return False
        return True
    column=(width-28)//2
    positions=[(8,40),(20+column,40)]
    def note(slot,value,dirty=False):
        x,y=positions[slot]
        scan(lambda w,d: pixel(w,d,x+5,y+10)==bytes.fromhex('9b7028' if dirty else '27664a') and text(w,d,x+10,y+20,value), f'{mode} document {slot}: {value!r}')
    def log(): return (output/'serial.log').read_text(errors='replace')
    def rows(value):
        return [list(map(int,m.split())) for m in re.findall(r'STORAGE_TRACE receiver-input ([0-9 ]+)\n',value)]
    def cursor(w,d): return any(d[i:i+3]==bytes.fromhex('ffd166') for i in range(0,len(d),3))
    left = 'hh' if resumed else ''
    right = ('kaakaa' if mode=='pressure' else 'aaaa') if resumed else ''
    click(152,height-48)
    note(0,left);note(1,right)
    for turn in range(2):
        start=len(log())
        click(width-8-256+205,18)
        marker = 'Cathedral receiver probe: stalled' if mode=='pressure' else 'Cathedral participant probe: holding pointer and key'
        scan(lambda w,d: marker in log()[start:] and cursor(w,d), 'producer active with captured input')
        if mode!='pressure':
            right+='a';note(1,right,True)
        click(13,160);key('h');left+='h';note(0,left,True)
        if mode=='pressure':
            # Another channel targeting the stalled receiver must survive the
            # overflowing producer stream. The left document remained usable.
            x,y=positions[1];click(x+5,y+120);key('k')
            right+='ka';note(1,right,True)
            scan(lambda w,d:'Cathedral participant probe: fresh input after pressure' in log()[start:], 'producer completed fresh input')
            evidence=log()[start:]
            ordered=['Cathedral receiver probe: stalled','Cathedral participant probe: flooding one channel','Cathedral participant probe: flood complete','Cathedral receiver probe: resumed','Cathedral participant probe: fresh input after pressure']
            assert [evidence.index(m) for m in ordered]==sorted(evidence.index(m) for m in ordered), evidence
            cancels=[r for r in rows(evidence) if r[1]==3]
            assert len(cancels)==1 and cancels[0][4]==1<<(14-1) and cancels[0][5]==0, cancels
        if mode!='disconnect': click(width-8-256+205,18)
        scan(lambda w,d: 'participant stopped and cursor retired' in log()[start:] and not cursor(w,d), 'producer and visible cursor retired')
        scan(lambda w,d: len([r for r in rows(log()[start:]) if r[1]==5])==2 and 'receiver-group-retired' in log()[start:], 'both channel retirements followed by group retirement')
        evidence=log()[start:]
        retired=[r for r in rows(evidence) if r[1]==5]
        if mode!='pressure':
            assert sorted(r[4] for r in retired)==[0,1<<(15-1)],retired
            assert sorted(r[5] for r in retired)==[0,1],retired
        if mode=='disconnect': assert 'producer endpoint closed' in evidence
        x,y=positions[1];click(x+5,y+120);key('a');right+='a';note(1,right,True)
        for x,y in positions: click(x+5,y+120);key('f1')
        note(0,left);note(1,right)
    generations=list(map(int,re.findall(r'STORAGE_TRACE producer-generation (\d+)',log())))
    assert len(generations)==2 and generations[1]>generations[0],generations
    click(width-22,18)
    scan(lambda w,d:pixel(w,d,13,18)==bytes.fromhex('101827'),'matrix subtree collected')
    print(f'PASS: {mode}, real receiver boundaries, unaffected input, fresh identities and durable edits')
