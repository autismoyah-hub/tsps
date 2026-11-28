"""Gate proof: the converted original (ceiling) must PASS, every damaged copy must FAIL.

Private fixtures go to private-proof/<kind>-<id>/ (Teacher only). usage: rs3proof.py [KIND:ID ...]
"""
import sys, json, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import rs3gate
from rs3gate import Reference
from hdgate import np, combined, shares, area, hdview
import hdasset
from rs3asset import REV, target, conversion, _swatch

TARGETS = [('item-ground', 47470), ('item-worn', 47470), ('item-female', 47470),('item-ground', 53375), ('item-worn', 53375), ('item-female', 53375),
           ('npc-part1', 28070), ('npc-part2', 28070), ('npc-part3', 28070), ('npc-part4', 28070)]

def controls(ref):
    m = combined(ref.converted)
    yield 'ceiling', m
    points = np.asarray(m['vertices']).copy()
    select = points[:, 0] > np.median(points[:, 0])
    points[select, 0] += max(12, np.ptp(points[:, 0])//3)
    yield 'right half displaced outward', dict(m, vertices=points.tolist())
    ts = np.array(ref.r['face_textures']); aa = np.array(ref.r['alpha'])
    if (ts >= 0).any():
        share = shares(ts, area(ref.r))
        t = max((k for k in share if k >= 0), key=share.get)
        damaged = np.array(m['face_textures']); damaged[ts == t] = -1
        yield f'material {t} region stripped', dict(m, face_textures=damaged.tolist())
        sig = hdasset.signature(_swatch(t)[0]); cat = hdasset.catalogue()
        worst = max(cat, key=lambda k: np.abs(sig-cat[k]).mean())
        yield f'material {t} region painted with distant 242 texture {worst}', dict(m, face_textures=np.where(ts == t, worst, m['face_textures']).tolist())
    if aa.any():
        yield 'transparency removed', dict(m, alpha=[0]*len(aa))
    else:
        yield 'transparency introduced', dict(m, alpha=[160]*len(aa), has_alpha=True)
    yield 'paint shifted (hue +16, all faces)', dict(m, colors=[((c >> 10) + 16) % 64 << 10 | (c & 1023) for c in m['colors']])
    if len(ref.parts) > 1:
        parts = list(ref.converted)
        centers = [np.mean(p['vertices'], axis=0).round().astype(int) for p in parts[:2]]
        for j, p in enumerate(parts[:2]):
            parts[j] = dict(p, vertices=(np.asarray(p['vertices'])+centers[1-j]-centers[j]).tolist())
        yield 'first two assembly parts exchanged positions', combined(parts)
    if ref.kind.startswith('item'):
        other = 'item-worn' if ref.kind == 'item-ground' else 'item-ground'
        d, p = target(REV, other, ref.number)
        yield 'swapped entire presentation with ' + other, combined(conversion(REV, d, p)[0])

def main():
    wanted = sys.argv[1:]
    summary = []
    for kind, i in TARGETS:
        if wanted and f'{kind}:{i}' not in wanted: continue
        ref = Reference(REV, kind, i); ref.number = i
        folder = Path(__file__).resolve().parent/'private-proof'/f'{kind}-{i}'
        folder.mkdir(parents=True, exist_ok=True)
        (folder/'mapping.json').write_text(json.dumps({str(k): v for k, v in ref.mapping.items()}, indent=2))
        for name, m in controls(ref):
            path = folder/(name+'.bin')
            path.write_bytes(hdview.packed([m]))
            q = ref.score(path, folder/('view-'+name) if name == 'ceiling' else None)
            lines = q.lines()
            (path.with_suffix('.txt')).write_text('\n'.join(lines)+'\n')
            fails = [l for l in lines if l.endswith('| FAIL')]
            summary.append(dict(target=f'{kind}:{i}', fixture=name, verdict=lines[-1], failing_lines=fails[:6]))
            print(kind, i, name, lines[-1], flush=True)
            ok = (name == 'ceiling') == (q.severity == 0) and (name == 'ceiling' or q.severity == 2)
            summary[-1]['expected'] = ok
    out = Path(__file__).resolve().parent/'gate-proof.json'
    prev = json.loads(out.read_text()) if out.exists() and wanted else []
    out.write_text(json.dumps([s for s in prev if s['target'] not in {x['target'] for x in summary}] + summary, indent=1))
    bad = [s for s in summary if not s['expected']]
    print('unexpected:', len(bad))
    for s in bad: print(s)

if __name__ == '__main__': main()
