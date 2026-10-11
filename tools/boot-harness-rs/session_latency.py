"""Sustained real-device dragging: detect dropped motion, stuck releases and stalls."""
import json
import time
import capture
import pointer_test


def drag(stream, output, move, button, text, wait):
    move(120, 280)
    offset = len(text())
    button(True)
    wait("app=1 pointer a=0 b=1 value=2", offset)
    start = time.monotonic()
    for _ in range(120):
        capture.command(stream, "input-send-event", {"events": [
            {"type": "rel", "data": {"axis": "x", "value": 1}}]})
        time.sleep(0.01)
    button(False)
    sent = time.monotonic()
    while True:
        width, _, pixels = pointer_test.read_scanout(stream, output)
        at = (280 * width + 240) * 3
        if pixels[at:at + 3] == b"\xff\xff\xff":
            break
        if time.monotonic() - sent > 1:
            raise RuntimeError("Sustained drag lost motion or cursor lag exceeded one second")
        time.sleep(0.01)
    elapsed = time.monotonic() - sent
    wait("app=1 pointer a=0 b=0 value=2", offset)
    if "cancel a=0" in text()[offset:] or "recovered" in text()[offset:] or "source offline" in text()[offset:]:
        raise RuntimeError("Ordinary sustained dragging cancelled input or triggered recovery")
    result = {"reports": 120, "send_seconds": sent - start, "final_cursor_visible_after_seconds": elapsed}
    (output / "latency.json").write_text(json.dumps(result, indent=2))
    print(f"Sustained drag: 120 reports, no lost motion; final cursor visible after {elapsed * 1000:.0f} ms", flush=True)
