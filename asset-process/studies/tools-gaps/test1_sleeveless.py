"""Test of label_armour.py --sleeveless on real sleeveless body items.

For each item: export its worn mesh through our encoder (reencode_item.py, worn space: wear offset and recolours
applied so the picture can be compared with item:ID), set every vertex and face label to 0, relabel with
--sleeveless (and, for contrast, without), then
  - label agreement with the real labels (exact, and same-transform class),
  - pose.py --diff real item against the relabelled copy over walk 819, run 824, slash 390, every frame.
usage: test1_sleeveless.py [ITEM ...]
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, collections, re
T = pathlib.Path(_AP.fix(r'<ROOT>\tools'))
sys.path.insert(0, str(T))
import numpy as np, osrscache, label_armour, reencode_item
PY = [_AP.fix(r'<PYTHON>'), '-I']
OUT = pathlib.Path(__file__).parent / 't1'
OUT.mkdir(exist_ok=True)
SEQS = {819: 8, 824: 8, 390: 10}
items = [int(a) for a in sys.argv[1:]] or [1101, 1129, 1135, 1133, 1131]


def run(*a):
    r = subprocess.run([*PY, *map(str, a)], capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    return r.stdout


for item in items:
    d = reencode_item.definition(item)
    real = OUT / f'{item}-real.bin'; bare = OUT / f'{item}-stripped.bin'
    run(T / 'reencode_item.py', item, real)
    m = osrscache.decode_model(real.read_bytes())
    truth = np.array(m['vertex_skins']); ftruth = np.array(m['face_skins'])
    bare.write_bytes(label_armour.encode(m, [0] * len(m['vertices']), [0] * len(m['faces'])))
    assert set(osrscache.decode_model(bare.read_bytes())['vertex_skins'] or [0]) == {0}
    print(f'\n=== item {item} {d["name"]!r} wear positions {d["wear_positions"]}, {len(truth)} vertices, |x| max {max(abs(v[0]) for v in m["vertices"])}')
    print(f'    true vertex labels {dict(sorted(collections.Counter(truth.tolist()).items()))}  face labels {dict(sorted(collections.Counter(ftruth.tolist()).items()))}')
    for mode, flags in (('sleeveless', ['--sleeveless']), ('old rule', [])):
        dst = OUT / f'{item}-{mode.replace(" ", "")}.bin'
        log = run(T / 'label_armour.py', bare, 'body', dst, '--quiet', *flags)
        g = osrscache.decode_model(dst.read_bytes())
        L = np.array(g['vertex_skins']); FL = np.array(g['face_skins'])
        cls = np.mean([label_armour.BASIC.get(int(a), -1) == label_armour.BASIC.get(int(b), -2) for a, b in zip(L, truth)])
        arm = int(np.isin(L, list(label_armour.ARM_LABELS)).sum()); armf = int(np.isin(FL, list(label_armour.ARM_FACE_LABELS)).sum())
        warn = len(re.findall('WARNING', log))
        print(f'  {mode}: labels {dict(sorted(collections.Counter(L.tolist()).items()))}; exact {100 * (L == truth).mean():.1f}%, same transforms {100 * cls:.1f}%, '
              f'face labels {100 * (FL == ftruth).mean():.1f}%; arm vertex labels {arm}, arm face labels {armf}, WARNING lines {warn}')
        if mode == 'sleeveless':
            assert arm == 0 and armf == 0 and warn == 0 and 'ARMS KIT MUST STAY' in log
        for seq, n in SEQS.items():
            png = OUT / f'{item}-{mode.replace(" ", "")}-{seq}.png'
            o = run(T / 'pose.py', png, f'item:{item},model:{dst.as_posix()}@body,none', '--seq', seq, '--frames', ','.join(map(str, range(n))), '--diff', '1:2')
            counts = [int(x) for x in re.findall(r'frame \d+: (\d+) of', o)]
            assert len(counts) == n
            print(f'     seq {seq}: differing pixels per frame {counts}  (max {max(counts)})')
