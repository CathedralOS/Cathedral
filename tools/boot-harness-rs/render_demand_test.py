"""Display demand is independent of live input, using installed Split controls."""
from pathlib import Path
import shutil


def exercise(output, click, key, scan, width, height, resumed, capture=False):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w, d, x, y): return d[(y*w+x)*3:(y*w+x+1)*3]
    def text(w, d, x, y, value):
        for i, ch in enumerate(value + ' '):
            for row, bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,d,x+i*12+col*2,y+row*2)==b'\xff\xff\xff') != bool(bits & (1<<(4-col))): return False
        return True
    column = (width-28)//2
    positions = [(8,40), (20+column,40)]
    def note(slot, value, dirty=False):
        x,y = positions[slot]
        scan(lambda w,d: pixel(w,d,x+5,y+10)==bytes.fromhex('9b7028' if dirty else '27664a') and text(w,d,x+10,y+20,value), f'demand document {slot}: {value!r}')
    def log(): return (output/'serial.log').read_text(errors='replace').replace('Cathedral Rust lab: ', '')
    left, right = ('hh','botbot') if resumed else ('','')
    click(152,height-48)
    note(0,left); note(1,right)
    for round in range(2):
        selected = log().count('Cathedral participant: private route selected')
        click(width-8-256+205,18)
        scan(lambda w,d: log().count('Cathedral participant: private route selected')>selected, 'AUTO selected right Notes')
        start = len(log())
        click(width-8-384+60,18)
        scan(lambda w,d: 'STORAGE_TRACE note-demand 0' in log()[start:], 'Notes acknowledged no frame demand')
        # Wait for an actual edit made after that acknowledgment. No sleep or
        # owner-side flag substitutes for evidence from the running app.
        wanted = len(right)+3
        scan(lambda w,d: any(line.startswith('STORAGE_TRACE note-hidden-edit ') and line.split()[-1]==str(wanted) for line in log()[start:].splitlines()), 'last AUTO letter processed while hidden')
        hidden = log()[start:].split('STORAGE_TRACE note-demand 0',1)[1]
        assert 'STORAGE_TRACE split-publication 1 ' not in hidden, hidden
        x,y = positions[1]
        scan(lambda w,d: pixel(w,d,x+5,y+10)==bytes.fromhex('101827'), 'hidden content removed from scene')
        # The physical route can still edit a sibling. Clicking the hidden area
        # cannot select it or deliver a physical key through its old placement.
        click(x+5,y+120); key('z')
        click(13,160); key('h'); left += 'h'; note(0,left,True)
        assert 'STORAGE_TRACE split-publication 1 ' not in log()[start:].split('STORAGE_TRACE note-demand 0',1)[1]
        shutil.copyfile(output/'display.png',output/f'demand-hidden-{round}.png')
        if capture:
            received=log().count('Cathedral capture: masked result received and presented')
            click(width-8-512+60,18)
            scan(lambda w,d: log().count('Cathedral capture: masked result received and presented')>received
                 and pixel(w,d,21,169)==bytes.fromhex('9b7028')
                 and pixel(w,d,31,189)==bytes(3), 'separate consumer displays fresh masked capture')
            assert log().count('Cathedral capture: recipient verified redaction, padding and single delivery')==round+1
            # Capturing has not revealed the source, and demand stops afterward.
            scan(lambda w,d: pixel(w,d,positions[1][0]+5,50)==bytes.fromhex('101827')
                 and log()[start:].count('STORAGE_TRACE note-demand 0')>=2, 'capture releases demand without revealing source')
            shutil.copyfile(output/'display.png',output/f'capture-{round}.png')
            click(width-8-512+60,18)
            scan(lambda w,d: log().count('Cathedral capture: consumer and retained result retired')==round+1, 'consumer and image retired')

        right += 'bot'
        click(width-8-384+60,18)
        note(1,right,True)
        shutil.copyfile(output/'display.png',output/f'demand-restored-{round}.png')
        click(width-8-256+205,18) # STOP retires the producer and cursor.
        for slot in (0,1):
            x,y=positions[slot]; click(x+5,y+120); key('f1')
        note(0,left); note(1,right)
    # Retire with a hidden, clean child, then reuse the same app slot/budget.
    # The replacement starts visible and loads the saved content.
    for reopen in (True, False):
        before = log().count('STORAGE_TRACE note-demand 0')
        click(width-8-384+60,18)
        scan(lambda w,d: log().count('STORAGE_TRACE note-demand 0')>before, 'hidden child before teardown')
        click(width-22,18)
        scan(lambda w,d:pixel(w,d,13,50)==bytes.fromhex('101827'), 'Split with hidden child retired')
        if reopen:
            click(152,height-48)
            note(0,left); note(1,right)
    assert log().count('Cathedral Split: participant stopped and cursor retired') == 2
    print('PASS: repeated hidden editing without publication, sibling input, restore and saved state')
