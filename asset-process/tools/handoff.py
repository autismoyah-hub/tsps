"""Shared project paths, lossless inspection of label-stream flags, and tool invocation."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(_AP.fix('<ROOT>'))
TOOLS = ROOT / 'tools'
sys.path.insert(0, str(TOOLS))
import osrscache


def decode(path):
    raw = Path(path).read_bytes()
    model = osrscache.decode_model(raw)
    footer = {'legacy': 18, 'skinned-legacy': 23, 'extended': 23, 'skinned-extended': 26}[model['format']]
    model['face_labels_present'] = raw[-footer + 8] == 1
    if not model['vertices'] or not model['faces']:
        raise ValueError('empty geometry')
    if any(i < 0 or i >= len(model['vertices']) for face in model['faces'] for i in face):
        raise ValueError('triangle index outside vertex array')
    return model


def run(script, *args):
    cmd = [sys.executable, '-I', '-B', str(script), *map(str, args)]
    result = subprocess.run(cmd, capture_output=True, text=True, errors='replace')
    if result.returncode:
        raise RuntimeError(f'{script.name} exit {result.returncode}: {(result.stdout + result.stderr).strip()}')
    return result.stdout


def entry(path, slot):
    path = Path(path).resolve(strict=True).as_posix()
    if any(char in path for char in '@+,'):
        raise ValueError('pose cannot encode @ + or comma in paths')
    return f'model:{path}@{slot.lstrip("@")}'
