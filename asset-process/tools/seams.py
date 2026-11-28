import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, random, collections, statistics as st
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, measure as m
def seam_share(md):
    v=md['vertices']; pos=collections.Counter(v)
    return sum(n for n in pos.values() if n>1)/len(v)
def tones(md):
    # how many lightness steps each (hue,sat) family uses, area-weighted mean
    fam=collections.defaultdict(set)
    for c in md['colors']:
        h,s,l=o.hsl_split(c); fam[(h,s)].add(l)
    return st.mean(len(x) for x in fam.values()), len(fam)
if __name__=='__main__':
    random.seed(2)
    d=m.ARCH/'objects'/'shape-10'; ids=[int(p.stem) for p in d.glob('*.json')]; random.shuffle(ids)
    S=[];T=[];Fm=[];seen=set()
    for i in ids:
        j=json.loads((d/f'{i}.json').read_text()); k=tuple(j.get('model_ids') or [])
        if not k or k in seen or (j.get('faces') or 0)<100: continue
        seen.add(k)
        try: mods=[o.model(x) for x in k]
        except Exception: continue
        S.append(st.mean(seam_share(x) for x in mods)); t=tones(o.merge(mods)); T.append(t[0]); Fm.append(t[1])
        if len(S)>=300: break
    q=lambda xs,p: sorted(xs)[int(p*(len(xs)-1))]
    print('real objects n',len(S))
    print('share of vertices sitting on a duplicated position: p10 %.2f median %.2f p90 %.2f'%(q(S,.1),q(S,.5),q(S,.9)))
    print('lightness tones per hue/sat family: median %.1f p90 %.1f ; families per object median %d p90 %d'%(q(T,.5),q(T,.9),q(Fm,.5),q(Fm,.9)))
    P=pathlib.Path(_AP.fix(r'<E114_SCENE>\package'))
    S=[];T=[];Fm=[]
    for f in sorted(P.glob('m*.bin')):
        md=o.decode_model(f.read_bytes()); S.append(seam_share(md)); t=tones(md); T.append(t[0]); Fm.append(t[1])
    print('E114 chunks n',len(S))
    print('share duplicated: p10 %.2f median %.2f p90 %.2f'%(q(S,.1),q(S,.5),q(S,.9)))
    print('tones per family: median %.1f p90 %.1f ; families median %d p90 %d'%(q(T,.5),q(T,.9),q(Fm,.5),q(Fm,.9)))
