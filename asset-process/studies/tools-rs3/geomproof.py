"""Decoder proof (b): geometry shared between build-951 archive 47 and OSRS 239 / HD 561, 601, 614 archive 7.

Conversion RS3 -> OSRS axes: (x, y, z) -> (x/4, -y/4, z/4); triangle winding reversed (one axis mirrored).
A model "matches" when the set of face-referenced positions is identical after conversion. For every match
the triangle sets are compared (each triangle as its three positions, rotation-normalised).
usage: geomproof.py [--workers N]   writes geomproof.json
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json, hashlib, collections, argparse
from multiprocessing import Pool
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.path.insert(1, _AP.fix('<ROOT>/studies/tools-hd-era'))
sys.dont_write_bytecode = True
import numpy as np
from rs3flat import Cache, ROOT
import rs3model

REVS = (239, 561, 601, 614)

def norm_tri(t):
    i = min(range(3), key=lambda k: t[k])
    return t[i:] + t[:i]

def tri_keys(pos, tris):
    return collections.Counter(norm_tri(tuple(map(tuple, pos[t]))) for t in tris)

def vkey(pos, tris):
    used = np.unique(pos[np.unique(tris)], axis=0) if len(tris) else np.zeros((0, 3), int)
    return hashlib.sha1(np.ascontiguousarray(used.astype('<i4')).tobytes()).hexdigest()[:20], len(used)

_c = None
def rs3_sig(g):
    global _c
    if _c is None: _c = Cache()
    try:
        m = rs3model.decode(_c.group(47, g))
    except Exception as e:
        return g, None, 'decode'
    pos = m['positions']
    tris = np.concatenate([gr['triangles'] for gr in m['groups']]) if m['groups'] else np.zeros((0, 3), int)
    if m['flags'] & 0x100 or len(tris) == 0: return g, None, 'float or empty'
    if np.any(pos % 4): return g, None, 'subunit'
    conv = np.stack([pos[:, 0]//4, -pos[:, 1]//4, pos[:, 2]//4], 1)
    k, n = vkey(conv, tris)
    return g, (k, n, len(tris)), None

_h = {}
def hd_model(rev, i):
    from hdcache import Cache as HDCache
    from models import decode as hddecode, osrs
    if rev not in _h: _h[rev] = HDCache(rev)
    raw = _h[rev].group(7, i)
    hm = osrs.decode_model(raw) if rev == 239 else hddecode(raw)
    v = np.array(hm['vertices'], dtype=np.int64).reshape(-1, 3)
    if rev != 239 and hm.get('hd', {}).get('version') in (14, 15): v = v//4
    f = np.array(hm['faces'], dtype=np.int64).reshape(-1, 3)
    return v, f

def hd_sig(job):
    rev, i = job
    try:
        v, f = hd_model(rev, i)
    except Exception as e:
        return rev, i, None
    if len(f) == 0: return rev, i, None
    k, n = vkey(v, f)
    return rev, i, (k, n, len(f))

def compare(job):
    g, rev, i = job
    m = rs3model.decode(Cache().group(47, g))
    pos = m['positions']
    conv = np.stack([pos[:, 0]//4, -pos[:, 1]//4, pos[:, 2]//4], 1)
    tris = np.concatenate([gr['triangles'] for gr in m['groups']])
    a = tri_keys(conv, tris[:, ::-1])
    v, f = hd_model(rev, i)
    b = tri_keys(v, f)
    only_rs3 = a - b; only_old = b - a
    flipped = sum(1 for t in only_rs3 if norm_tri(t[::-1]) in only_old)
    degenerate_old = sum(n for t, n in b.items() if len(set(t)) < 3)
    return dict(rs3=g, rev=rev, model=i, rs3_triangles=sum(a.values()), old_triangles=sum(b.values()),
                identical=(a == b), only_rs3=sum(only_rs3.values()), only_old=sum(only_old.values()),
                winding_flipped=flipped, degenerate_old=degenerate_old,
                exact_vertex_set=vkey(conv, tris)[0] == vkey(v, f)[0],
                only_rs3_examples=[[list(map(int, p)) for p in t] for t in list(only_rs3)[:3]],
                only_old_examples=[[list(map(int, p)) for p in t] for t in list(only_old)[:3]])

if __name__ == '__main__':
    ap = argparse.ArgumentParser(); ap.add_argument('--workers', type=int, default=8); a = ap.parse_args()
    from hdcache import Cache as HDCache
    ids = Cache().present(47)
    sig = {}; skipped = collections.Counter()
    with Pool(a.workers) as pool:
        for g, s, why in pool.imap_unordered(rs3_sig, ids, chunksize=128):
            if s: sig[g] = s
            else: skipped[why] += 1
        print('rs3 signatures', len(sig), dict(skipped), flush=True)
        by_key = collections.defaultdict(list)
        for g, s in sig.items(): by_key[s[0]].append(g)
        jobs = [(rev, i) for rev in REVS for i in HDCache(rev).index(7)]
        hits = collections.defaultdict(set); seen = collections.Counter()
        near = lambda x, y: abs(x-y) <= max(2, y//100)
        for rev, i, s in pool.imap_unordered(hd_sig, jobs, chunksize=128):
            if not s: continue
            seen[rev] += 1
            bucket = by_key.get(s[0], [])
            if s[1] >= 12 and len(bucket) <= 20:  # trivial shapes (quads, cubes) would pair with everything
                for g in bucket:
                    hits[rev].add((g, rev, i))
            # Same model ID with near-equal welded vertex and face counts: compare even if not exact.
            if i in sig and near(sig[i][1], s[1]) and near(sig[i][2], s[2]):
                hits[rev].add((i, rev, i))
        hits = {r: sorted(h) for r, h in hits.items()}
        print('vertex-set matches', {r: len(h) for r, h in hits.items()}, flush=True)
        results = {}
        for rev in REVS:
            results[rev] = sorted(pool.map(compare, hits.get(rev, []), chunksize=16), key=lambda r: (r['rs3'], r['model']))
    out = dict(conversion='(x,y,z)_rs3 -> (x/4, -y/4, z/4)_osrs, triangle winding reversed', rs3_models_signed=len(sig),
               rs3_skipped=dict(skipped), comparison_models_decoded={str(r): seen[r] for r in REVS}, per_revision={})
    for rev, rows in results.items():
        same_id = [r for r in rows if r['rs3'] == r['model']]
        out['per_revision'][str(rev)] = dict(
            pairs_compared=len(rows), exact_vertex_set_pairs=sum(r['exact_vertex_set'] for r in rows),
            distinct_rs3_models=len({r['rs3'] for r in rows}),
            identical_vertices_and_faces=sum(r['identical'] for r in rows),
            identical_distinct_rs3=len({r['rs3'] for r in rows if r['identical']}),
            same_id_matches=len(same_id), same_id_identical=sum(r['identical'] for r in same_id),
            differences=[r for r in rows if not r['identical']],
            identical_pairs=[[r['rs3'], r['model']] for r in rows if r['identical']])
    (ROOT/'geomproof.json').write_text(json.dumps(out, indent=1))
    print(json.dumps({k: {kk: vv for kk, vv in v.items() if kk not in ('differences', 'identical_pairs')} for k, v in out['per_revision'].items()}, indent=1))
