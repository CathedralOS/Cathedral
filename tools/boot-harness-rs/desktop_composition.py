"""The stock desktop's Omega matrix composition, shared by F5 and regressions.
The Rust kernel and providers stay selected by the distribution profile; root,
user and nested stock matrices all use the same standalone Omega executable.
"""
from copy import deepcopy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STOCK = ROOT / 'source-rs/distribution/profile.json'


def omega_matrix(stock, rust_user=False):
    """Replace only stock matrix images, preserving the caller's source profile."""
    composition = deepcopy(stock)
    composition['name'] = 'Cathedral desktop over the Omega stock matrix'
    launches = composition['startup']['launches']
    replaced = 0
    for index, launch in enumerate(launches):
        if launch.get('package') == 'cathedral-stock-matrix':
            launches[index] = {'target': launch['target'], 'clock': True,
                               'entry': 'distribution/matrix/native/main.omg',
                               'ipc_channels': launch['ipc_channels'],
                               'private_pages': launch['private_pages'],
                               'shared_pages': launch['shared_pages']}
            replaced += 1
    if replaced != 1:
        raise ValueError('the stock composition must launch one root matrix')
    # The stock realm as F5 installs it: the user template is object three,
    # with its launcher, object four, Notes and Colors beneath it. The Omega
    # program takes the template's place, so a user made from it runs the
    # Omega matrix over the user's realm, which configures itself from the
    # Matrix record the installer puts beside the launcher.
    programs = composition['startup']['programs']
    template = programs[2]
    if template.get('package') != 'cathedral-stock-matrix' or template.get('inside') != 1:
        raise ValueError('object three must be the stock user template')
    if rust_user:
        # The unprofiled template asks for the owner's entire large offer.
        # Use the existing bounded desktop declaration beneath the Omega root.
        template.setdefault('features', []).append('desktop-host')
    else:
        programs[2] = {'package': template['package'], 'entry': 'distribution/matrix/native/main.omg',
                       'target': template['target'], 'inside': template['inside'],
                       'display_title': template['display_title']}
    # Every stock matrix F5 installs in the realm, the nested desktops and
    # their user templates, is the Omega program too, so the store shares one
    # image's chunks between them as the swapped distribution will.
    realm = composition['startup']['realm_programs']
    for index, entry in enumerate(realm):
        if not rust_user and entry.get('package') == 'cathedral-stock-matrix':
            realm[index] = {key: value for key, value in entry.items() if key not in ('features', 'entry')}
            realm[index]['entry'] = 'distribution/matrix/native/main.omg'
    return composition


def write(output, composition):
    output.mkdir(parents=True, exist_ok=True)
    profile = output / f'omega-desktop-{output.name}.json'
    profile.write_text(json.dumps(composition, indent=2), encoding='utf-8', newline='\n')
    return profile


def compose(output, rust_user=False):
    stock = json.loads(STOCK.read_text(encoding='utf-8'))
    return write(output, omega_matrix(stock, rust_user=rust_user))
