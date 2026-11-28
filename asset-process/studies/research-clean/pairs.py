"""Small mirrored feature pairs in a head box (candidates for eyes, nostrils, teeth, horns), strongest contrast first.
usage: pairs.py ENTRY upLo,upHi [zfracLo,zfracHi] [--pic OUT.png]"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl, features as ft

def pairs(c, box, max_size=0.09):
    feats, hgt = ft.find(c)
    feats = [f for f in feats if ft.in_box(f, c, box) and f['size_h'] <= max_size]
    out = []
    for f in feats:
        if f['cen'][0] >= -1.5: continue
        for g in feats:
            if g['colour'] == f['colour'] and g['faces'] in (f['faces'], f['faces'] + 6, f['faces'] - 6) and abs(g['cen'][0] + f['cen'][0]) < 4 and abs(g['cen'][1] - f['cen'][1]) < 4 and abs(g['cen'][2] - f['cen'][2]) < 4:
                out.append(f); break
    out.sort(key=lambda f: -f['contrast'])
    return out, hgt

if __name__ == '__main__':
    a = sys.argv[1:]
    c = cl.load(a[0]); box = [float(x) for x in a[1].split(',')]
    if len(a) > 2 and not a[2].startswith('--'):
        zl, zh = [float(x) for x in a[2].split(',')]
        Vu = c['V'][np.unique(c['F'][c['drawn'] & (c['A'] == 0)])]; z0, z1 = Vu[:, 2].min(), Vu[:, 2].max()
        box = box[:2] + [z0 + zl * (z1 - z0), z0 + zh * (z1 - z0)]
    ps, hgt = pairs(c, box)
    print(f"== {c['name']}  height {hgt:.0f}: mirrored small pairs (one side listed)")
    ft.table(ps[:16])
    if '--pic' in a: ft.picture(c, ps[:16], box, a[a.index('--pic') + 1], yaws=(0, 35, 90), size=520)
