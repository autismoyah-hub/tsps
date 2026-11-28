"""soft.py: HOW softness is made, measured on real effects (and on ours with `one`).
usage: soft.py measure            -> data/soft.json, data/soft.txt      (the study set below: real effects of the reference fights)
       soft.py one EFFECT JOB [--height H]     the same numbers for one effect (id:N | def:PATH | loc:ID | locf:PATH)
From the FILES (model + sequence):
  parts      connected pieces of the mesh (welded by position): count, faces per piece, closed shells or open cards
  layers     pieces nested inside others (box inside box): depth of nesting, alpha per level from the innermost out
  alpha      values by surface share; alpha against distance from the model's centre (r, rank correlation); lightness the same
  facets     median face edge over the largest dimension; share of smooth faces; colours
  motion     over the frames: posed size and mean alpha; correlation; where in the life the size peaks; mean alpha at the peak
On SCREEN (close lens, the moment the effect covers most pixels; pixels changed by 8 or more against the same scene without it):
  edge       mean change (0..255 per channel) of pixels 1 px inside the outline, 2..3 px inside, and of the innermost third;
             edge ratio = edge / core: 1.0 is a hard cut-out, 0.3 a soft falloff
  levels     how many distinct change levels (bins of 16) carry at least 4% of the pixels: tonal steps across the mass
"""
import sys, json, collections, pathlib, math
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx, fxtools, objlib, objcensus
from fxlib import *
import numpy as np

# (effect, job, height, kind): real effects of the reference fights, chosen by drawing (the first engineer's study set + objects)
STUDY = [
    ('id:1997', 'proj', 0, 'projectile'), ('id:1999', 'proj', 0, 'projectile'), ('id:2004', 'proj', 0, 'projectile'), ('id:2007', 'proj', 0, 'projectile'),
    ('id:1764', 'proj', 0, 'projectile'), ('id:1766', 'proj', 0, 'projectile'), ('id:1340', 'proj', 0, 'projectile'), ('id:1580', 'proj', 0, 'projectile'),
    ('id:1583', 'proj', 0, 'projectile'), ('id:1593', 'proj', 0, 'projectile'), ('id:2206', 'proj', 0, 'projectile'), ('id:3445', 'proj', 0, 'projectile'),
    ('id:3254', 'proj', 0, 'projectile'), ('id:1477', 'proj', 0, 'projectile'), ('id:1711', 'proj', 0, 'projectile'),
    ('id:1998', 'actor', 100, 'impact'), ('id:2008', 'actor', 100, 'impact'), ('id:1765', 'actor', 100, 'impact'), ('id:1584', 'actor', 100, 'impact'),
    ('id:2205', 'actor', 100, 'impact'), ('id:3255', 'actor', 100, 'impact'), ('id:1478', 'actor', 100, 'impact'), ('id:2005', 'actor', 100, 'impact'),
    ('id:2015', 'actor', 0, 'aura'), ('id:2017', 'actor', 0, 'aura'), ('id:2125', 'actor', 0, 'aura'), ('id:3243', 'actor', 0, 'aura'), ('id:2141', 'actor', 0, 'aura'),
    ('id:1558', 'actor', 0, 'aura'), ('id:1597', 'actor', 0, 'aura'), ('id:3280', 'actor', 0, 'aura'),
    ('id:1654', 'spot', 0, 'hazard'), ('id:1767', 'spot', 0, 'hazard'), ('id:2013', 'spot', 0, 'hazard'), ('id:3857', 'spot', 0, 'hazard'), ('id:3429', 'spot', 0, 'hazard'),
    ('id:1356', 'spot', 0, 'hazard'), ('id:2432', 'spot', 0, 'hazard'),
    ('id:1446', 'spot', 0, 'telegraph'), ('id:3415', 'spot', 0, 'telegraph'), ('id:2667', 'spot', 0, 'telegraph'), ('id:1595', 'spot', 0, 'telegraph'),
    ('id:2146', 'spot', 0, 'telegraph'), ('id:2440', 'spot', 0, 'telegraph'),
    ('id:2197', 'spot', 0, 'hit'), ('id:3370', 'spot', 0, 'hit'), ('id:3262', 'spot', 0, 'hit'),
    ('loc:11700', 'loc', 0, 'object'), ('loc:37739', 'loc', 0, 'object'), ('loc:30021', 'loc', 0, 'object'), ('loc:33035', 'loc', 0, 'object'),
    ('loc:47573', 'loc', 0, 'object'), ('loc:56003', 'loc', 0, 'object'), ('loc:56264', 'loc', 0, 'object'), ('loc:54279', 'loc', 0, 'object'),
]


def load(effect):
    """(model dict recoloured, sequence stats dict or None, scale)"""
    kind, v = effect.split(':', 1)
    if kind == 'id':
        d = decode_spot(spots()[int(v)]); m = spot_model(d)
        seq = None
        if d['sequence'] >= 0:
            s = al.sequence(d['sequence'])
            if s['cached_animation'] is None: seq = al.sequence_frames(d['sequence'])
        return m, seq
    if kind == 'def':
        D = fxtools.read_def(v); q = D['seq']
        return D['model'], (q['s'], q['bid'], q['base'], q['frames']) if q else None
    if kind == 'loc':
        d = objlib.obj(int(v)); m = objcensus.combined(d, objlib.shape_for(d)); seq = None
        if d['anim'] is not None:
            s = al.sequence(d['anim'])
            if s['cached_animation'] is None: seq = al.sequence_frames(d['anim'])
        return m, seq
    if kind == 'locf':
        kv = dict(l.split('=', 1) for l in pathlib.Path(v).read_text().splitlines() if '=' in l and not l.startswith('#'))
        m = oc.decode_model((pathlib.Path(v).parent / kv['model'].split(',')[0]).read_bytes()); seq = None
        if 'anim' in kv:
            ad = pathlib.Path(v).parent / kv['anim']; s = al.decode_sequence((ad / 'seq.dat').read_bytes()); frames = []; base = bid = None
            for fid in s['frames']:
                data = (ad / f'frame_{fid >> 16}_{fid & 0xffff}.dat').read_bytes()
                if base is None: bid = int.from_bytes(data[:2], 'big'); base = al.decode_base((ad / f'base_{bid}.dat').read_bytes())
                frames.append(al.decode_frame(data, base))
            seq = (s, bid, base, frames)
        return m, seq
    raise SystemExit(effect)


def rank_corr(a, b):
    if len(a) < 4 or len(set(a)) < 2 or len(set(b)) < 2: return None
    ra = np.argsort(np.argsort(a)); rb = np.argsort(np.argsort(b))
    return round(float(np.corrcoef(ra, rb)[0, 1]), 2)


def file_stats(m, seq):
    V = np.array(m['vertices'], float); F = m['faces']; nf = len(F)
    if nf == 0: return {}
    # weld by position -> components
    key = {}; vid = []
    for p in m['vertices']: vid.append(key.setdefault(tuple(p), len(key)))
    parent = list(range(len(key)))
    def find(a):
        while parent[a] != a: parent[a] = parent[parent[a]]; a = parent[a]
        return a
    for a, b, c in F:
        ra, rb, rc = find(vid[a]), find(vid[b]), find(vid[c]); parent[rb] = ra; parent[rc] = ra
    comp = collections.defaultdict(list)
    for i, (a, b, c) in enumerate(F): comp[find(vid[a])].append(i)
    tri = np.array([[V[a], V[b], V[c]] for a, b, c in F]); cen = tri.mean(axis=1)
    area = 0.5 * np.linalg.norm(np.cross(tri[:, 1] - tri[:, 0], tri[:, 2] - tri[:, 0]), axis=1); tot = area.sum() or 1
    alpha = np.array(m['alpha'], float); light = np.array([c & 127 for c in m['colors']], float); sat = np.array([(c >> 7) & 7 for c in m['colors']], float)
    lo, hi = V.min(axis=0), V.max(axis=0); big = float((hi - lo).max()) or 1
    centre = (cen * area[:, None]).sum(axis=0) / tot
    r = np.linalg.norm(cen - centre, axis=1) / (big / 2)
    parts = []
    for root, fs in comp.items():
        fs = np.array(fs); pv = tri[fs].reshape(-1, 3); plo, phi = pv.min(axis=0), pv.max(axis=0)
        edges = collections.Counter()
        for i in fs:
            a, b, c = (vid[x] for x in F[i])
            for e in ((a, b), (b, c), (c, a)): edges[tuple(sorted(e))] += 1
        closed = all(n == 2 for n in edges.values())
        dims = sorted(phi - plo)
        parts.append(dict(faces=len(fs), area=float(area[fs].sum()), lo=plo, hi=phi, size=float(dims[2]), flat=bool(dims[0] <= 0.12 * max(1.0, dims[2])), closed=closed,
                          alpha=float((alpha[fs] * area[fs]).sum() / max(1e-9, area[fs].sum())), light=float((light[fs] * area[fs]).sum() / max(1e-9, area[fs].sum())),
                          sat=float((sat[fs] * area[fs]).sum() / max(1e-9, area[fs].sum()))))
    # nesting: part a is inside part b if its box is inside b's box (2 units of slack) and it is smaller
    for a in parts:
        a['depth'] = sum(1 for b in parts if b is not a and b['size'] > a['size'] * 1.08 and (a['lo'] >= b['lo'] - 2).all() and (a['hi'] <= b['hi'] + 2).all() and not b['flat'])
    maxd = max(p['depth'] for p in parts)
    levels = []
    for dpt in range(maxd, -1, -1):   # innermost first
        ps = [p for p in parts if p['depth'] == dpt]; w = sum(p['area'] for p in ps) or 1
        levels.append(dict(level_from_inside=maxd - dpt, parts=len(ps), alpha=round(sum(p['alpha'] * p['area'] for p in ps) / w), light=round(sum(p['light'] * p['area'] for p in ps) / w), sat=round(sum(p['sat'] * p['area'] for p in ps) / w, 1)))
    edge_len = np.concatenate([np.linalg.norm(tri[:, i] - tri[:, (i + 1) % 3], axis=1) for i in range(3)])
    av = collections.Counter()
    for a, ar in zip(alpha, area): av[int(a)] += ar
    out = dict(faces=nf, big=round(big), parts=len(parts), part_faces=sorted((p['faces'] for p in parts), reverse=True)[:8], closed_parts=sum(p['closed'] for p in parts), card_parts=sum(p['flat'] for p in parts),
               small_parts=sum(1 for p in parts if p['size'] < 0.2 * big), nest_depth=maxd, levels=levels,
               alpha_share=round(float(area[alpha > 0].sum() / tot), 2), alpha_values=[[a, round(ar / tot, 2)] for a, ar in sorted(av.items()) if ar / tot >= 0.02],
               alpha_vs_radius=rank_corr(r, alpha) if (alpha > 0).any() else None, light_vs_radius=rank_corr(r, light), sat_vs_radius=rank_corr(r, sat),
               inner_alpha=round(float((alpha[r < 0.4] * area[r < 0.4]).sum() / max(1e-9, area[r < 0.4].sum()))) if (r < 0.4).any() else None,
               outer_alpha=round(float((alpha[r > 0.7] * area[r > 0.7]).sum() / max(1e-9, area[r > 0.7].sum()))) if (r > 0.7).any() else None,
               inner_light=round(float((light[r < 0.4] * area[r < 0.4]).sum() / max(1e-9, area[r < 0.4].sum()))) if (r < 0.4).any() else None,
               outer_light=round(float((light[r > 0.7] * area[r > 0.7]).sum() / max(1e-9, area[r > 0.7].sum()))) if (r > 0.7).any() else None,
               edge_over_size=round(float(np.median(edge_len) / big), 3), smooth_share=round(sum(1 for t in m['render_types'] if not t & 1) / nf, 2), colours=len(set(m['colors'])),
               z_back=round(float(hi[2])), z_front=round(float(lo[2])))
    # a trail: alpha and width against z behind the origin (projectiles fly -z first)
    back = cen[:, 2] > 0.15 * big
    if back.sum() >= 6 and (cen[back, 2] < np.median(cen[back, 2])).any():
        out['trail'] = dict(faces=int(back.sum()), alpha_vs_z=rank_corr(cen[back, 2], alpha[back]), width_vs_z=rank_corr(cen[back, 2], np.linalg.norm(cen[back, :2], axis=1)),
                            alpha_near=round(float(alpha[back][cen[back, 2] < np.median(cen[back, 2])].mean())), alpha_far=round(float(alpha[back][cen[back, 2] >= np.median(cen[back, 2])].mean())))
    if seq:
        s, bid, base, frames = seq
        labels = m['vertex_skins'] or [0] * len(m['vertices']); sizes = []; alphas = []; holds = s['lengths']
        for fr in frames:
            v = [list(p) for p in m['vertices']]
            if m['vertex_skins'] is not None: al.apply_frame(v, labels, fr, base)
            v = np.array(v, float); sizes.append(float((v.max(axis=0) - v.min(axis=0)).max()))
            a = list(m['alpha'])
            for g, val in al.transforms(fr, base):
                if base['types'][g] == 5:
                    ls = set(base['labels'][g])
                    for j in range(len(a)):
                        if m['face_skins'][j] in ls: a[j] = max(0, min(255, a[j] + 8 * val[0]))
            alphas.append(float((np.array(a) * area).sum() / tot))
        t = np.cumsum([0] + holds[:-1]) / max(1, sum(holds))
        k = int(np.argmax(sizes))
        out['motion'] = dict(frames=len(frames), cycles=sum(holds), size_first=round(sizes[0]), size_max=round(max(sizes)), size_last=round(sizes[-1]), peak_at=round(float(t[k]), 2),
                             alpha_first=round(alphas[0]), alpha_at_peak=round(alphas[k]), alpha_min=round(min(alphas)), alpha_last=round(alphas[-1]),
                             size_alpha_corr=rank_corr(sizes, alphas), fade_from=round(float(t[int(np.argmin(alphas))]), 2))
    return out


def erode(mask):
    m = mask.copy(); m[1:] &= mask[:-1]; m[:-1] &= mask[1:]; m[:, 1:] &= mask[:, :-1]; m[:, :-1] &= mask[:, 1:]
    m[0] = m[-1] = False; m[:, 0] = m[:, -1] = False
    return m


def screen_stats(effect, job, height=0, n=9):
    total = fx.life(effect, job); cyc = fx.moments(n, total)
    A, _ = fx.render(effect, job, cyc, close=True, size=(480, 400), height=height)
    import tempfile, shutil
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='soft-'))
    fx.run(fx.scene(job, True, (480, 400)) + [f'render {",".join(str(4 + c) for c in cyc)} {tmp.as_posix()}/e'])
    E = fx.frames(f'{tmp.as_posix()}/e', [4 + c for c in cyc]); shutil.rmtree(tmp, ignore_errors=True)
    best = None
    for c, a, e in zip(cyc, A, E):
        d = np.abs(np.asarray(a).astype(int) - np.asarray(e).astype(int)); mask = (d >= 8).any(axis=2)
        if best is None or mask.sum() > best[0]: best = (int(mask.sum()), c, d.mean(axis=2), mask)
    px, c, d, mask = best
    if px < 80: return dict(px=px, cycle=c)
    rings = []; cur = mask
    for _ in range(40):
        nxt = erode(cur); rings.append(cur & ~nxt); cur = nxt
        if not cur.any(): break
    depth = len(rings)
    edge = float(d[rings[0]].mean()); in2 = float(d[rings[1] | rings[2]].mean()) if depth > 2 and (rings[1] | rings[2]).any() else edge
    core_mask = np.zeros_like(mask)
    for q in rings[max(1, (2 * depth) // 3):]: core_mask |= q
    core = float(d[core_mask].mean()) if core_mask.any() else in2
    peak = float(np.percentile(d[mask], 90))
    hist = np.bincount((d[mask] // 16).astype(int), minlength=16) / px
    return dict(px=px, cycle=c, edge=round(edge, 1), in2=round(in2, 1), core=round(core, 1), p90=round(peak, 1), mean=round(float(d[mask].mean()), 1), edge_ratio=round(edge / max(1e-6, max(core, peak * 0.8)), 2),
                levels=int((hist >= 0.04).sum()), depth_px=depth)


def one(effect, job, height=0, screen=True):
    m, seq = load(effect)
    r = dict(effect=effect, job=job, files=file_stats(m, seq))
    if screen: r['screen'] = screen_stats(effect, job, height)
    return r


def line(r, name=''):
    f = r['files']; s = r.get('screen', {}); mo = f.get('motion', {})
    lv = ' > '.join(f'a{l["alpha"]} L{l["light"]}' for l in f['levels'])
    return (f'{r["effect"]:10s} {name[:38]:38s} {f["faces"]:5d}f parts {f["parts"]:3d} (closed {f["closed_parts"]}, cards {f["card_parts"]}, small {f["small_parts"]}) nest {f["nest_depth"]} [{lv}] '
            f'alpha share {f["alpha_share"]} in/out {f["inner_alpha"]}/{f["outer_alpha"]} r {f["alpha_vs_radius"]} light in/out {f["inner_light"]}/{f["outer_light"]} smooth {f["smooth_share"]} edge/size {f["edge_over_size"]} '
            f'| motion size {mo.get("size_first")}>{mo.get("size_max")}@{mo.get("peak_at")} alpha {mo.get("alpha_first")}>{mo.get("alpha_at_peak")}>{mo.get("alpha_last")} corr {mo.get("size_alpha_corr")} '
            f'| screen px {s.get("px")} edge {s.get("edge")} core {s.get("core")} ratio {s.get("edge_ratio")} levels {s.get("levels")}')


def measure():
    out = []; res = []
    for effect, job, height, kind in STUDY:
        try:
            r = one(effect, job, height); r['kind'] = kind
            k, v = effect.split(':'); r['name'] = names(8 if k == 'id' else 6).get(int(v), '')
        except BaseException as e:
            print(effect, 'FAILED', str(e)[:200]); continue
        res.append(r); l = f'{kind:10s} ' + line(r, r['name']); print(l); out.append(l)
    (HERE / 'data' / 'soft.json').write_text(json.dumps(res, indent=1, default=lambda o: o.tolist() if hasattr(o, 'tolist') else str(o)), encoding='utf-8')
    (HERE / 'data' / 'soft.txt').write_text('\n'.join(out) + '\n', encoding='utf-8')


if __name__ == '__main__':
    a = sys.argv[1:]
    if a[0] == 'measure': measure()
    elif a[0] == 'one':
        h = int(a[a.index('--height') + 1]) if '--height' in a else 0
        print(line(one(a[1], a[2], h)))
