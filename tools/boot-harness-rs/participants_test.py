"""Physical and synthetic input in the installed Split matrix, through normal F5."""
from pathlib import Path
import shutil

def exercise(output, click, key, scan, width, height, resumed):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w, d, x, y): return d[(y*w+x)*3:(y*w+x+1)*3]
    def text(w, d, x, y, value):
        for i, ch in enumerate(value + ' '):
            for row, bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,d,x+(i%15)*12+col*2,y+(i//15)*16+row*2)==b'\xff\xff\xff') != bool(bits & (1<<(4-col))): return False
        return True
    column = (width-28)//2
    positions = [(8,40,column),(20+column,40,column)]
    def note(slot, value, dirty=False):
        x,y,_ = positions[slot]
        scan(lambda w,d: pixel(w,d,x+5,y+10)==bytes.fromhex('9b7028' if dirty else '27664a') and text(w,d,x+10,y+20,value), f'participant document {slot}: {value!r}')
    def cursor(w,d):
        yellow = bytes.fromhex('ffd166')
        points = [(i//3%w,i//3//w) for i in range(0,len(d),3) if d[i:i+3]==yellow]
        assert all(positions[1][0] <= x < width-8 and 40 <= y < height-80 for x,y in points), points
        return bool(points)
    def log(): return (output/'serial.log').read_text(errors='replace')
    left, right = ('hhr','botbotbot') if resumed else ('','')
    click(152,height-48)
    note(0,left); note(1,right)
    for round in range(2):
        selected = log().count('Cathedral participant: private route selected')
        click(width-8-256+205,18)
        scan(lambda w,d: log().count('Cathedral participant: private route selected')>selected and cursor(w,d), 'synthetic route and visible scoped cursor')
        assert 'Cathedral participant: shared broker and producer ready; storage denied' in log()
        # Select the other document with physical input while the synthetic
        # keyboard continues using its own selection.
        click(13,160); key('h'); left += 'h'
        note(0,left,True)
        right += 'bot'; note(1,right,True)
        scan(cursor, 'cursor persists independently of physical keyboard focus')
        shutil.copyfile(output/'display.png',output/f'participants-{round}.png')
        click(width-8-256+205,18)
        scan(lambda w,d: not cursor(w,d), 'STOP retires cursor')
        for slot in (0,1):
            x,y,_=positions[slot]; click(x+5,y+120); key('f1')
        note(0,left); note(1,right)
    # A third run ends through view replacement, without STOP. Let its three
    # letters settle first so the durable result is deterministic.
    click(width-8-256+205,18)
    right += 'bot'; note(1,right,True)
    scan(cursor, 'third participant before resize')
    click(232,height-48); key('f2')
    scan(lambda w,d:text(w,d,(width-512)//2+4,height-78,'BAR HEIGHT'),'outer settings')
    click((width-512)//2+296,height-28); click((width-512)//2+392,height-28)
    column = (width-60)//2
    positions[:] = [(24,56,column),(36+column,56,column)]
    note(0,left); note(1,right,True)
    scan(lambda w,d:not cursor(w,d) and log().count('Cathedral Split: participant stopped and cursor retired')==3, 'view invalidation revokes producer and cursor')
    click((width-512)//2+472,height-44)
    click(29,176); key('r'); left += 'r'; note(0,left,True)
    for slot in (0,1):
        x,y,_=positions[slot]; click(x+5,y+120); key('f1')
    note(0,left); note(1,right)
    click(width-38,34)
    scan(lambda w,d:pixel(w,d,29,34)==bytes.fromhex('101827'), 'resized matrix retired')
    # Restore ordinary placement before the cold boot.
    click(248,height-64); key('f2')
    click((width-512)//2+216,height-44); click((width-512)//2+392,height-44)
    scan(lambda w,d:text(w,d,(width-512)//2+4,height-78,'BAR HEIGHT'),'restored settings')
    click((width-512)//2+472,height-28)
    assert log().count('Cathedral Split: participant admitted with scoped input and cursor') == 3
    assert log().count('Cathedral Split: participant stopped and cursor retired') == 3
    print('PASS: independent input, cursor, same-matrix regrant, resize revocation, sibling survival and saved documents')
