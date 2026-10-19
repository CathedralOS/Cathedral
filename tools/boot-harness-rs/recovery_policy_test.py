"""Run the desktop's restart-budget policy tests with its real contract constants."""
import json
import os
import subprocess
import run as harness


def run():
    target = harness.BUILD / 'recovery-policy-cargo'
    result = subprocess.run(['cargo', 'build', '--locked', '-p', 'cathedral-platform-contracts',
                             '--config', 'profile.dev.panic="unwind"', '--target-dir', str(target),
                             '--message-format=json'], cwd=harness.WORKSPACE,
                            check=True, capture_output=True, text=True)
    artifacts = [json.loads(line) for line in result.stdout.splitlines() if line.startswith('{')]
    library = next(path for artifact in artifacts
                   if artifact.get('reason') == 'compiler-artifact'
                   and artifact['target']['name'] == 'cathedral_platform_contracts'
                   for path in artifact['filenames'] if path.endswith('.rlib'))
    program = harness.BUILD / ('recovery-policy.exe' if os.name == 'nt' else 'recovery-policy')
    subprocess.run(['rustc', '--edition', '2024', '--test',
                    str(harness.WORKSPACE / 'distribution/desktop/session/persistence/policy.rs'),
                    '--extern', 'cathedral_platform_contracts=' + library,
                    '-L', 'dependency=' + str(target / 'debug/deps'), '-o', str(program)],
                   cwd=harness.WORKSPACE, check=True)
    subprocess.run([str(program)], check=True)
    subprocess.run(['rustc', '--edition', '2024', '--test',
                    str(harness.WORKSPACE / 'distribution/matrix/service/parent/recovery/authority.rs'),
                    '-o', str(program)], cwd=harness.WORKSPACE, check=True)
    subprocess.run([str(program)], check=True)


if __name__ == '__main__':
    run()
