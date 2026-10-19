"""Visible nested-matrix assertions used by desktop_test.py --nested."""
from pathlib import Path
import shutil

def exercise(output, click, key, scan, width, height):
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
        scan(lambda w,d: pixel(w,d,x+5,y+10)==bytes.fromhex('9b7028' if dirty else '55447a') and text(w,d,x+10,y+20,value), f'Split child {slot} text {value!r}')
    def blank(): scan(lambda w,d: pixel(w,d,app_x+5,top+10)==bytes.fromhex('101827'),'Split subtree retired')
    for round in range(3):
        click(120,336)
        note(0,''); note(1,'')
        if round==0:
            for slot,ch in [(0,'a'),(1,'b')]:
                x,y,_=positions()[slot]; click(x+5,y+120); key(ch); note(slot,ch,True)
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
                click(x+40,y+60) # Discard temporary note
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
    assert log.count('Cathedral Split: two Notes admitted; storage withheld from children')==3,log[-3000:]
    assert log.count('Cathedral Notes: temporary; no storage endpoint; interaction storage denied')==6,log[-3000:]
    print('PASS: independent Split layout, focus, attenuated storage, dirty-close, resize and three reclaimed subtrees')
