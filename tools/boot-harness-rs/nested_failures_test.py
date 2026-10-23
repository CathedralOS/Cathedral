"""Fault only Nested's installed programs; outer apps and grants must survive."""
import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
import shutil
import subprocess
import sys
import capture
import database_image
import desktop_test
import disk_image
import elf_trim
import run as harness


def check(mode, output, click, key, scan, width, height, reboot):
    font = bytes.fromhex((harness.WORKSPACE/'platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w,d,x,y): return d[(y*w+x)*3:(y*w+x+1)*3].hex()
    def text(w,d,x,y,value):
        return all((pixel(w,d,x+i*12+c*2,y+r*2)=='ffffff') == bool(bits & (1<<(4-c)))
                   for i,ch in enumerate(value) for r,bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]) for c in range(5))
    def log(): return (output/'serial.log').read_text(errors='replace')
    def label(x,y,value): scan(lambda w,d:text(w,d,x,y,value),value)
    def draft(): scan(lambda w,d:pixel(w,d,13,18)=='9b7028' and text(w,d,18,28,'z'),'outer draft intact')
    click(40,height-48)
    scan(lambda w,d:pixel(w,d,13,18)=='27664a','outer Notes')
    click(13,128); key('z'); draft()
    for cycle in range(2):
        click(208,height-48)
        if mode=='picker':
            scan(lambda w,d:log().count('picker retry exhausted')==cycle+1 and pixel(w,d,13,18)=='9b7028','local picker retries exhausted')
            assert log().count('deliberate picker startup failure')==2*(cycle+1)
            draft()
            continue
        # Source is a three-level fixture with one saved user at each level.
        left=8+(width-16-280)//2; top=8+(height-104-96)//2
        label(left+28,top+76,'USER 1')
        for attempt in range(3 if mode=='user' else 1):
            click(left+64,top+48)
            if mode=='user':
                scan(lambda w,d:log().count('local picker restored')==cycle*3+attempt+1 and text(w,d,8+(width-16-180)//2,8+(height-104)//2+88,'CANNOT OPEN'),'fresh local picker after selected-user failure')
            else:
                scan(lambda w,d:log().count('background retry consumed')==cycle+1 and pixel(w,d,40,height-152)=='d6e5ef','background retry leaves inner chrome usable')
                assert log().count('deliberate background startup failure')==2*(cycle+1)
                click(104,height-144)
                scan(lambda w,d:pixel(w,d,21,26) in ('315ca8','d18b35'),'inner Colors after background failure')
        # Outer owner closes just this subtree; its other child retains a draft.
        click(520,height-48) # second task's X (slot two)
        draft()
    assert log().count('note-loaded') <= 1
    print(f'PASS: Nested {mode} failures, bounded local recovery and repeated reclamation; outer draft survives')


def run(source):
    before=source.read_bytes()
    output=harness.BUILD/'nested-failures'/datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile=output/'nested-failures.json'
    shutil.copyfile(harness.WORKSPACE/'distribution/profile.json',profile)
    subprocess.run([sys.executable,str(Path(__file__).with_name('run.py')),'--release','--build-only','--profile',str(profile)],check=True)
    specs=[('picker','cathedral-chooser','picker-startup-fault','Nested/Login','deliberate picker startup failure'),
           ('user','cathedral-stock-matrix','startup-fault','Nested/Users/User 1','deliberate matrix startup failure'),
           ('background','cathedral-background','startup-fault','Nested/Users/User 1/Background','deliberate background startup failure')]
    oracle=database_image.oracle()
    for mode,package,feature,target,panic in specs:
        subprocess.run(['cargo','build','--locked','--release','--target','x86_64-unknown-none','--target-dir',str(harness.BUILD/'cargo'),'--package',package,'--features',feature],cwd=harness.WORKSPACE,check=True)
        fault=output/(mode+'.elf')
        elf_trim.trim(harness.BUILD/'cargo/x86_64-unknown-none/release'/package,fault,None)
        folder=output/mode; folder.mkdir()
        shutil.copytree(harness.BUILD/'nested-failures-interactive/esp',folder/'esp')
        image=folder/'guest.raw';shutil.copyfile(source,image)
        subprocess.run([str(oracle),'engines','volume','failure-fixture',str(image),str(database_image.OFFSET),str(database_image.VOLUME_BYTES),'path:'+target,str(fault)],check=True)
        qemu=harness.find_qemu()
        command=[qemu,'-machine','q35','-cpu','qemu64','-accel','tcg','-smp','1','-m','128']+harness.firmware_args(qemu,folder)
        command+=['-drive','format=raw,file=fat:rw:esp','-nic','none','-monitor','none','-no-reboot','-display','none']+disk_image.arguments(image)
        capture.run(command,folder,240,subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0,
                    'Cathedral desktop: ready; choose a user',
                    lambda stream,out,serial:desktop_test.desktop(stream,out,serial,True,user_check=lambda *args:check(mode,*args),allowed_panics=(panic,)))
    assert source.read_bytes()==before
    print(f'PASS: nested startup failures; artifacts: {output}')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image',type=Path,help='three-level nested desktop fixture; copies only')
    run(parser.parse_args().image)
