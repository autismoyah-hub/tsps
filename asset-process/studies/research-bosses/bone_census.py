"""Byte round trip of bonecodec over the whole cache (read only).

usage: bone_census.py models | bases | anims | all      -> prints counts; writes data/census-*.json
models: every file of archive 7 decoded; every model re-encoded (a) with the stored hints, (b) without; (a) and (b) are
        compared with the stored bytes when the stored layout is the 'skinned' one (trailer ff fd); (b) is always
        decoded again and compared as CONTENT.
bases:  every group of archive 1: encode_base(decode_base(x)) == x.
anims:  every file of archive 22: encode_anim(decode_anim(x)) == x.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, collections, traceback
HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-animation'))
import animlib as A, bonecodec as B
DATA = HERE / 'data'; DATA.mkdir(exist_ok=True)
c = A.Cache()

def models():
    ref = c.reference(7); ids = sorted(ref['groups']); st = collections.Counter(); bad = []; weighted = []
    for n, mid in enumerate(ids):
        try: d = c.file(7, mid, 0)
        except Exception as e: st['unreadable'] += 1; continue
        try: m = B.decode_model(d)
        except Exception as e:
            st['decode failed'] += 1; bad.append((mid, 'decode', repr(e)[:80])); continue
        st['format ' + m['format']] += 1
        w = m['bone_weights'] is not None
        if w: st['weighted'] += 1; weighted.append(mid)
        if m['format'] != 'skinned':
            continue
        try:
            e1 = B.encode_model(m, hints=True); e2 = B.encode_model(m, hints=False)
        except Exception as e:
            st['encode failed'] += 1; bad.append((mid, 'encode', repr(e)[:80])); continue
        key = 'weighted ' if w else 'unweighted '
        st[key + 'skinned total'] += 1
        st[key + 'hints byte-identical'] += e1 == d
        st[key + 'no-hints byte-identical'] += e2 == d
        if e1 != d: bad.append((mid, 'hints differ', len(e1) - len(d)))
        diff = B.model_equal(m, B.decode_model(e2))
        st[key + 'no-hints content-identical'] += not diff
        if diff: bad.append((mid, 'content', diff))
        if e2 != d:
            st[key + 'no-hints bytes saved'] += len(d) - len(e2)
            m2 = B.decode_model(e2)
            st[key + 'no-hints: vertex flags differ'] += m2['_vflags'] != m['_vflags']
            st[key + 'no-hints: strip opcodes differ'] += m2['_ops'] != m['_ops']
        if n % 5000 == 0: print(n, len(ids), flush=True)
    out = dict(counts=dict(sorted(st.items())), failures=bad[:200], weighted=weighted)
    (DATA / 'census-models.json').write_text(json.dumps(out)); print(json.dumps(out['counts'], indent=1)); print('failures', len(bad), bad[:10])

def bases():
    ref = c.reference(1); st = collections.Counter(); bad = []; boned = {}
    for bid in sorted(ref['groups']):
        d = c.file(1, bid, 0); st['bases'] += 1
        try: b = B.decode_base(d)
        except AssertionError as e:
            st['rejected (the client rejects it too: ' + str(e) + ')'] += 1; bad.append(bid); continue
        ok = B.encode_base(b) == d; st['byte-identical'] += ok
        if not ok: bad.append(bid)
        if b['bones']:
            st['with bones'] += 1; boned[bid] = dict(groups=len(b['types']), bones=len(b['bones']), poses=b['poses'],
                                                      roots=sum(1 for x in b['bones'] if x['parent'] < 0),
                                                      extra_nonzero=sum(1 for x in b['bones'] for e in x['extra'] if any(e)))
        st['tail ' + b['_tail'].hex()[:12]] += 1 if not b['bones'] else 0
    (DATA / 'census-bases.json').write_text(json.dumps(dict(counts=dict(st), failures=bad, boned=boned)))
    print(json.dumps(dict(st), indent=1)); print('failures', bad[:20])
    print('poses', collections.Counter(v['poses'] for v in boned.values()), 'roots', collections.Counter(v['roots'] for v in boned.values()))
    print('bones min/median/max', sorted(v['bones'] for v in boned.values())[::max(1, len(boned) // 2)], max(v['bones'] for v in boned.values()))

def anims():
    ref = c.reference(22); st = collections.Counter(); bad = []
    kinds = collections.Counter(); chans = collections.Counter(); ext = collections.Counter(); nkeys = []
    ranges = collections.Counter(); tangents = collections.Counter(); byte0 = collections.Counter(); vers = collections.Counter(); maxt = []
    for gid in sorted(ref['groups']):
        for fid, d in c.files(22, gid).items():
            st['files'] += 1
            try: a = B.decode_anim(d)
            except Exception as e: st['decode failed'] += 1; bad.append((gid, fid, repr(e)[:60])); continue
            ok = B.encode_anim(a) == d; st['byte-identical'] += ok
            if not ok: bad.append((gid, fid, 'bytes'))
            vers[a['version']] += 1; ranges['start 0' if a['range'][0] == 0 else 'start >0'] += 1; st['pose ' + str(a['pose'])] += 1
            st['channels'] += len(a['channels'])
            tmax = 0
            for ch in a['channels']:
                kinds[ch['kind']] += 1; chans[(ch['kind'], ch['channel'])] += 1; cv = ch['curve']
                ext[(cv['before'], cv['after'])] += 1; byte0[cv['byte0']] += 1; st['weighted curves'] += cv['weighted'] != 0
                nkeys.append(len(cv['keys'])); tmax = max(tmax, cv['keys'][-1][0])
                for k in cv['keys'][:-1]:
                    o = (k[4], k[5])
                    tangents['step (out 0,0)' if o == (0.0, 0.0) else 'step-next (out max,max)' if k[4] > 3e38 and k[5] > 3e38 else 'spline'] += 1
            maxt.append((tmax, a['range'][1]))
    nkeys.sort()
    out = dict(counts=dict(st), versions=dict(vers), kinds={str(k): v for k, v in kinds.items()}, channels={f'{k[0]}:{k[1]}': v for k, v in sorted(chans.items())},
               extrapolation={f'{k[0]},{k[1]}': v for k, v in ext.items()}, byte0={str(k): v for k, v in byte0.items()}, ranges=dict(ranges), tangents=dict(tangents),
               keys_per_curve=dict(min=nkeys[0], median=nkeys[len(nkeys) // 2], p90=nkeys[int(len(nkeys) * .9)], max=nkeys[-1]),
               last_key_beyond_range=sum(1 for t, e in maxt if t > e), last_key_equals_range=sum(1 for t, e in maxt if t == e), failures=bad[:100])
    (DATA / 'census-anims.json').write_text(json.dumps(out)); print(json.dumps(out, indent=1))

what = sys.argv[1] if len(sys.argv) > 1 else 'all'
if what in ('bases', 'all'): bases()
if what in ('anims', 'all'): anims()
if what in ('models', 'all'): models()
