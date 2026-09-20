#!/usr/bin/env python3
"""Reject deliberately wrong expectations inside each semantic fixture body."""
import argparse
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
CASES = [
    ('scalars', 'main.omg', 'boolean_from_bool(true).raw == 1', 'boolean_from_bool(true).raw == 0'),
    ('tables', 'main.omg', 'BOOTSERVICES_RESERVED_SLOT == 17', 'BOOTSERVICES_RESERVED_SLOT == 18'),
    ('storage', 'main.omg', 'FILE_ATTRIBUTE_VALID_ATTR.raw == 0x37', 'FILE_ATTRIBUTE_VALID_ATTR.raw == 0x36'),
    ('console', 'fixtures.omg', 'device_path_length(&header) == 4', 'device_path_length(&header) == 5'),
    ('network', 'main.omg', 'network_statistic_available(0) &&', '!network_statistic_available(0) &&'),
    ('hii', 'main.omg', 'hii_package_length(0xdf000004) == 4', 'hii_package_length(0xdf000004) == 5'),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent / 'Omega/target/release/omega')
    args = parser.parse_args()
    for name, filename, before, after in CASES:
        fixture = ROOT / ('tools/ports/uefi-' + name)
        with tempfile.TemporaryDirectory(prefix='cathedral-behavior-negative-') as directory:
            target = Path(directory)
            for source in fixture.glob('*.omg'):
                text = source.read_text()
                if source.name == filename:
                    if text.count(before) != 1:
                        raise ValueError(name + ': expected one mutation anchor')
                    text = text.replace(before, after)
                if source.name == 'build.omg':
                    text = re.sub(r'location: "([^"]+)"', lambda match: 'location: "' + str((fixture / match[1]).resolve()) + '"', text)
                (target / source.name).write_text(text)
            result = subprocess.run([str(args.omega.resolve()), '--check', str(target / 'main.omg')], cwd=ROOT,
                                    text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            if result.returncode == 0 or 'cannot prove requires contract' not in result.stdout or '1 == 0' not in result.stdout:
                raise ValueError(name + ': wrong expected behavior did not reject computed test result1:\n' + result.stdout)
        print(name + ': wrong behavior expectation produced test failure1; unchanged success assertion rejected it', flush=True)


if __name__ == '__main__':
    main()
