import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, collections
sys.path.insert(0, _AP.fix(r'<ROOT>\studies\research-bosses'))
import bonelib as L, bonecodec as B
import numpy as np
for npc in [int(x) for x in sys.argv[1:]]:
    d = L.npc_def(npc); print({k: d[k] for k in d if k not in ('recolor',)})
    bid = L.base_of_seq(d['anims'][0]); base = B.decode_base(L.base_bytes(bid))
    ns = 0; mx = 0
    for b in base['bones']:
        M = np.array(b['matrices'][0]).reshape(4, 4).T
        sc = np.linalg.norm(M[:3, :3], axis=0)
        if abs(sc - 1).max() > 1e-4: ns += 1; mx = max(mx, abs(sc - 1).max())
    print('base', bid, len(base['bones']), 'bones; with rest scale', ns, mx, 'groups', len(base['types']))
    for sid, n in L.seqs_on_base(bid):
        aid, rng = L.anim_id(sid); a = B.decode_anim(L.anim_bytes(aid))
        times = collections.Counter(); w = 0; scv = 0; nonu = 0; kinds = collections.Counter(); tang = collections.Counter()
        sc = {}
        for c in a['channels']:
            kinds[(c['kind'])] += 1
            w += c['curve']['weighted']
            ks = c['curve']['keys']
            times[tuple(k[0] for k in ks)[:6] + (len(ks), ks[-1][0])] += 1
            for k in ks[:-1]:
                o = (k[4], k[5]); tang['step' if o[0] > 1e38 else 'hold' if o == (0.0, 0.0) else 'spline'] += 1
            if c['kind'] == 1 and c['channel'] >= 7:
                sc.setdefault(c['target'], {})[c['channel']] = [k[1] for k in ks]
                if any(abs(k[1] - 1) > 1e-3 for k in ks): scv += 1
        for b, chs in sc.items():
            if len(chs) == 3 and len({len(v) for v in chs.values()}) == 1:
                v = np.array([chs[7], chs[8], chs[9]])
                if (abs(v - v[0]).max() > 1e-3): nonu += 1
            elif len({len(v) for v in chs.values()}) != 1: nonu += 1
        print(' seq', sid, n, 'anim', aid, 'range', a['range'], rng, 'kinds', dict(kinds), 'weighted', w, 'scale curves moving', scv, 'bones non-uniform scale', nonu, 'segs', dict(tang), 'time patterns', times.most_common(3))
if '--keys' in sys.argv:
    pass
