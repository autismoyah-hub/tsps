import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, numpy as np
import itemlib as L
male=dict(hair=230,jaw=249,body=28515,arms=26632,hands=176,legs=28285,feet=181)
female=dict(hair=403,jaw=456,body=18554,arms=3476,hands=353,legs=14423,feet=358)
for k in male:
    a,b=L.model(male[k]),L.model(female[k])
    A=a['V']*[1,-1,1]; B=b['V']*[1,-1,1]
    same=len(A)==len(B) and np.array_equal(a['F'],b['F'])
    print(f'== {k}: male {len(A)}v {len(a["F"])}f box {A.min(0).tolist()}..{A.max(0).tolist()} | female {len(B)}v {len(b["F"])}f box {B.min(0).tolist()}..{B.max(0).tolist()} same topology {same}; priorities m {sorted(set(a["priorities"]))} f {sorted(set(b["priorities"]))}')
    if same:
        print('   labels equal', list(a['vertex_skins'])==list(b['vertex_skins']))
        for i in np.argsort(-A[:,1]):
            print(f'   {a["vertex_skins"][i]:3d} m {A[i].tolist()} -> f {B[i].tolist()}  d {(B[i]-A[i]).tolist()}')
    else:
        for name,m,V in (('male',a,A),('female',b,B)):
            by=collections.defaultdict(list)
            for v,l in zip(V,m['vertex_skins']): by[l].append(v.tolist())
            print('  ',name)
            for l in sorted(by):
                c=np.mean(by[l],0).round(1).tolist()
                print(f'      label {l}: n={len(by[l])} centroid {c} ' + (str(sorted(by[l],key=lambda p:(-p[1],p[0],p[2]))) if len(by[l])<=10 else ''))
