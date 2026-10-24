"""Pointer-target suppression through real app IPC and nested matrices."""
from pathlib import Path
import shutil


def exercise(output, click, key, scan, width, height, resumed, synthetic=False):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w, d, x, y): return d[(y*w+x)*3:(y*w+x+1)*3]
    def text(w, d, x, y, value):
        for i, ch in enumerate(value + ' '):
            for row, bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,d,x+i*12+col*2,y+row*2)==b'\xff\xff\xff') != bool(bits & (1<<(4-col))): return False
        return True
    def pointer(x, y, visible):
        scan(lambda w,d: (pixel(w,d,x,y)==b'\xff\xff\xff') == visible, f'pointer at {x},{y}, visible={visible}')
    def yellow(w,d): return any(d[i:i+3]==b'\xff\xd1\x66' for i in range(0,len(d),3))
    column = (width-28)//2
    right = 20+column
    click(152,height-48)
    scan(lambda w,d: pixel(w,d,13,50)==bytes.fromhex('27664a'), 'Split notes ready')
    if synthetic:
        # A disposable participant invokes the same F6 action using its own
        # keyboard group. No privileged test-only cursor command is involved.
        click(width-8-256+205,18)
        def suppressed(w, d):
            log = (output/'serial.log').read_text(errors='replace')
            return 'requested own cursor suppression' in log and not yellow(w,d)
        scan(suppressed, 'AUTO pointer preference reached its owner renderer')
        click(right+5,160); pointer(right+5,160,True)
        key('h')
        scan(lambda w,d:text(w,d,right+10,60,'bothboth' if resumed else 'both'), 'physical input coexists with hidden AUTO')
        key('f1')
        scan(lambda w,d:pixel(w,d,right+5,50)==bytes.fromhex('27664a'), 'saved private-group edits')
        click(width-8-256+205,18)
        scan(lambda w,d:not yellow(w,d), 'stopped hidden producer')
        click(width-22,18)
        scan(lambda w,d:pixel(w,d,13,18)==bytes.fromhex('101827'), 'Split retired')
        print('PASS: producer-group cursor suppression leaves physical pointer and keyboard usable')
        return
    click(13,160); pointer(13,160,True)
    key('f6'); pointer(13,160,False)
    key('h')
    scan(lambda w,d:text(w,d,18,60,'hh' if resumed else 'h'), 'hidden cursor still permits input')
    key('f1')
    scan(lambda w,d:pixel(w,d,13,50)==bytes.fromhex('27664a'), 'saved hidden-cursor document')
    click(right+5,160); pointer(right+5,160,True)
    click(13,160); pointer(13,160,False)
    key('f6'); pointer(13,160,True)
    key('f6'); pointer(13,160,False)
    # AUTO is separately registered: hiding the physical pointer must not hide it.
    click(width-8-256+205,18)
    scan(yellow, 'independent AUTO cursor')
    click(13,160); pointer(13,160,False)
    scan(yellow, 'AUTO remains visible with physical pointer suppressed')
    scan(lambda w,d:text(w,d,right+10,60,'botbot' if resumed else 'bot'), 'AUTO edits independently')
    click(width-8-256+205,18)
    scan(lambda w,d:not yellow(w,d), 'STOP retires only AUTO cursor')
    click(right+5,160); key('f1')
    scan(lambda w,d:pixel(w,d,right+5,50)==bytes.fromhex('27664a'), 'saved AUTO document')
    # Close the hidden-pointer target. The final click position is unchanged;
    # owner recomputation must restore cursor pixels without further motion.
    click(8+column-14,50)
    scan(lambda w,d:pixel(w,d,13,50)==bytes.fromhex('101827'), 'target process retired')
    pointer(8+column-14,50,True)
    click(width-22,18)
    scan(lambda w,d:pixel(w,d,13,18)==bytes.fromhex('101827'), 'Split retired')
    # Repeat through the same Split program nested beneath itself.
    click(152,height-48)
    scan(lambda w,d:pixel(w,d,13,50)==bytes.fromhex('27664a'), 'Split restarted')
    click(width-8-256+130,18)
    scan(lambda w,d:pixel(w,d,13,82)==bytes.fromhex('27664a'), 'nested Notes ready')
    click(13,192); pointer(13,192,True)
    key('f6'); pointer(13,192,False)
    click(13,18); pointer(13,18,True)
    click(13,192); pointer(13,192,False)
    shutil.copyfile(output/'display.png',output/'cursor-nested-hidden.png')
    click(8+column-14,82)
    scan(lambda w,d:pixel(w,d,13,82)==bytes.fromhex('101827'), 'nested target retired')
    pointer(8+column-14,82,True)
    click(width-22,18)
    scan(lambda w,d:pixel(w,d,13,18)==bytes.fromhex('101827'), 'recursive close')
    print('PASS: cursor hide/restore, independent producer, target death, recursive preferences and cold saved text')
