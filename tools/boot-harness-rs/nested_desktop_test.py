"""Drive the ordinary installed nested desktop, including its own saved user."""
from pathlib import Path
import shutil

def exercise(output, click, key, scan, width, height, reboot):
    font=bytes.fromhex((Path(__file__).resolve().parents[2]/'source-rs/platform/libraries/bitmap-font/font.hex').read_text())
    def pixel(w,d,x,y): return d[(y*w+x)*3:(y*w+x+1)*3]
    def text(w,d,x,y,value):
        for i,ch in enumerate(value):
            for row,bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (pixel(w,d,x+i*12+col*2,y+row*2)==b'\xff\xff\xff') != bool(bits & (1<<(4-col))): return False
        return True
    def label(x,y,value): scan(lambda w,d:text(w,d,x,y,value),value)
    def note(value,dirty=False):
        scan(lambda w,d:pixel(w,d,469,58)==bytes.fromhex('9b7028' if dirty else '27664a') and text(w,d,474,68,value),f'inner Notes {value!r}')
    click(120,456) # Outer Nested tile.
    if not reboot:
        label(568,412,'NEW USER')
        click(616,384)
    label(504,412,'USER 1')
    shutil.copyfile(output/'display.png',output/'nested-picker.png')
    click(540,384)
    label(322,148,'NOTES')
    label(316,268,'COLORS')
    label(316,388,'NESTED')
    click(352,120)
    note('inner' if reboot else '')
    if not reboot:
        click(470,168)
        for ch in 'inner': key(ch)
        note('inner',True)
        click(190,544) # Outer chrome asks the whole nested subtree about loss.
        label(30,696,'UNSAVED WORK')
        note('inner',True)
        click(120,96) # Unfunded sibling cannot destroy the running subtree.
        label(30,696,'NO ROOM')
        note('inner',True)
        click(470,168);key('f1');note('inner')
    # Keep Notes alive while another leaf requests funding through both owners.
    click(352,240)
    scan(lambda w,d:pixel(w,d,469,58)==bytes.fromhex('d18b35' if reboot else '315ca8'),'inner Colors')
    # A third leaf does not fit the surface grant. Repeated refusal must leave
    # both live apps usable, then closing one must allow admission again.
    for _ in range(3):
        click(352,120)
        label(262,672,'NO ROOM')
    click(320,568) # Inner running-app row: Notes.
    note('inner')
    click(962,58)
    scan(lambda w,d:pixel(w,d,469,58)==bytes.fromhex('d18b35' if reboot else '315ca8'),'Colors survived sibling close')
    if not reboot:
        click(470,168);key('ret')
        scan(lambda w,d:pixel(w,d,469,58)==bytes.fromhex('d18b35'),'saved inner color')
    click(962,58)
    click(352,120)
    note('inner')
    click(962,58)
    click(352,360) # Same Nested app is present; another level exceeds this grant.
    label(316,388,'NESTED')
    label(262,672,'NO ROOM')
    scan(lambda w,d:pixel(w,d,469,58)==bytes.fromhex('101827'),'bounded nesting refusal preserves desktop')
    shutil.copyfile(output/'display.png',output/'nested-desktop.png')
    click(190,544)
    scan(lambda w,d:pixel(w,d,237,34)==bytes.fromhex('101827'),'nested subtree closed')
    click(120,96)
    scan(lambda w,d:pixel(w,d,237,34)==bytes.fromhex('27664a') and not text(w,d,242,44,'inner'),'outer Notes never sees inner save')
    click(width-38,34)
    traces=(output/'serial.log').read_text(errors='replace')
    if 'STORAGE_TRACE matrix-budget' in traces:
        assert 'funding-granted' in traces, 'nested app launch must exercise owner-mediated growth'
    print('PASS: nested desktop, concurrent Notes/Colors, recursive funding, bounded refusal, reclamation and isolation')
