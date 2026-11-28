"""File the owner's saved answers for a sitting into VERDICTS_DIR, words unchanged.

usage: python tools/collect_sitting.py SITTING_ID [--downloads DIR]
Takes the newest sitting-<id>-answers*.json from Downloads and writes VERDICTS_DIR/sitting-<id>.json.
If a filed copy exists and differs, the new one is written beside it with its save time, never over it.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse, json, re
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument('sitting'); ap.add_argument('--downloads', type=Path, default=Path.home() / 'Downloads')
ap.add_argument('--out', type=Path, default=Path(_AP.get('VERDICTS_DIR')))
a = ap.parse_args()
pat = re.compile(r'sitting-' + re.escape(a.sitting) + r'-answers(?: \(\d+\))?\.json')
files = sorted((p for p in a.downloads.iterdir() if pat.fullmatch(p.name)), key=lambda p: p.stat().st_mtime)
if not files:
    ap.exit(1, 'No saved answers yet.\n')
data = json.loads(files[-1].read_text(encoding='utf-8-sig'))
if data.get('sitting') != a.sitting:
    ap.error('the file is for another sitting')
target = a.out / f'sitting-{a.sitting}.json'
if target.exists() and json.loads(target.read_text(encoding='utf-8')) != data:
    target = a.out / f"sitting-{a.sitting}-{data['saved_at'].replace(':', '').replace('.', '')}.json"
target.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(target.name, f'({len(files)} saved file(s) found; newest taken)')
