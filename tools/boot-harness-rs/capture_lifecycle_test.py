"""Installed capture: ordinary delivery and live IPC participant/timeout cleanup."""
from pathlib import Path
import shutil


def exercise(output, click, key, scan, width, height, resumed, mode):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w, d, x, y): return d[(y*w+x)*3:(y*w+x+1)*3]
    def text(w, d, x, y, value):
        for i, ch in enumerate(value + ' '):
            for row, bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,d,x+i*12+col*2,y+row*2)==b'\xff\xff\xff') != bool(bits & (1<<(4-col))): return False
        return True
    def log(): return (output/'serial.log').read_text(errors='replace')
    right_x = 20 + (width-28)//2
    def note(slot, value, dirty=False):
        x = right_x if slot else 8
        scan(lambda w,d: pixel(w,d,x+5,50)==bytes.fromhex('9b7028' if dirty else '27664a') and text(w,d,x+10,60,value), f'capture document {slot}: {value!r}')
    left = 'hhh' if resumed else ''
    right = 'r'
    for cycle in range(3):
        if cycle == 0 or mode == 'source':
            click(152,height-48)
        note(0,left)
        if cycle == 0 and not resumed:
            note(1,'')
            click(right_x+5,160); key('r'); note(1,right,True); key('f1')
        note(1,right)
        click(width-8-384+60,18)
        scan(lambda w,d: pixel(w,d,right_x+5,50)==bytes.fromhex('101827'), 'source hidden before capture')
        start = len(log())
        click(width-8-512+60,18)
        if mode == 'ordinary':
            scan(lambda w,d: 'Cathedral capture: masked result received and presented' in log()[start:]
                 and pixel(w,d,21,169)==bytes.fromhex('27664a') and pixel(w,d,31,189)==bytes(3), 'ordinary capture viewer')
            assert pixel(width, (output/'display.ppm').read_bytes()[-width*height*3:], right_x+5,50)==bytes.fromhex('101827')
            shutil.copyfile(output/'display.png',output/f'capture-ordinary-{cycle}.png')
            click(width-8-512+60,18)
        else:
            expected = {'source': 'source exits before acknowledgment', 'recipient': 'recipient exits with pending request', 'timeout': 'recipient observed expiry'}[mode]
            scan(lambda w,d: f'Cathedral capture: {expected}' in log()[start:], f'capture {mode} fault reached')
            scan(lambda w,d: 'Cathedral capture: pending frame and output released' in log()[start:], 'pending output/demand reclaimed')
            assert 'Cathedral capture: masked result received and presented' not in log()[start:]
            assert 'capture-viewer-failed' not in log()[start:]
        scan(lambda w,d: 'Cathedral capture: consumer and retained result retired' in log()[start:], 'capture viewer retired')
        assert 'Cathedral capture: shared broker and capture ready' in log()[start:]
        # A sibling remains usable, then teardown and readmission must recover all
        # grants. Cold boot independently checks both saved documents again.
        click(13,160); key('h'); left += 'h'; note(0,left,True); key('f1'); note(0,left)
        if mode != 'source':
            click(width-8-384+60,18); note(1,right)
        if mode == 'source' or cycle == 2:
            click(width-22,18)
            scan(lambda w,d: pixel(w,d,13,50)==bytes.fromhex('101827'), 'capture subtree retired')
    click(152,height-48); note(0,left); note(1,right)
    click(width-22,18)
    scan(lambda w,d: pixel(w,d,13,50)==bytes.fromhex('101827'), 'final capture subtree retired')
    print(f'PASS: capture {mode}, three grants, sibling input, subtree reuse and saved state')
