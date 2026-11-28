"""Step 2 of research-paint: small details of a real boss against a rebuild, mark by mark.

usage: detail.py CASE [CASE..]      CASE = tekton1 tekton2 tekton3 nex lynx minotaur araxyte   (dlib.OURS)
                                         | ceiling:NPC | allout:NPC | islands10:NPC | islands50:NPC | islands100:NPC
                                         | under:NPC:PX | NAME=NPC:PATH.json
       detail.py --table [TAG..]    print the tables from work/results/*.json
writes work/results/TAG.json and prints the tables.
  ceiling     the real mesh through our encoder
  allout      every mark of the real (strict or loose) painted out
  islandsP    P percent of the real's LOOSE marks (research-tone/damage.py's rule and seed) repainted as their largest neighbour
  under:NPC:PX  only the marks whose largest game-size footprint is under PX pixels painted out
"""
import sys, json, random, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import dlib
from dlib import tl, L

def zone_fn(npc):
    import decomp
    return decomp.regions_for(npc)

def real_set(npc):
    """Real mesh, definition, sequences, its marks, its pictures and the pictures of the real with every mark out."""
    mr, dr, seqs, pr = dlib.real_pictures(npc); ar = tl.arrays(mr)
    _, marks = dlib.mesh_marks(ar, dr['scale'][0])
    m0, n = dlib.paint_out(mr, ar, marks)
    pr0 = dlib.lit_only(f'real0-{npc}', npc, m0, dr, seqs)
    return mr, dr, seqs, ar, marks, pr, pr0

def variant(kind, npc):
    mr, dr, seqs, ar, marks, pr, pr0 = real_set(npc)
    if kind == 'ceiling': mm = tl.clone(mr); note = 'real mesh through our encoder'
    else:
        if kind == 'allout': take = marks
        elif kind.startswith('islands'):
            pct = int(kind[7:]); cand = [mk for mk in marks if mk['loose']]
            rng = random.Random(239); rng.shuffle(cand); take = cand[:max(1, round(len(cand) * pct / 100))]
        else:
            lim = int(kind.split('-')[1]); rec = dlib.compare(ar, marks, pr, pr0, pr, npc, cams=('game',))
            take = [mk for mk, r in zip(marks, rec) if r['game']['max_px'] < lim]
        mm, n = dlib.paint_out(mr, ar, take); note = f'{len(take)} of {len(marks)} marks painted out'
    return tl.write_variant(f'{npc}-{kind}', mm, dr, 'variants'), note

def run(case):
    note = ''
    if '=' in case:
        tag, rest = case.split('=', 1); npc, path = rest.split(':', 1); npc = int(npc)
    elif ':' in case:
        parts = case.split(':'); kind, npc = parts[0], int(parts[1])
        if kind == 'under': kind = f'under-{parts[2]}'
        path, note = variant(kind, npc); tag = f'{kind}-{npc}'
    else:
        npc, path = dlib.OURS[case]; tag = case
    mr, dr, seqs, ar, marks_r, pr, pr0 = real_set(npc)
    mo, jo, po = dlib.ours_pictures(tag, npc, path)
    ao = tl.arrays(mo); sc = dr['scale'][0]
    _, marks_o = dlib.mesh_marks(ao, jo.get('scale', [128, 128])[0])
    mo0, _ = dlib.paint_out(mo, ao, marks_o); po0 = dlib.lit_only(f'ours0-{tag}', npc, mo0, jo, seqs)
    names, zf = zone_fn(npc); zr = zf(ar)
    rec = dlib.compare(ar, marks_r, pr, pr0, po, npc)
    rev = dlib.compare(ao, marks_o, po, po0, pr, npc, cams=('game',))
    def strip(mk, r, zone=None):
        d = {k: (v if not isinstance(v, np.generic) else v.item()) for k, v in mk.items() if k not in ('faces', 'centre')}
        d['centre'] = [float(x) for x in mk['centre']]; d['nfaces'] = int(len(mk['faces']))
        for cn in r:
            if cn != 'inst': d[cn] = r[cn]
        d['game_inst'] = r['inst']['game']
        if zone is not None: d['zone'] = zone
        return d
    out = dict(case=case, tag=tag, npc=npc, path=str(path), note=note, zones=names, scale=sc,
               real=[strip(mk, r, names[int(np.bincount(zr[mk['faces']]).argmax())]) for mk, r in zip(marks_r, rec)],
               ours=[strip(mk, r) for mk, r in zip(marks_o, rev)],
               faces_real=int(tl.drawn(ar).sum()), faces_ours=int(tl.drawn(ao).sum()))
    (dlib.WORK / 'results').mkdir(exist_ok=True)
    (dlib.WORK / 'results' / f'{tag}.json').write_text(json.dumps(out, indent=0))
    return out

def gate_value(o, cam='game'):
    """recall (each visible opaque mark of the real once), count, spurious share of ours' visible marks, count."""
    v = [m[cam]['kept'] for m in o['real'] if not m['translucent'] and m[cam]['kept'] is not None]
    s = [m['game']['kept'] for m in o['ours'] if not m['translucent'] and m['game']['kept'] is not None]
    return (float(np.mean(v)) if v else None), len(v), (1 - float(np.mean(s)) if s else None), len(s)

def tables(o):
    R = [m for m in o['real'] if not m['translucent']]; O = [m for m in o['ours'] if not m['translucent']]
    print(f"\n== {o['case']} (real NPC {o['npc']}) {o['note']}: drawn faces real {o['faces_real']} ours {o['faces_ours']}")
    print(f"   marks on the mesh (opaque): real {len(R)} (strict {sum(m['strict'] for m in R)}, loose {sum(m['loose'] for m in R)});"
          f" ours {len(O)} (strict {sum(m['strict'] for m in O)}, loose {sum(m['loose'] for m in O)});"
          f" translucent marks real {len(o['real']) - len(R)} ours {len(o['ours']) - len(O)}")
    print('   mesh side, by longest dimension (units after scale):  class: real / ours   |  by paint contrast (grey 0-255): real / ours')
    line = '      ' + '  '.join(f"{n}: {sum(dlib.cls(m['long'], dlib.SIZE_U) == n for m in R)} / {sum(dlib.cls(m['long'], dlib.SIZE_U) == n for m in O)}" for _, _, n in dlib.SIZE_U)
    line += '   |   ' + '  '.join(f"{n}: {sum(dlib.cls(abs(m['dgrey']), dlib.CON) == n for m in R)} / {sum(dlib.cls(abs(m['dgrey']), dlib.CON) == n for m in O)}" for _, _, n in dlib.CON)
    print(line)
    for cam in ('game', 'close', 'board'):
        vis = [m for m in R if m[cam]['kept'] is not None]
        if not vis: continue
        rc, n, sp, ns = gate_value(o, cam)
        print(f"   {cam}: real marks visible (>= {dlib.PX_MIN} px, effect >= {dlib.C_MIN:.0f} grey) {n} of {len(R)}; RECALL {rc:.3f}" + (f"; ours' visible marks {ns}, of them NOT in the real picture {sp:.3f}" if cam == 'game' and sp is not None else ''))
        if cam == 'board': continue
        print('      by largest footprint: ' + '  '.join(f"{nm}: {np.mean([m[cam]['kept'] for m in vis if dlib.cls(m[cam]['max_px'], dlib.SIZE_PX) == nm]):.2f} (n {sum(dlib.cls(m[cam]['max_px'], dlib.SIZE_PX) == nm for m in vis)})"
                                                 for _, _, nm in dlib.SIZE_PX if any(dlib.cls(m[cam]['max_px'], dlib.SIZE_PX) == nm for m in vis)))
        print('      by lit effect:        ' + '  '.join(f"{nm}: {np.mean([m[cam]['kept'] for m in vis if dlib.cls(m[cam]['max_con'], dlib.CON) == nm]):.2f} (n {sum(dlib.cls(m[cam]['max_con'], dlib.CON) == nm for m in vis)})"
                                                 for _, _, nm in dlib.CON if any(dlib.cls(m[cam]['max_con'], dlib.CON) == nm for m in vis)))
        if cam == 'game':
            print('      by zone:              ' + '  '.join(f"{z}: {np.mean([m[cam]['kept'] for m in vis if m['zone'] == z]):.2f} (n {sum(m['zone'] == z for m in vis)})" for z in o['zones'] if any(m['zone'] == z for m in vis)))

if __name__ == '__main__':
    args = sys.argv[1:]
    if args and args[0] == '--table':
        fs = [dlib.WORK / 'results' / f'{t}.json' for t in args[1:]] or sorted((dlib.WORK / 'results').glob('*.json'))
        for f in fs: tables(json.loads(f.read_text()))
    else:
        for c in args: tables(run(c))
