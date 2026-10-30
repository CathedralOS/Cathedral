"""Hold Background's actual broker while checking independent owner progress."""
import argparse
from datetime import datetime, timezone
from pathlib import Path
import subprocess

from build import ROOT, prepare
from run_test import boot


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    args = parser.parse_args()
    image = prepare(ROOT / 'source/distribution/applications/background/native', args.compiler)
    output = ROOT / 'build/omega-user/background-control' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    cargo = ROOT / 'build/boot-harness-rs/cargo'
    with (output / 'owner-build.log').open('w') as log:
        subprocess.run(['cargo', 'build', '--locked', '--release', '--package', 'cathedral-channels-lab',
                        '--features', 'omega-background', '--target', 'x86_64-unknown-none',
                        '--target-dir', str(cargo)], cwd=ROOT / 'source-rs',
                       stdout=log, stderr=subprocess.STDOUT, check=True)
    owner = cargo / 'x86_64-unknown-none/release/cathedral-channels-lab'
    composition = dict(CATHEDRAL_LAUNCH_COUNT='1', CATHEDRAL_CHANNEL_LIMITS='16,2',
                       CATHEDRAL_CPU_LIMITS='0,0', CATHEDRAL_INIT_PRIVATE_PAGES='8',
                       CATHEDRAL_INIT_SHARED_PAGES='8', CATHEDRAL_LAUNCH_0_ELF=str(image),
                       CATHEDRAL_LAUNCH_0_ARGUMENT='1', CATHEDRAL_LAUNCH_0_CLOCK='1',
                       CATHEDRAL_LAUNCH_0_PRIVATE_PAGES='2', CATHEDRAL_LAUNCH_0_SHARED_PAGES='2')

    def verify(stream, directory, text):
        if 'Omega Background independent control PASS\n' not in text or 'initial program stopped: Some(Returned(0))' not in text:
            raise RuntimeError(f'Background control witness failed: {directory / "serial.log"}')

    boot(owner, output / 'boot', True, 90, composition=composition, verifier=verify)
    print(f'Omega Background independent control PASS: {output}', flush=True)


if __name__ == '__main__':
    main()
