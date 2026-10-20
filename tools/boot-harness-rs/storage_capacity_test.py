"""Full retained quota in the ordinary desktop: saves refuse and drafts survive."""
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
import run as harness


def exercise(output, click, key, scan, width, height, reboot):
    font = bytes.fromhex((harness.WORKSPACE/'platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w,d,x,y): return d[(y*w+x)*3:(y*w+x+1)*3].hex()
    def text(w,d,x,y,value):
        return all((pixel(w,d,x+i*12+c*2,y+r*2)=='ffffff') == bool(bits & (1<<(4-c)))
                   for i,ch in enumerate(value) for r,bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]) for c in range(5))
    def note(error=False):
        scan(lambda w,d: pixel(w,d,13,18)==('993f46' if error else '27664a') and text(w,d,18,28,'hello cathedral'), 'saved text with expected save status')
    click(40,height-48); note()
    click(13,128); key('z')
    for attempt in range(3):
        if attempt:
            key('f1') # Reconcile the old error without discarding the draft.
            scan(lambda w,d:pixel(w,d,13,18)=='9b7028','reconciled draft')
        key('f1'); note(True)
        scan(lambda w,d:text(w,d,18,44,'z'),'unsaved last character preserved')
    # SAVE in app-owned close must refuse too, keeping the editor alive.
    click(width-22,18)
    scan(lambda w,d:text(w,d,18,28,'SAVE ERROR'),'close confirmation')
    click(48,52) # Reconcile, then retry the save explicitly.
    scan(lambda w,d:text(w,d,18,28,'UNSAVED NOTE'),'reconciled close confirmation')
    click(48,52)
    scan(lambda w,d:text(w,d,18,28,'SAVE ERROR'),'failed save stays open')
    click(48,84) # Cancel restores the same draft.
    note(True)
    scan(lambda w,d:text(w,d,18,44,'z'),'draft survives failed close-save')
    # A sibling still opens and reads the old committed revision.
    click(40,height-48); note()
    click(width-22,18); note(True)
    # Explicit reload, not refusal, discards the draft.
    click(13,128);key('f2');note()
    serial=(output/'serial.log').read_text(errors='replace')
    assert serial.count('Cathedral: object storage ready')==1, 'ordinary quota refusal restarted provider'
    print('PASS: repeated full-store save refusal, draft and close protection, sibling reads, explicit reload')


def run(source):
    before=source.read_bytes()
    output=harness.BUILD/'storage-capacity'/datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile=output/'storage-capacity.json'
    shutil.copyfile(harness.WORKSPACE/'distribution/profile.json',profile)
    subprocess.run([sys.executable,str(Path(__file__).with_name('run.py')),'--release','--build-only','--profile',str(profile)],check=True)
    shutil.copytree(harness.BUILD/'storage-capacity-interactive/esp',output/'esp')
    image=output/'guest.raw';shutil.copyfile(source,image)
    oracle=database_image.oracle()
    subprocess.run([str(oracle),'engines','volume','capacity-fixture',str(image),str(database_image.OFFSET),str(database_image.VOLUME_BYTES)],check=True)
    full=output/'full-before.raw';shutil.copyfile(image,full)
    qemu=harness.find_qemu()
    command=[qemu,'-machine','q35','-cpu','qemu64','-accel','tcg','-smp','1','-m','128']+harness.firmware_args(qemu,output)
    command+=['-drive','format=raw,file=fat:rw:esp','-nic','none','-monitor','none','-no-reboot','-display','none']+disk_image.arguments(image)
    for boot in range(2):
        capture.run(command,output,240,subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0,
                    'Cathedral desktop: ready; choose a user',
                    lambda stream,out,serial:desktop_test.desktop(stream,out,serial,True,user_check=exercise))
        shutil.copyfile(output/'serial.log',output/f'boot-{boot+1}-serial.log')
    subprocess.run([str(oracle),'engines','volume','verify-unchanged',str(full),str(image),str(database_image.OFFSET),str(database_image.VOLUME_BYTES)],check=True)
    assert source.read_bytes()==before
    print(f'PASS: full-store desktop and cold reopen, exact records preserved; artifacts: {output}')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image',type=Path,help='completed ordinary desktop fixture; copies only')
    run(parser.parse_args().image)
