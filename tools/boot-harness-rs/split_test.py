"""Visible nested-matrix assertions used by desktop_test.py --nested."""
from pathlib import Path
import shutil

def exercise(output, click, key, scan, width, height, resumed=False, deep=False):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w,d,x,y): return d[(y*w+x)*3:(y*w+x+1)*3]
    def text(w,d,x,y,value):
        for i,ch in enumerate(value):
            for row,bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,d,x+i*12+col*2,y+row*2)==b'\xff\xff\xff') != bool(bits & (1<<(4-col))): return False
        return True
    app_x, top, margin = 232,24,24
    def positions():
        column=(width-margin-app_x-12)//2
        return [(app_x,top+32,column),(app_x+column+12,top+32,column)]
    def note(slot,value,dirty=False):
        x,y,_=positions()[slot]
        scan(lambda w,d: pixel(w,d,x+5,y+10)==bytes.fromhex('9b7028' if dirty else '27664a') and text(w,d,x+10,y+20,value), f'Split child {slot} text {value!r}')
    def blank(): scan(lambda w,d: pixel(w,d,app_x+5,top+10)==bytes.fromhex('101827'),'Split subtree retired')
    for round in range(3):
        click(120,336)
        note(0,'ax' if resumed or round else ''); note(1,'by' if resumed or round else '')
        if round==0 and not resumed:
            for slot,ch in [(0,'a'),(1,'b')]:
                x,y,_=positions()[slot]; click(x+5,y+120); key(ch); note(slot,ch,True)
            note(0,'a',True); note(1,'b',True)
            # A matrix and two ordinary siblings fit together. Hidden descendants
            # retain their drafts while chrome changes the selected outer app.
            click(120, 96)  # Notes in outer app slot two.
            scan(lambda w,d: pixel(w,d,app_x+5,top+10)==bytes.fromhex('27664a'), 'Notes beside live Split')
            click(app_x+5,top+120); key('q')
            click(120,216)  # Colors in slot three.
            scan(lambda w,d: pixel(w,d,app_x+5,top+10)==bytes.fromhex('315ca8'), 'Colors beside live Split and Notes')
            click(width-margin-14,top+10)  # Close Colors; Split becomes selected.
            note(0,'a',True); note(1,'b',True)
            click(90,24+384+32+16)  # Select the still-live outer Notes.
            scan(lambda w,d: text(w,d,app_x+10,top+20,'q'), 'outer Notes draft survived matrix switch')
            click(width-margin-14,top+10)
            scan(lambda w,d: text(w,d,app_x+10,top+20,'UNSAVED NOTE'), 'outer Notes close')
            click(app_x+40,top+60)  # Discard its private draft.
            note(0,'a',True); note(1,'b',True)
            # Matrix close surveys children without blocking their upstream calls.
            click(width-margin-14,top+10)
            scan(lambda w,d:text(w,d,width-margin-256+8,top+6,'UNSAVED NOTES'),'recursive dirty close refused')
            note(0,'a',True); note(1,'b',True)
            # The outer matrix changes its placement. Both inner drafts survive.
            click(40,430); key('f2')
            scan(lambda w,d:text(w,d,44,50,'APP STRIP'),'outer settings')
            click(160,100); click(80,260) # width +64; save
            app_x=296
            note(0,'a',True); note(1,'b',True)
            click(208,260) # new panel Cancel (margin + (256-160)/2 +120)
            for slot,ch in [(0,'x'),(1,'y')]:
                x,y,_=positions()[slot]; click(x+5,y+120); key(ch)
            note(0,'ax',True); note(1,'by',True)
            shutil.copyfile(output/'display.png',output/'split-resized.png')
            for slot in (0,1):
                x,y,cw=positions()[slot]
                click(x+cw-14,y+10)
                scan(lambda w,d:text(w,d,x+10,y+20,'UNSAVED NOTE'),f'child {slot} owns close dialog')
                click(x+40,y+44) # Save this child's draft before closing
                scan(lambda w,d:pixel(w,d,x+5,y+10)==bytes.fromhex('101827'),f'child {slot} retired')
            click(width-margin-14,top+10); blank()
            # Restore saved layout so cold boot tests the identical composition.
            click(40,430); key('f2')
            click(112,100); click(112,260) # smaller strip, save
            app_x=232
            scan(lambda w,d:text(w,d,44,50,'APP STRIP'),'restored outer layout')
            click(160,260) # Cancel in the original panel
            scan(lambda w,d: pixel(w,d,100,70)==bytes.fromhex('d9e4ef'),'launcher restored')
        else:
            click(width-margin-14,top+10); blank()
    log=(output/'serial.log').read_text(errors='replace')
    assert log.count('Cathedral Split: two Notes admitted with separate persistent realms')==3,log[-3000:]
    assert 'Cathedral Notes: temporary' not in log,log[-3000:]
    print('PASS: independent Split layout, focus, isolated persistence, dirty-close, resize and three reclaimed subtrees')
    if not deep:
        return
    for round in range(2):
        click(120,336)
        note(0,'ax'); note(1,'by') # Deep edits never replace these siblings.
        click(width-margin-256+130,top+10)
        scan(lambda w,d:text(w,d,width-margin-256+8,top+6,'NESTED'),'same-code child matrix admitted')
        top += 32
        note(0,'axa' if resumed or round else 'ax')
        note(1,'byb' if resumed or round else 'by')
        if not resumed and round == 0:
            for slot,ch in [(0,'a'),(1,'b')]:
                x,y,_=positions()[slot]; click(x+5,y+120); key(ch)
            note(0,'axa',True); note(1,'byb',True)
            click(width-margin-14,24+10)
            scan(lambda w,d:text(w,d,width-margin-256+8,24+6,'UNSAVED NOTES'),'deep dirty close refused')
            # The inner matrix lacks another subtree's funding. Refusal must
            # retain its live children and drafts, and leave closing usable.
            click(width-margin-256+130,top+10)
            scan(lambda w,d:text(w,d,width-margin-256+8,top+6,'NO ROOM'),'bounded deeper admission refusal')
            note(0,'axa',True); note(1,'byb',True)
            for slot in (0,1):
                x,y,_=positions()[slot]; click(x+5,y+120); key('f1')
            note(0,'axa'); note(1,'byb')
            shutil.copyfile(output/'display.png',output/'split-deeper.png')
        click(width-margin-14,24+10)
        top -= 32
        blank()
    print('PASS: same matrix at another level, confined durable edits, recursive close and funded refusal')
