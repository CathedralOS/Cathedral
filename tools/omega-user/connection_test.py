"""Exercise Omega v4 client and owner server against the Rust transport."""
import argparse
from datetime import datetime, timezone
from pathlib import Path
import subprocess

from build import ROOT, prepare
from run_test import boot


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--timeout', type=int, default=90)
    parser.add_argument('--service', action='store_true', help='exercise Omega multi-client acceptance, attachments and isolation')
    parser.add_argument('--admission', action='store_true', help='exercise Omega child admission, service offers and reclamation')
    args = parser.parse_args()
    client = prepare(ROOT / 'source/distribution/lab/user-entry/native', args.compiler)
    output = ROOT / 'build/omega-user/connections' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    cargo = ROOT / 'build/boot-harness-rs/cargo'
    features = 'omega-admission' if args.admission else 'omega-service' if args.service else 'omega-peer'
    with (output / 'provider-build.log').open('w', encoding='utf-8') as log:
        subprocess.run(['cargo', 'build', '--locked', '--release', '--package', 'cathedral-channels-lab',
                        '--features', features, '--target', 'x86_64-unknown-none',
                        '--target-dir', str(cargo)], cwd=ROOT / 'source-rs',
                       stdout=log, stderr=subprocess.STDOUT, check=True)
    provider = cargo / 'x86_64-unknown-none/release/cathedral-channels-lab'
    if args.admission:
        admission(provider, client, output, args.timeout)
        return
    composition = dict(CATHEDRAL_LAUNCH_COUNT='2', CATHEDRAL_CHANNEL_LIMITS='16,4,3' if args.service else '16,4,2',
                       CATHEDRAL_CPU_LIMITS='0,0,0', CATHEDRAL_INIT_PRIVATE_PAGES='16',
                       CATHEDRAL_INIT_SHARED_PAGES='16',
                       CATHEDRAL_LAUNCH_0_ELF=str(provider), CATHEDRAL_LAUNCH_0_ARGUMENT='1',
                       CATHEDRAL_LAUNCH_1_ELF=str(client), CATHEDRAL_LAUNCH_1_ARGUMENT='2')
    for index in range(2):
        composition.update({f'CATHEDRAL_LAUNCH_{index}_CLOCK': '1',
                            f'CATHEDRAL_LAUNCH_{index}_PRIVATE_PAGES': '1',
                            f'CATHEDRAL_LAUNCH_{index}_SHARED_PAGES': '1'})
    # The client retains its mailbox while staging a 12-page Notes-sized image.
    composition['CATHEDRAL_LAUNCH_1_PRIVATE_PAGES'] = '13'
    composition['CATHEDRAL_LAUNCH_0_SHARED_PAGES'] = '13'
    composition['CATHEDRAL_LAUNCH_0_PRIVATE_PAGES'] = '2'
    composition['CATHEDRAL_LAUNCH_1_SHARED_PAGES'] = '5' if args.service else '2'

    def verify(stream, directory, text):
        markers = ('Omega multi-client service PASS\n', 'initial program stopped: Some(Returned(0))') if args.service else (
                'Omega shared connection PASS\n', 'Omega owner connection PASS\n',
                'Omega service acquisition PASS\n',
                'initial program stopped: Some(Returned(0))')
        if not all(marker in text for marker in markers):
            raise RuntimeError(f'Omega/Rust connection witness failed: {directory / "serial.log"}')

    boot(provider, output / 'boot', True, args.timeout, composition=composition, verifier=verify)
    label = 'Omega multi-client service' if args.service else 'Omega/Rust shared connection'
    print(f'{label} PASS: {output}', flush=True)


def admission(provider, client, output, timeout):
    # The Rust owner lends launches 1 and 2 (Rust peers) to the Omega supervisor at
    # launch 0; the kernel lends only toward lower slots. Its fourteen channels cover
    # two children with storage plus their listeners and the owner bell; a per-child
    # leak exhausts them within the repeated rounds.
    composition = dict(CATHEDRAL_LAUNCH_COUNT='3', CATHEDRAL_CHANNEL_LIMITS='16,14,6,6',
                       CATHEDRAL_CPU_LIMITS='0,0,0,0', CATHEDRAL_INIT_PRIVATE_PAGES='16',
                       CATHEDRAL_INIT_SHARED_PAGES='16',
                       CATHEDRAL_LAUNCH_0_ELF=str(client), CATHEDRAL_LAUNCH_0_ARGUMENT='2',
                       CATHEDRAL_LAUNCH_1_ELF=str(provider), CATHEDRAL_LAUNCH_1_ARGUMENT='1',
                       CATHEDRAL_LAUNCH_2_ELF=str(provider), CATHEDRAL_LAUNCH_2_ARGUMENT='1')
    for index in range(3):
        pages = '4' if index == 0 else '2'
        composition.update({f'CATHEDRAL_LAUNCH_{index}_CLOCK': '1',
                            f'CATHEDRAL_LAUNCH_{index}_PRIVATE_PAGES': pages,
                            f'CATHEDRAL_LAUNCH_{index}_SHARED_PAGES': pages})

    def verify(stream, directory, text):
        markers = ('Omega child admission PASS\n', 'initial program stopped: Some(Returned(0))')
        if not all(marker in text for marker in markers):
            raise RuntimeError(f'Omega child admission witness failed: {directory / "serial.log"}')

    boot(provider, output / 'boot', True, timeout, composition=composition, verifier=verify)
    print(f'Omega child admission PASS: {output}', flush=True)


if __name__ == '__main__':
    main()
