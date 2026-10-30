"""Explicit Rust witnesses for mechanisms not implemented by standalone apps.

Only fixture setup calls these selectors. Normal builds never substitute a
Rust host for a selected Omega executable.
"""


def notes(program, feature):
    if program.get('package') != 'cathedral-notes':
        raise ValueError('Notes probe requires a Notes program entry')
    if feature not in {
        'render-demand-probe', 'capture-source-exit', 'capture-source-stall',
        'input-receiver-probe', 'input-producer-partial', 'input-producer-probe',
        'control-delay-probe', 'render-probes',
    }:
        raise ValueError(f'Unknown Rust Notes probe: {feature}')
    program['entry'] = 'distribution/applications/notes/main.rs'
    program.setdefault('features', []).append(feature)
