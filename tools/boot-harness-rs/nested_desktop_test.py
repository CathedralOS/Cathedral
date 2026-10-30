"""Recursive ordinary desktops: nested logins, persistence and reclamation."""
from pathlib import Path
import shutil
import time


class Nested:
    """Coordinates and screen checks shared by every recursive-desktop workload.

    Every nested user matrix has the same 8px inset and 80px taskbar, so a
    depth selects one viewport. Checks compare exact glyphs and status colors.
    """

    def __init__(self, click, key, scan, width, height, pointer_stress=False):
        self.click, self.key, self.scan = click, key, scan
        self.width, self.height, self.pointer_stress = width, height, pointer_stress
        self.font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())

    @staticmethod
    def pixel(w, data, x, y):
        return data[(y*w+x)*3:(y*w+x+1)*3]

    def text(self, w, data, x, y, value):
        for i, ch in enumerate(value):
            for row, bits in enumerate(self.font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                for col in range(5):
                    if (self.pixel(w,data,x+i*12+col*2,y+row*2)==b'\xff\xff\xff') != bool(bits & (1 << (4-col))):
                        return False
        return True

    def label(self, x, y, value):
        self.scan(lambda w,d: self.text(w,d,x,y,value), value)

    def viewport(self, depth):
        return (8*depth, 8*depth, self.width-16*depth, self.height-104*depth)

    def launch(self, depth, index):
        x,y,w,h = self.viewport(depth)
        self.click(x+40+56*index, y+h-48)

    def task_close(self, depth, index=0):
        x,y,w,h = self.viewport(depth)
        self.click(x+376+(56 if depth == 0 else 0)+144*index, y+h-48)

    def app_close(self, depth):
        x,y,w,h = self.viewport(depth)
        self.click(x+w-22, y+18)

    def focus_note(self, depth):
        self.click(8*depth+13, 144)

    def desktop(self, depth):
        x,y,w,h = self.viewport(depth)
        # Keep the scan's motion running after the desktop first becomes visible.
        # Startup-only input misses failures while an idle subtree is already live.
        visible_since = None
        def visible(sw, data):
            nonlocal visible_since
            shown = self.pixel(sw,data,x+148,y+h-48)==bytes.fromhex('c7a2ed') and self.text(sw,data,x+w-150,y+h-32,'SETTINGS')
            if not shown:
                visible_since = None
                return False
            if not self.pointer_stress:
                return True
            if visible_since is None:
                visible_since = time.monotonic()
            return time.monotonic() - visible_since >= 3
        self.scan(visible, f'depth {depth} taskbar')

    def enter(self, depth, create):
        started = time.monotonic()
        self.launch(depth-1, 3 if depth==1 else 2)
        x,y,w,h = self.viewport(depth)
        top = y+(h-96)//2
        if create:
            left = x+(w-128)//2
            self.label(left+16,top+76,'NEW USER')
            self.click(left+64,top+48)
        left = x+(w-280)//2
        self.label(left+28,top+76,'USER 1')
        self.click(left+64,top+48)
        self.desktop(depth)
        print(f'Nested desktop depth {depth} ready after {time.monotonic()-started:.2f}s', flush=True)

    def note(self, depth, value, dirty=False):
        x,y,w,h = self.viewport(depth)
        self.scan(lambda sw,d: self.pixel(sw,d,x+13,y+18)==bytes.fromhex('9b7028' if dirty else '27664a') and self.text(sw,d,x+18,y+28,value),f'depth {depth} Notes {value!r}')

    def color(self, depth, saved):
        x,y,w,h = self.viewport(depth)
        self.scan(lambda sw,d: self.pixel(sw,d,x+13,y+18)==bytes.fromhex('d18b35' if saved else '315ca8')
                  and self.pixel(sw,d,x+40,y+50)==bytes.fromhex('e5e7eb'), f'depth {depth} Colors saved={saved}')

    def background(self, depth):
        x,y,w,h = self.viewport(depth)
        # Clearing a surface precedes child collection. Every nested background
        # has the same color, so it cannot prove that a subtree was stopped.
        # Wait for the owner's first running-task cell to become empty as well.
        self.scan(lambda sw,d: self.pixel(sw,d,x+13,y+18)==bytes.fromhex('101827')
                  and self.pixel(sw,d,x+249+(56 if depth == 0 else 0),y+h-60)==bytes.fromhex('101827'),f'depth {depth} closed and collected')


def exercise(output, click, key, scan, width, height, reboot, pointer_stress=False, interaction_delay=False, nested_depth=3):
    n = Nested(click, key, scan, width, height, pointer_stress)
    launch, task_close, app_close, enter, note, background = n.launch, n.task_close, n.app_close, n.enter, n.note, n.background

    def delayed(trigger, action, markers):
        offset = len((output/'serial.log').read_text(errors='replace'))
        key(trigger)
        action()
        def resumed(_width, _data):
            log = (output/'serial.log').read_text(errors='replace')[offset:]
            assert 'matrix-service-exit' not in log and 'Cathedral user panic:' not in log, log
            return all(marker in log for marker in ('interaction-delay-released', *markers))
        scan(resumed, f'{trigger} delayed reply recovered: {markers}')

    for depth in range(1,nested_depth+1):
        enter(depth, not reboot)
    shutil.copyfile(output/'display.png', output/f'{nested_depth}-nested-desktops.png')
    log = (output/'serial.log').read_text(errors='replace')
    # Each level admits a workspace and then its user matrix, in addition to the
    # outer user. Their input wakes must not quietly fall back to positional queues.
    assert log.count('Cathedral matrix: brokered startup and notification input wake ready') >= 2*nested_depth+1, log[-4000:]
    assert log.count('Cathedral matrix: shared owner control ready') >= 2*nested_depth+1, log[-4000:]
    assert 'matrix-control-open-refused' not in log, log[-4000:]
    launch(nested_depth,0)
    note(nested_depth,'deep' if reboot else '')
    if not reboot:
        n.focus_note(nested_depth)
        for ch in 'deep': key(ch)
        note(nested_depth,'deep',True)
    if interaction_delay:
        n.focus_note(nested_depth)
        delayed('f8', lambda: None, ('matrix-poll-delayed', 'matrix-poll-local', 'matrix-poll-resumed'))
        note(nested_depth,'deep',not reboot)
        # Lateness is not input loss: subsequent keys must retain this focus.
        delayed('f7', lambda: key('x'), ('shared-call-delayed', 'shared-call-resumed'))
        note(nested_depth,'deepx',True)
        # On cold open, deleting the added x restores the confirmed document.
        # On the fresh boot, deep itself has not been saved yet.
        key('backspace'); note(nested_depth,'deep',not reboot)
        if reboot:
            key('f1'); note(nested_depth,'deep')
    if not reboot:
        task_close(0) # Loss propagates through the whole subtree.
        n.label(width-192,height-56,'UNSAVED WORK')
        note(nested_depth,'deep',True)
        n.focus_note(nested_depth); key('f1'); note(nested_depth,'deep')
    app_close(nested_depth); background(nested_depth)
    if interaction_delay:
        offset = len((output/'serial.log').read_text(errors='replace'))
        def during_commit():
            launch(nested_depth,1)
            scan(lambda _w,_d: 'interaction-delay-held' in (output/'serial.log').read_text(errors='replace')[offset:], 'held presentation')
            key('a')
            def input_arrived(_w, _d):
                log = (output/'serial.log').read_text(errors='replace')[offset:]
                assert 'interaction-delay-released' not in log, 'input did not progress until presentation released'
                realms = {line.split('matrix-input-during-present ')[1].split()[0] for line in log.splitlines() if 'matrix-input-during-present ' in line}
                return len(realms) >= 2*nested_depth+1
            scan(input_arrived, 'fresh input delivered while presentation is held')
        delayed('f6', during_commit, ('matrix-present-delayed', 'matrix-present-resumed'))
    else:
        launch(nested_depth,1)
    # This is the installed Omega app, through the same recursive services as
    # Notes. Prove its durable bytes survive teardown and cold-open at this scope.
    n.color(nested_depth, reboot)
    if not reboot:
        x,y,w,h = n.viewport(nested_depth)
        click(x+13,y+70); key('ret'); n.color(nested_depth, True)
    app_close(nested_depth); background(nested_depth)
    # Collapse each level, then re-enter the subtree. Old child accounts, image
    # readers and endpoints must be reusable without rebooting the outer desktop.
    for _ in range(2):
        task_close(0); background(0)
        for depth in range(1,nested_depth+1): enter(depth,False)
        launch(nested_depth,0); note(nested_depth,'deep'); app_close(nested_depth); background(nested_depth)
    log = (output/'serial.log').read_text(errors='replace')
    assert 'matrix-control-open-refused' not in log, log[-4000:]
    task_close(0); background(0)
    launch(0,0); note(0,'') # Inner saved content never leaks to the outer realm.
    if interaction_delay:
        # F4 travels past the root and arms the outer desktop's matrix: the
        # close that follows, Notes from the taskbar (a window's own control is
        # the app leaving by itself), counts that matrix's storage as busy for
        # two seconds, past the answer's deadline, and Notes learns of it only
        # when that ends and answers at once, inside the grace the held deadline
        # keeps. That answer must be accepted, not refused.
        offset = len((output/'serial.log').read_text(errors='replace'))
        delayed('f4', lambda: task_close(0), ('interaction-delay-held',))
        background(0)
        assert 'matrix-close-refused' not in (output/'serial.log').read_text(errors='replace')[offset:], 'a clean close answered after the storage hold was refused'
    else:
        app_close(0); background(0)
    launch(0,1); n.color(0, False); app_close(0); background(0)
    print(f'PASS: {nested_depth} recursive desktops, protected close, saved Notes/Omega Colors, repeated subtree reclamation and isolation')
