"""Teacher-side preparation using the revision-239 tools. All scratch is local to this build."""
import sys
sys.dont_write_bytecode = True
from pathlib import Path
import json
from collections import Counter
import contextlib
import numpy as np
from PIL import Image, ImageDraw
ROOT = Path(__file__).resolve().parent
BUILDS = ROOT.parent
sys.path[:0] = [str(BUILDS/'research-tone'), str(BUILDS/'research-paint/detail')]
import tl, dlib, detail, detailgate, sheet_npc, sheet_boss, poseset, gate_npc, decomp
from tl import L, cmesh, meshio
ATTACKS = {12596:[11974,11976,11978,11981,11982],14147:[11973,11975,11977,11979,11980]}
DEATH = {12596:11992,14147:11987}
def setup(n, action):
    w = ROOT/'teacher-only'/str(n)/action
    (w/'data').mkdir(parents=True,exist_ok=True)
    tl.WORK = dlib.WORK = L.HERE = w
    cmesh.DATA = w/'data'
    tl.RENDER = dlib.RENDER = w
    L.KNOWN_SEQS[n] = dict(attack=ATTACKS[n][0],death=DEATH[n])
    return w
def write(path,data):
    path.write_text(json.dumps(data,indent=1,default=lambda o:o.item()),encoding='utf-8')
def fixtures(n,w):
    m,kv = tl.real_mesh(n)
    d = dict(tl.real_def(n),zbuffer=kv['zbuffer']=='1')
    a = tl.arrays(m)
    part = Counter(m['vertex_skins']).most_common(2)[1][0]
    chosen = (a['L'][a['F']]==part).sum(1)>=2
    for case in ['ceiling','labels','alpha','paint']:
        v=tl.clone(m)
        if case=='labels':
            common=Counter(m['vertex_skins']).most_common(1)[0][0]
            v['vertex_skins']=[common if l==part else l for l in m['vertex_skins']]
        if case=='alpha':
            v['alpha']=[0 if 0<x<255 else x for x in m['alpha']]
        if case=='paint':
            v['colors']=[c+(1 if c%128<126 else -1) if s else c for c,s in zip(m['colors'],chosen)]
        path=tl.write_variant(case,v,d,'fixtures')
        back,j=tl.ours_mesh(path)
        assert back==v
    sk=L.seq_frames(d['stand'])[2]
    names={int(t[1]):t[2] for line in (BUILDS/'research-authorship/all-names.txt').read_text().splitlines() if (t:=line.split()) and t[0]=='seq'}
    seq=[dict(s,name=names.get(s['id'],'unresolved')) for s in L.census('sequences').values() if s['skeleton']==sk]
    by=json.loads((L.CENSUS/'by-skeleton.json').read_text())
    write(w/'inventory.json',dict(npc=L.npc_def(n),model=L.census('models')[L.npc_def(n)['models'][0]],skeleton=L.census('skeletons')[sk],sharing=by.get(str(sk)),sequences=seq,alpha=Counter(m['alpha']),palette=Counter(m['colors']),part=part,paint_faces=int(chosen.sum()),alpha_by_group={str(g):dict(Counter(a['A'][a['FS']==g].tolist())) for g in np.unique(a['FS'])}))
def main():
    action,n=sys.argv[1],int(sys.argv[2])
    w=setup(n,action)
    sheet=ROOT/'sheets'/str(n)
    fix=ROOT/'teacher-only'/str(n)/'fixtures/fixtures'
    if action=='fixtures': fixtures(n,w)
    elif action=='sheet':
        sheet_npc.build(n,sheet)
        sheet_boss.build(n,sheet,old=sheet/f'sheet-{n}.json')
    elif action=='pose':
        poseset.build(n,sheet)
    elif action in ('gates','score'):
        files=[Path(sys.argv[3]).resolve()] if action=='score' else list(fix.glob('*.json'))
        codes=[]
        for path in files:
            for attack in ATTACKS[n] if path.stem=='ceiling' or action=='score' else ATTACKS[n][:1]:
                with (w/f'{path.stem}-{attack}.txt').open('w') as log,contextlib.redirect_stdout(log):
                    code,rows,result=gate_npc.gate(n,str(path),overrides=dict(attack=attack,death=DEATH[n]))
                codes.append(code)
                write(w/f'{path.stem}-{attack}.json',dict(code=code,checks=rows,cells=result['cells']))
                print(path.stem,attack,gate_npc.LEVELS[code],flush=True)
        if action=='score': sys.exit(max(codes))
    elif action=='detail':
        res=detailgate.line(detail.run(f'ceiling-{n}={n}:{fix/"ceiling.json"}'))
        detailgate.show(res)
        write(w/'detail.json',res)
    elif action=='tone':
        for attack in ATTACKS[n]:
            L.KNOWN_SEQS[n]['attack']=attack
            decomp.run(f'ceiling-{attack}',n,fix/'ceiling.json',pictures=False)
    elif action=='ordinary-alpha':
        m,kv=tl.real_mesh(n)
        m['alpha']=[0 if 0<x<254 else x for x in m['alpha']]
        path=tl.write_variant('ordinary-alpha',m,dict(tl.real_def(n),zbuffer=kv['zbuffer']=='1'),'fixtures')
        assert tl.ours_mesh(path)[0]==m
        with (w/'gate.txt').open('w') as log,contextlib.redirect_stdout(log):
            code,rows,result=gate_npc.gate(n,str(path),overrides=dict(attack=ATTACKS[n][0],death=DEATH[n]))
        write(w/'gate.json',dict(code=code,checks=rows))
if __name__=='__main__': main()

