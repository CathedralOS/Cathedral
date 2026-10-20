"""Host-observed desktop latency and serialized storage-provider phase samples.

Wall observations include QMP, fixture input spacing and screenshot polling.
Guest ticks (nominally 100 Hz) and charged CPU are separate clocks; never subtract
them from host time or attribute their difference solely to the scheduler.
"""
from contextlib import contextmanager
import json
import re
from storage_timing import provider_samples as storage_samples
import time


def redraw_samples(serial):
    pending, samples = None, []
    for phase, ticks, cpu in re.findall(r'INPUT_TRACE compositor-redraw-(begin|end) (\d+) (\d+)', serial):
        ticks, cpu = int(ticks), int(cpu)
        if phase == 'begin':
            pending = ticks, cpu
        elif pending is not None:
            if ticks >= pending[0] and cpu >= pending[1]:
                samples.append(dict(guest_ticks=ticks-pending[0], cpu_us=cpu-pending[1]))
            pending = None
    return samples


def input_samples(serial):
    """Ordered observations only; matrix CPU clocks belong to different tasks.

    A typing step injects one key, so source -> Notes ticks measure delivery
    along that route. No global task identity or event identifier is inferred.
    """
    return [dict(stage=stage, guest_tick=int(ticks), account_cpu_us=int(cpu))
            for stage, ticks, cpu in re.findall(
                r'INPUT_TRACE (source-key|matrix-key|notes-key|notes-paint-begin|notes-paint-end) (\d+) (\d+)', serial)]


class Timings:
    def __init__(self, output, reboot):
        self.output = output
        self.path = output / f'boot-{2 if reboot else 1}-latency.json'
        self.samples = []

    @contextmanager
    def step(self, name):
        serial = self.output / 'serial.log'
        offset = serial.stat().st_size
        start = time.perf_counter()
        completed = False
        try:
            yield
            completed = True
        finally:
            elapsed = round((time.perf_counter() - start) * 1000, 1)
            with serial.open('rb') as stream:
                stream.seek(offset)
                text = stream.read().decode(errors='replace')
            self.samples.append(dict(name=name, observed_ms=elapsed, completed=completed,
                                     storage=storage_samples(text), redraws=redraw_samples(text),
                                     input=input_samples(text)))
            self.path.write_text(json.dumps(dict(
                measurement='Host upper bounds including QMP, input spacing and screenshot polling; guest ticks nominally 100 Hz; trace logging perturbs execution.',
                steps=self.samples), indent=2))
            print(f'Desktop latency: {name} {elapsed} ms (observed upper bound)', flush=True)


def report(output):
    for path in sorted(output.glob('boot-*-latency.json')):
        print(path.name)
        for step in json.loads(path.read_text())['steps']:
            provider = [s for s in step['storage'] if s['phase'] == 'provider']
            redraws = step.get('redraws', [])
            rendering = f"{sum(s['cpu_us'] for s in redraws)/1000:.1f} ms" if redraws else 'no samples'
            print(f"  {step['name']}: {step['observed_ms']} ms; "
                  f"provider CPU {sum(s['cpu_us'] for s in provider)/1000:.1f} ms ({len(provider)} spans); "
                  f"sectors R/W {sum(s.get('read_sectors', 0) for s in provider)}/"
                  f"{sum(s.get('written_sectors', 0) for s in provider)}; "
                  f"redraw CPU {rendering}; "
                  f"completed={step['completed']}")


if __name__ == '__main__':
    import argparse
    from pathlib import Path
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifact_directory', type=Path)
    report(parser.parse_args().artifact_directory)
