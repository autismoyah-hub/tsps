"""Decode every archive-47 group; count versions/flags; round-trip; per-model statistics.

usage: census47.py [--sample N] [--workers W]
writes census47.json (aggregate + errors with examples) and model-stats.sqlite (one row per model).
"""
import sys, pathlib, json, collections, argparse, sqlite3, random, traceback
from multiprocessing import Pool
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
from rs3flat import Cache, ROOT
import rs3model
import numpy as np

cache = None
def work(g):
    global cache
    if cache is None: cache = Cache()
    out = dict(id=g)
    try:
        ver = cache.verify(47, g)
        out['ref_ok'] = all(ver.values())
        data = cache.group(47, g)
    except Exception as e:
        out['error'] = 'container: ' + repr(e); return out
    out['header'] = data[:17].hex()
    try:
        m = rs3model.decode(data)
    except Exception as e:
        out['error'] = f'{type(e).__name__}: {e}'; out['size'] = len(data); return out
    out['roundtrip'] = rs3model.encode(m) == data
    nv = len(m['positions'])
    tris = [g_['triangles'] for g_ in m['groups']]
    nf = sum(len(t) for t in tris)
    mats = sorted({g_['material'] for g_ in m['groups']})
    out.update(version=m['version'], sub=m['sub'], flags=m['flags'], vertices=nv, faces=nf,
               groups=len(m['groups']), materials=mats,
               group_flags=sorted({g_['flags'] for g_ in m['groups']}),
               group_heads=sorted({g_['head'].hex() + ':%d' % g_['byte7'] for g_ in m['groups']}),
               tail={k: len(v) for k, v in m['tail'].items()},
               max_bones=max((len(b) for b in m.get('bones', [])), default=0),
               bone_hist=dict(collections.Counter(len(b) for b in m.get('bones', []))),
               alpha_vertices=int((m['alpha'] < 255).sum()) if 'alpha' in m else 0,
               bounds=[m['positions'].min(0).tolist(), m['positions'].max(0).tolist()] if nv else None,
               unused_vertices=int(nv - len(np.unique(np.concatenate(tris)))) if nf else nv,
               index_width=m['index_width'], count_overflow=sum(1 for g_ in m['groups'] if g_['count_overflow']),
               alpha_flag_consistent=all(bool(g_['flags'] & 2) == bool(len(g_['triangles']) and 'alpha' in m and (m['alpha'][g_['triangles']] < 255).any()) for g_ in m['groups']),
               emitter_ids=[int.from_bytes(r[:2], 'little') for r in m['tail'][50]],
               effector_ids=[int.from_bytes(r[:2], 'little') for r in m['tail'][18]],
               rec39_heads=[r[:7].hex() for r in m['tail'][39]],
               uv_range=[float(m['uv'].min()), float(m['uv'].max())] if nv else None,
               s08=sorted(set(m['stream08'].tolist()))[:8] if 'stream08' in m else None,
               s04=sorted(set(m['stream04'].tolist()))[:8] if 'stream04' in m else None)
    return out

if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('--sample', type=int)
    ap.add_argument('--workers', type=int, default=6)
    ap.add_argument('--out', default='census47')
    a = ap.parse_args()
    ids = Cache().present(47)
    refs = Cache().references(47)
    missing = sorted(set(refs) - set(ids))
    if a.sample:
        random.seed(7); ids = sorted(random.sample(ids, a.sample))
    agg = dict(groups_in_reference=len(refs), files_present=len(Cache().present(47)), missing_from_export=missing[:50],
               missing_count=len(missing))
    versions = collections.Counter(); flags = collections.Counter(); vf = collections.Counter()
    gflags = collections.Counter(); heads = collections.Counter(); tails = collections.Counter()
    errors = collections.defaultdict(list); roundtrip = collections.Counter(); refok = collections.Counter()
    bones = collections.Counter(); subs = collections.Counter()
    extra = collections.defaultdict(collections.Counter); overflow_ids = []
    em = set(Cache().index(27)[0]); ef = set(Cache().index(27)[1])
    db = sqlite3.connect(ROOT/f'{a.out}.sqlite')
    db.execute('DROP TABLE IF EXISTS models')
    db.execute('CREATE TABLE models(id INTEGER PRIMARY KEY, version INT, flags INT, vertices INT, faces INT, groups INT, materials TEXT, alpha_vertices INT, max_bones INT, bone_hist TEXT, bounds TEXT, tail TEXT, roundtrip INT, ref_ok INT, error TEXT)')
    with Pool(a.workers) as pool:
        for i, r in enumerate(pool.imap_unordered(work, ids, chunksize=64)):
            refok[r.get('ref_ok')] += 1
            if 'error' in r:
                key = r['error'].split(' at ')[0][:90]
                errors[key].append(r['id'])
                db.execute('INSERT INTO models(id,error,ref_ok) VALUES(?,?,?)', (r['id'], r['error'], r.get('ref_ok')))
                continue
            versions[r['version']] += 1; subs[r['sub']] += 1
            flags[(r['version'], r['flags'])] += 1; vf[r['flags']] += 1
            for f in r['group_flags']: gflags[f] += 1
            for h in r['group_heads']: heads[h] += 1
            tails[json.dumps(r['tail'])] += 1
            roundtrip[r['roundtrip']] += 1
            bones[r['max_bones']] += 1
            extra['index_width'][r['index_width']] += 1
            extra['models_with_count_overflow'][r['count_overflow'] > 0] += 1
            extra['alpha_flag_consistent'][r['alpha_flag_consistent']] += 1
            for e in r['emitter_ids']: extra['emitter_id_in_27_0'][e in em] += 1
            for e in r['effector_ids']: extra['effector_id_in_27_1'][e in ef] += 1
            for h in r['rec39_heads']: extra['rec39_type_byte'][h[:2]] += 1
            if r['uv_range']:
                extra['uv_min_bucket'][int(max(-4, min(4, r['uv_range'][0])))] += 1
                extra['uv_max_bucket'][int(max(-4, min(4, r['uv_range'][1])))] += 1
            if r['count_overflow']: overflow_ids.append(r['id'])
            db.execute('INSERT INTO models VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)', (r['id'], r['version'], r['flags'], r['vertices'], r['faces'], r['groups'],
                       json.dumps(r['materials']), r['alpha_vertices'], r['max_bones'], json.dumps(r['bone_hist']), json.dumps(r['bounds']),
                       json.dumps(r['tail']), int(r['roundtrip']), int(bool(r.get('ref_ok'))), None))
            if i % 10000 == 0: print(i, flush=True)
    db.commit()
    agg.update(decoded=sum(versions.values()), versions=dict(sorted(versions.items())), sub_bytes=dict(subs),
               version_flags={f'v{v} 0x{f:x}': n for (v, f), n in sorted(flags.items())},
               vertex_flags={f'0x{f:x}': n for f, n in sorted(vf.items())},
               group_flags_models={f'0x{f:02x}': n for f, n in sorted(gflags.items())},
               group_head_models=dict(heads.most_common(60)), group_head_distinct=len(heads),
               tail_counts=dict(tails.most_common(40)), roundtrip_identical=dict((str(k), v) for k, v in roundtrip.items()),
               reference_crc_ok=dict((str(k), v) for k, v in refok.items()), max_bones_models=dict(sorted(bones.items())),
               extra={k: {str(kk): vv for kk, vv in v.items()} for k, v in extra.items()}, count_overflow_ids=sorted(overflow_ids),
               errors={k: dict(count=len(v), examples=sorted(v)[:12]) for k, v in sorted(errors.items(), key=lambda kv: -len(kv[1]))})
    (ROOT/f'{a.out}.json').write_text(json.dumps(agg, indent=1))
    print(json.dumps({k: agg[k] for k in ('decoded', 'versions', 'vertex_flags', 'roundtrip_identical', 'reference_crc_ok')}))
    print(json.dumps(agg['errors'], indent=0)[:3000])
