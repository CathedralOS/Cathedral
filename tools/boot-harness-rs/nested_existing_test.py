"""Replay one saved outer user through nested login on a disposable disk copy.

No installer or program migration touches the copied object store. The current
provider/kernel boot existing nested program snapshots, creating missing users.
Requires one outer User 1 and the stock compact taskbar. Source remains unchanged.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import capture
import database_image
import desktop_test
import disk_image
import run as harness


def exercise(depths, output, click, key, scan, width, height, reboot):
    font = bytes.fromhex((harness.WORKSPACE/'platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w, data, x, y):
        return data[(y*w+x)*3:(y*w+x+1)*3]
    def text(w, data, x, y, value):
        return all((pixel(w,data,x+i*12+c*2,y+r*2)==b'\xff\xff\xff') == bool(bits & (1 << (4-c)))
                   for i,ch in enumerate(value)
                   for r,bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]) for c in range(5))
    for depth in range(1, depths+1):
        parent = depth-1
        click(8*parent+40+56*(3 if depth==1 else 2),8*parent+height-104*parent-48)
        x = y = 8*depth
        w, h = width-16*depth, height-104*depth
        top, fresh, saved = y+(h-96)//2, x+(w-128)//2, x+(w-280)//2
        state = []
        def picker(sw, data):
            if text(sw,data,saved+28,top+76,'USER 1'):
                state[:] = [False]
            elif text(sw,data,fresh+16,top+76,'NEW USER'):
                state[:] = [True]
            return bool(state)
        scan(picker, f'existing depth {depth} picker')
        if state[0]:
            click(fresh+64,top+48)
            scan(lambda sw,d:text(sw,d,saved+28,top+76,'USER 1'),f'depth {depth} user creation')
        click(saved+64,top+48)
        scan(lambda sw,d:pixel(sw,d,x+148,y+h-48)==bytes.fromhex('c7a2ed')
             and text(sw,d,x+w-150,y+h-32,'SETTINGS'),f'existing depth {depth} taskbar')
        print(f'Existing desktop depth {depth} ready',flush=True)
    # Exercise the leaf while all ancestors remain alive. Edits stay on the copy.
    click(x+40,y+h-48)
    scan(lambda sw,d:pixel(sw,d,x+13,y+18)==bytes.fromhex('27664a'),'deep Notes loaded')
    click(x+13,y+120); key('z')
    scan(lambda sw,d:pixel(sw,d,x+13,y+18)==bytes.fromhex('9b7028'),'deep Notes accepts input')
    click(376,height-48)
    scan(lambda sw,d:text(sw,d,width-192,height-56,'UNSAVED WORK'),'dirty subtree preserved')
    click(x+13,y+120);key('f1')
    scan(lambda sw,d:pixel(sw,d,x+13,y+18)==bytes.fromhex('27664a'),'deep save confirmed')
    click(x+w-22,y+18)
    scan(lambda sw,d:pixel(sw,d,x+249,y+h-60)==bytes.fromhex('101827'),'deep Notes collected')
    click(376,height-48)
    scan(lambda sw,d:pixel(sw,d,249,height-60)==bytes.fromhex('101827'),'subtree collected')
    serial=(output/'serial.log').read_text(errors='replace')
    assert serial.count('Cathedral: object storage ready')==1, 'provider restarted'
    assert 'matrix-service-exit' not in serial, 'matrix retired unexpectedly'
    print('PASS: saved subtree login, editing, protected close, save and collection',flush=True)


def run(source, depth, timeout):
    database_image.require_available(source)
    original = source.read_bytes()
    output=harness.BUILD/'nested-existing'/datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    print(f'Saved-disk replay artifacts: {output}', flush=True)
    profile=output/f'nested-existing-{output.name}.json'
    shutil.copyfile(harness.WORKSPACE/'distribution/profile.json',profile)
    subprocess.run([sys.executable,str(Path(__file__).with_name('run.py')),'--release','--build-only',
                    '--trace-storage','--profile',str(profile)],check=True)
    shutil.copytree(harness.BUILD/f'{profile.stem}-interactive/esp',output/'esp')
    image=output/'guest.raw';image.write_bytes(original)
    (output/'source-sha256.txt').write_text(hashlib.sha256(original).hexdigest()+'\n')
    qemu=harness.find_qemu()
    command=[qemu,'-machine','q35','-cpu','qemu64','-accel','tcg','-smp','1','-m','128']
    command+=harness.firmware_args(qemu,output)
    command+=['-drive','format=raw,file=fat:rw:esp','-nic','none','-monitor','none','-no-reboot','-display','none']
    command+=disk_image.arguments(image)
    try:
        for boot in (1,2):
            capture.run(command,output,timeout,subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0,
                        ready_marker='Cathedral desktop: ready; choose a user',
                        verify=lambda stream,out,serial:desktop_test.desktop(stream,out,serial,True,
                            nested_desktop=True,pointer_stress=True,
                            user_check=lambda *args:exercise(depth,*args)))
            for name in ('serial.log','display.png','qemu.log'):
                shutil.copyfile(output/name,output/f'boot-{boot}-{name}')
    finally:
        assert source.read_bytes()==original, 'source changed during replay'
    print(f'PASS: saved disk copy across cold reboot; artifacts: {output}')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path)
    parser.add_argument('--depth',type=int,choices=(1,2,3,4),default=3)
    parser.add_argument('--timeout',type=float,default=240)
    args=parser.parse_args()
    run(args.source.resolve(),args.depth,args.timeout)
