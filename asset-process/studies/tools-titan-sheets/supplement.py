"""Version-three diagrams. Render only aggregate paint, mark and phase information."""
import sys
from pathlib import Path
sys.dont_write_bytecode = True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from titan import ROOT, BUILDS, ATTACKS, DEATH, setup, write, tl, L, cmesh, poseset, sheet_boss, dlib
import numpy as np
import json
from collections import Counter
from PIL import Image,ImageDraw
import ortho, layers, light_npc

def lighting(a,ambient,contrast):
    cl,lit=original(a,ambient,contrast)
    fs=[(*f.tolist(),int(c),int(t),int(al),-1) for f,c,t,al in zip(a['F'],a['C'],a['T'],a['A'])]
    _,norm=light_npc.normals(a['V'].astype(int).tolist(),fs)
    den=light_npc.denominator(850+contrast*5)
    for i in np.flatnonzero(a['T']==1):
        ln=ambient+64+light_npc.tdiv(sum(v*d for v,d in zip(norm[i],light_npc.DIRECTION)),den+den//2)
        cl[i]=ln
        lit[i]=np.clip((int(a['C'][i])%128*ln)>>7,2,126)
    return cl,lit
original=sheet_boss.corner_lit
def board(n,sh):
    d=L.sequences(n)
    cam=dict(cell=(500,560),zoom=1350,aim=420,pitch=128)
    board=Image.new('RGB',(1000,3*592),(31,32,37))
    for col,npc in enumerate([n,7541]):
        seq=L.sequences(npc)
        if npc==7541:
            tek=json.loads((BUILDS/'research-tone/sheets/tekton/pose.json').read_text())
            seq=tek['sequences']
        seq['special']=ATTACKS[n][3] if npc==n else seq['attack']
        rows=['bind',f"attack:{L.seq_frames(seq['attack'])[0]//3}",f"special:{2*L.seq_frames(seq['special'])[0]//3}"]
        cs,_=tl.draw([L.entry_real(npc,seq)],rows,cam,192)
        for i,(row,c) in enumerate(zip(rows,cs)):
            im=Image.fromarray(c[0])
            body=tl.mask(c[0])
            assert body.any() and not any(v.any() for v in (body[0],body[-1],body[:,0],body[:,-1]))
            if npc==n: im.save(sh/f'comparison-{i}.png')
            board.paste(im,(col*500,i*592+32))
            role=row.split(':')[0]
            cap='Bind' if role=='bind' else f"Sequence {seq[role]}, frame {row.split(':')[1]}"
            ImageDraw.Draw(board).text((col*500+12,i*592+8),f'{L.npc_def(npc)["name"]}    {cap}',fill='white')
    board.save(ROOT/f'board-{n}-tekton.png')
def gallery(n,sh):
    inv=json.loads((ROOT/'teacher-only'/str(n)/'fixtures/inventory.json').read_text())
    names=json.loads((ROOT/'teacher-only'/str(n)/'sequence-names.json').read_text())
    seq={f'q{s["id"]}':s['id'] for s in inv['sequences'] if not names[str(s['id'])].startswith('loc_')}
    rows=[f'{k}:{int((L.seq_frames(v)[0]-1)*frac)}' for k,v in seq.items() for frac in [.33,.67]]
    out=sh/'actions';out.mkdir(exist_ok=True)
    for y in [192,1216]:
        for off in range(0,len(rows),6):
            group=rows[off:off+6]
            cells,msg=tl.draw([L.entry_real(n,seq)],group,tl.camera(n,(540,540)),y)
            assert 'NOT APPLIED' not in msg and 'NO `' not in msg
            for row,cs in zip(group,cells):
                Image.fromarray(cs[0]).save(out/f"{row.replace(':','-')}-y{y}.png")
    board(n,sh)
def diagrams(n,sh):
    m,_=tl.real_mesh(n); a=tl.arrays(m);d=tl.real_def(n)
    cache=tl.WORK/'visibility.npy'
    if cache.exists():
        cls=np.load(cache)
    else:
        cls=layers.table('reference',m,layers.vis(m,d,f'phase-{n}',sh))
        np.save(cache,cls)
    _,marks=dlib.mesh_marks(a,d['scale'][0])
    mark=np.zeros(len(a['F']),int)
    for j,mk in enumerate(marks,1):mark[mk['faces']]=j
    meta=json.loads((sh/'sheet.json').read_text())
    style=cmesh.measure(cmesh.load(f'npc:{n}'),tuple(d['scale']))
    meta['style_targets']={k:round(style[k],3) for k in ('fold_med_deg','fold_over30_share','detail_islands','colour_edge_share','valence_6_share','faces_for_half_surface')}
    meta['faces_by_pose']=dict(Counter(cls.tolist()))
    best={}
    folder=sh/'phases';folder.mkdir(exist_ok=True)
    maskdir=sh/'marks';maskdir.mkdir(exist_ok=True)
    lo,hi=np.array(meta['bounds_min']),np.array(meta['bounds_max'])
    for phase in sorted(set(cls)-{'undrawn'}):
        sel=(cls==phase)&tl.drawn(a)
        for name,c in ortho.VIEWS[:10]:
            px=1.5;fr=ortho.frame(lo,hi,c,px)
            ids,_,_=ortho.raster(a['V'],a['F'],c,fr,px,np.flatnonzero(sel),a['D']*.5)
            ok=ids>=0
            rgb=tl.TABLE[a['C'][ids]].astype('uint8');rgb[~ok]=[42,44,50]
            fn=f"{phase.replace(' ','-')}-{name}"
            Image.fromarray(rgb).save(folder/f'{fn}.png')
            marks_here=np.where(ok,mark[ids],0)
            Image.fromarray(marks_here.astype('uint16')).save(folder/f'{fn}-marks.png')
            for j,num in zip(*np.unique(marks_here,return_counts=True)):
                if not j or (j in best and best[j][0]>=num):continue
                yy,xx=np.where(marks_here==j)
                x0,x1=max(0,xx.min()-12),min(fr[2],xx.max()+13)
                y0,y1=max(0,yy.min()-12),min(fr[3],yy.max()+13)
                sl=np.s_[y0:y1,x0:x1]
                bw=np.uint8(marks_here[sl]==j)*255
                best[j]=(num,fn,[int(x0),int(y0)],rgb[sl].copy(),bw)
    index=[]
    for j,mk in enumerate(marks,1):
        if j not in best:
            # Expose an occluded mark by drawing its paint region alone in its broadest axis view.
            choices=[]
            for name,c in ortho.SIX:
                fr=ortho.frame(lo,hi,c,1.5)
                ids,_,_=ortho.raster(a['V'],a['F'],c,fr,1.5,mk['faces'],cull=False)
                yy,xx=np.where(ids>=0)
                if len(yy):
                    y0,x0=int(yy.min()),int(xx.min())
                    bw=np.uint8(ids[y0:yy.max()+1,x0:xx.max()+1]>=0)*255
                    rgb=np.where(bw[...,None]>0,tl.TABLE[mk['colour']],[42,44,50]).astype('uint8')
                    choices.append((len(yy),name,[x0,y0],rgb,bw))
            if not choices:
                index.append(dict(mark=j,paint=mk['colour'],pixels=0)); continue
            best[j]=max(choices,key=lambda v:v[0])
        num,view,origin,rgb,bw=best[j]
        stem=f'{j:03d}'
        Image.fromarray(bw).convert('1').save(maskdir/f'{stem}-mask.png')
        Image.fromarray(rgb).save(maskdir/f'{stem}-paint.png')
        index.append(dict(mark=j,view=view,origin_pixels=origin,pixels=int(num),paint=mk['colour'],translucent=mk['translucent']))
    write(maskdir/'index.json',index)
    meta['sheet_version']=3
    meta['detail_marks']=len(index)
    for path in [sh/f'sheet-{n}.json',sh/'sheet.json']:
        dat=meta if path.name=='sheet.json' else json.loads(path.read_text())
        for lab in dat['labels'].values():
            lab.pop('min',None);lab.pop('max',None)
            if lab['vertices']<=1:lab.pop('centroid',None)
        write(path,dat)

def closeups(n,sh):
    out=sh/'enlargements'
    for nm,cx,cy in ([('head',.40,.22),('hand-weapon',.16,.74),('hand-grip',.62,.70),('crown-flame',.40,.10)] if n==12596 else [('head',.47,.47),('hand-weapon',.23,.80),('hand-grip',.64,.79),('floating-ice',.47,.18)]):
        ims=[]
        for k in ('paint','lit','tpaint'):
            im=Image.open(sh/f'views/front-{k}.png').convert('RGBA')
            if k!='tpaint':im.alpha_composite(Image.open(sh/f"views/front-{'tpaint' if k=='paint' else 'tlit'}.png"))
            bg=Image.new('RGB',im.size,(55,58,65));bg.paste(im,mask=im.getchannel('A'))
            W,H=im.size; side=int(W*(.94 if nm=='floating-ice' else .36))
            x=int(np.clip(W*cx-side/2,0,W-side));y=int(np.clip(H*cy-side/2,0,H-side))
            ims.append(bg.crop((x,y,x+side,y+side)).resize((460,460),Image.Resampling.NEAREST))
        strip=Image.new('RGB',(1380,460))
        for i,im in enumerate(ims):strip.paste(im,(460*i,0))
        strip.save(out/f'{nm}.png')
    for name,_ in ortho.VIEWS:
        t=np.array(Image.open(sh/f'views/{name}-tpaint.png'))[...,3]
        opa=np.array(Image.open(sh/f'views/{name}-paint.png'))[...,3]>0
        pic=np.zeros((*t.shape,3),np.uint8);pic[:]=[30,32,38];pic[opa]=[79,80,83]
        visible=t>0
        raw=255-t
        pic[visible]=np.stack([raw,np.full_like(t,190),255-raw],axis=-1)[visible]
        Image.fromarray(pic).save(sh/f'views/{name}-alpha.png')

if __name__=='__main__':
    action,n=sys.argv[1],int(sys.argv[2]);setup(n,action);sh=ROOT/'sheets'/str(n)
    if action=='gallery':gallery(n,sh)
    elif action=='board':board(n,sh)
    elif action=='flat':
        sheet_boss.corner_lit=lighting
        sheet_boss.build(n,sh,old=sh/f'sheet-{n}.json')
        closeups(n,sh)
    else:diagrams(n,sh)

