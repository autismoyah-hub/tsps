"""census.py: every spot animation in the cache: definition, model numbers, sequence numbers, how its motion is built.
usage: census.py            -> data/spots.jsonl, data/census.txt
One row per spot animation:
  id, name (gameval), def fields, model: verts faces colours alpha-faces alpha-values flat textured priorities labels box,
  sequence: kind (classic/cached/none), frames, cycles, loop, base, and for classic sequences which transform KINDS the
  frames use (translate/rotate/scale/alpha), the posed box range (how far scale grows it), and the mean alpha over time.
"""
import sys, json, statistics as st, collections, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fxlib import *

OUT = HERE / 'data'; OUT.mkdir(exist_ok=True)


def model_stats(m):
    v = m['vertices']; f = m['faces']
    if not v: return dict(verts=0, faces=0)
    lo = [min(p[i] for p in v) for i in range(3)]; hi = [max(p[i] for p in v) for i in range(3)]
    area = []
    for a, b, c in f:
        ax, ay, az = v[a]; bx, by, bz = v[b]; cx, cy, cz = v[c]
        ux, uy, uz = bx - ax, by - ay, bz - az; wx, wy, wz = cx - ax, cy - ay, cz - az
        n = (uy * wz - uz * wy, uz * wx - ux * wz, ux * wy - uy * wx)
        area.append(0.5 * (n[0] ** 2 + n[1] ** 2 + n[2] ** 2) ** 0.5)
    tot = sum(area) or 1
    alpha = m['alpha']
    drawn = [i for i in range(len(f)) if m['render_types'][i] & 2 == 0]
    al_faces = sum(1 for i in range(len(f)) if alpha[i] not in (0,))
    col_area = collections.Counter()
    for i in range(len(f)):
        if m['face_textures'][i] == -1: col_area[m['colors'][i]] += area[i]
    cols = sorted(col_area.items(), key=lambda kv: -kv[1])
    acc = 0; n90 = 0
    for c, a in cols:
        acc += a; n90 += 1
        if acc >= 0.9 * sum(col_area.values()): break
    return dict(verts=len(v), faces=len(f), alpha_faces=al_faces, alpha_share=round(sum(area[i] for i in range(len(f)) if alpha[i]) / tot, 3),
                alpha_values=sorted(set(a for a in alpha if a)), mean_alpha=round(sum(alpha[i] * area[i] for i in range(len(f))) / tot, 1),
                flat=sum(1 for r in m['render_types'] if r & 1), textured=sum(1 for t in m['face_textures'] if t != -1),
                colours=len(col_area), colours90=n90, top=[[*hsl(c), round(a / tot, 3)] for c, a in cols[:6]],
                prios=len(set(m['priorities'])), vlabels=len(set(m['vertex_skins'] or [])), flabels=len(set(m['face_skins'])) if any(m['face_skins']) else 0,
                box=[hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]], lo=lo, hi=hi, fmt=m['format'], weights=m['has_bone_weights'],
                median_face=round(st.median(area), 1) if area else 0, undrawn=len(f) - len(drawn))


def seq_stats(sid, m):
    try: s = al.sequence(sid)
    except KeyError: return dict(kind='missing')
    if s['cached_animation'] is not None:
        r = s['cached_range'] or (0, 0)
        return dict(kind='cached', frames=r[1] - r[0], cycles=r[1] - r[0], loop=s['loop_frames'], max_loops=s['max_loops'])
    if not s['frames']: return dict(kind='empty')
    out = dict(kind='classic', frames=len(s['frames']), cycles=sum(s['lengths']), loop=s['loop_frames'], max_loops=s['max_loops'],
               distinct=len(set(s['frames'])), lengths=s['lengths'][:40])
    try:
        s, bid, base, frames = al.sequence_frames(sid)
    except Exception as e:
        out['error'] = str(e)[:60]; return out
    out['base'] = bid; out['groups'] = len(base['types'])
    kinds = collections.Counter()
    for fr in frames:
        for g, v in fr['groups'].items():
            k = base['types'][g]
            default = 128 if k == 3 else 0
            if any(x != default for x in v): kinds[al.KIND.get(k, str(k))] += 1
    out['uses'] = {k: n for k, n in kinds.items() if k != 'origin'}
    # posed size and alpha over time (the python port of the client's deformation)
    if m and m['vertices'] and m['vertex_skins'] is not None or (m and m['vertices']):
        labels = m['vertex_skins'] or [0] * len(m['vertices'])
        fl = m['face_skins']
        sizes = []; alphas = []
        step = max(1, len(frames) // 12)
        for fr in frames[::step]:
            v = [list(p) for p in m['vertices']]
            try:
                al.apply_frame(v, labels, fr, base)
            except Exception as e:
                out['pose_error'] = str(e)[:60]; break
            lo = [min(p[i] for p in v) for i in range(3)]; hi = [max(p[i] for p in v) for i in range(3)]
            sizes.append(max(hi[i] - lo[i] for i in range(3)))
            # alpha groups: type 5 adds 8*x to faces whose face label is listed
            a = list(m['alpha'])
            for g, val in al.transforms(fr, base):
                if base['types'][g] == 5:
                    ls = set(base['labels'][g])
                    for i in range(len(a)):
                        if fl[i] in ls: a[i] = max(0, min(255, a[i] + 8 * val[0]))
            alphas.append(round(sum(a) / max(1, len(a)), 1))
        if sizes: out['size_min'] = min(sizes); out['size_max'] = max(sizes); out['size_first'] = sizes[0]; out['size_last'] = sizes[-1]
        if alphas: out['alpha_first'] = alphas[0]; out['alpha_last'] = alphas[-1]; out['alpha_min'] = min(alphas); out['alpha_max'] = max(alphas)
    return out


def main():
    nm = names(8); sn = names(7)
    rows = []
    for sid in sorted(spots()):
        d = decode_spot(spots()[sid])
        row = dict(id=sid, name=nm.get(sid, ''), model=d['model'], sequence=d['sequence'], seq_name=sn.get(d['sequence'], ''), scale=d['scale'], rotation=d['rotation'],
                   ambient=d['ambient'], contrast=d['contrast'], flag=d['flag'], recolor=len(d['recolor']), retexture=len(d['retexture']))
        m = None
        try:
            m = spot_model(d); row['m'] = model_stats(m)
        except Exception as e:
            row['m'] = dict(error=str(e)[:60])
        row['s'] = seq_stats(d['sequence'], m) if d['sequence'] >= 0 else dict(kind='none')
        rows.append(row)
    with (OUT / 'spots.jsonl').open('w', encoding='utf-8') as f:
        for r in rows: f.write(json.dumps(r) + '\n')
    print(len(rows), 'spot animations; named', sum(1 for r in rows if r['name']))


if __name__ == '__main__':
    main()
