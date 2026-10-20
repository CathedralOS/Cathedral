"""Recursive ordinary desktops: three nested logins, persistence and reclamation."""
from pathlib import Path
import shutil
import time


def exercise(output, click, key, scan, width, height, reboot):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w, data, x, y):
        return data[(y*w+x)*3:(y*w+x+1)*3]
    def text(w, data, x, y, value):
        for i, ch in enumerate(value):
            for row, bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,data,x+i*12+col*2,y+row*2)==b'\xff\xff\xff') != bool(bits & (1 << (4-col))):
                        return False
        return True
    def label(x, y, value):
        scan(lambda w,d: text(w,d,x,y,value), value)
    def viewport(depth):
        return (8*depth, 8*depth, width-16*depth, height-104*depth)
    def launch(depth, index):
        x,y,w,h = viewport(depth)
        click(x+40+56*index, y+h-48)
    def task_close(depth, index=0):
        x,y,w,h = viewport(depth)
        click(x+376+144*index, y+h-48)
    def app_close(depth):
        x,y,w,h = viewport(depth)
        click(x+w-22, y+18)
    def desktop(depth):
        x,y,w,h = viewport(depth)
        scan(lambda sw,d: pixel(sw,d,x+148,y+h-48)==bytes.fromhex('c7a2ed') and text(sw,d,x+w-150,y+h-32,'SETTINGS'), f'depth {depth} taskbar')
    def enter(depth, create):
        started = time.monotonic()
        launch(depth-1, 3 if depth==1 else 2)
        x,y,w,h = viewport(depth)
        top = y+(h-96)//2
        if create:
            left = x+(w-128)//2
            label(left+16,top+76,'NEW USER')
            click(left+64,top+48)
        left = x+(w-280)//2
        label(left+28,top+76,'USER 1')
        click(left+64,top+48)
        desktop(depth)
        print(f'Nested desktop depth {depth} ready after {time.monotonic()-started:.2f}s', flush=True)
    def note(depth, value, dirty=False):
        x,y,w,h = viewport(depth)
        scan(lambda sw,d: pixel(sw,d,x+13,y+18)==bytes.fromhex('9b7028' if dirty else '27664a') and text(sw,d,x+18,y+28,value),f'depth {depth} Notes {value!r}')
    def background(depth):
        x,y,w,h = viewport(depth)
        # Clearing a surface precedes child collection. Every nested background
        # has the same color, so it cannot prove that a subtree was stopped.
        # Wait for the owner's first running-task cell to become empty as well.
        scan(lambda sw,d: pixel(sw,d,x+13,y+18)==bytes.fromhex('101827')
             and pixel(sw,d,x+249,y+h-60)==bytes.fromhex('101827'),f'depth {depth} closed and collected')

    for depth in range(1,4):
        enter(depth, not reboot)
    shutil.copyfile(output/'display.png', output/'three-nested-desktops.png')
    launch(3,0)
    note(3,'deep' if reboot else '')
    if not reboot:
        click(37,144)
        for ch in 'deep': key(ch)
        note(3,'deep',True)
        task_close(0) # Loss propagates through the whole subtree.
        label(width-192,height-56,'UNSAVED WORK')
        note(3,'deep',True)
        click(37,144); key('f1'); note(3,'deep')
    app_close(3); background(3)
    # Collapse each level, then re-enter all three. Old child accounts, image
    # readers and endpoints must be reusable without rebooting the outer desktop.
    for _ in range(2):
        task_close(0); background(0)
        for depth in range(1,4): enter(depth,False)
        launch(3,0); note(3,'deep'); app_close(3); background(3)
    task_close(0); background(0)
    launch(0,0); note(0,'') # Inner saved content never leaks to the outer realm.
    app_close(0); background(0)
    print('PASS: three recursive desktops, protected close, saved inner data, repeated subtree reclamation and isolation')
