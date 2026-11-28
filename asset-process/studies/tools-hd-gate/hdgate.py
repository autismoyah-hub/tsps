"""Teacher-only comparison against one HD presentation of a rebuild."""
import argparse
import collections
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from hdasset import *
import compare
from region import correspond,isolate
from PIL import ImageDraw

BG=(38,42,48)
LEVELS=('PASS','BORDERLINE','FAIL')
class Checks:
    def __init__(self):
        self.rows=[]
        self.severity=0
    def check(self,name,value,good,edge,lower=True):
        level=(0 if value<=good else 1 if value<=edge else 2) if lower else (0 if value>=good else 1 if value>=edge else 2)
        self.severity=max(self.severity,level)
        op='<=' if lower else '>='
        self.rows.append(f'{name} | {value:.5f} | PASS {op} {good:.5f}, BORDERLINE {op} {edge:.5f} | {LEVELS[level]}')
    def info(self,name,value):
        self.rows.append(f'{name} | {value} | INFO')
    def lines(self):
        return self.rows+['VERDICT '+LEVELS[self.severity]]

def aligned(m,worn):
    v=np.asarray(m['vertices'])
    mid=(v.max(0)+v.min(0))/2
    mid[1]=v.max(0)[1]
    return dict(m,vertices=(v-(0 if worn else mid)).tolist())

def overlap(a,b):
    return 1. if not (a|b).any() else float((a&b).sum()/(a|b).sum())

def geometry(q,r,u,worn):
    for name,m in [('real',r),('ours',u)]:
        duplicate=sum(n for n in collections.Counter(map(tuple,m['vertices'])).values() if n>1)
        q.info(name+' vertices / faces / shared-position vertices / nonzero shading type',
               [len(m['vertices']),len(m['faces']),duplicate,int(np.count_nonzero(m['render_types']))])
    for axis,error in zip('xyz',abs(np.ptp(r['vertices'],axis=0)-np.ptp(u['vertices'],axis=0))):
        q.check('size '+axis+' absolute error',error,1,2)
    q.check('face count error %',100*abs(len(u['faces'])/len(r['faces'])-1),10,15)
    cr,cu=compare.colour_shares(r),compare.colour_shares(u)
    q.check('distinct painted colours',len(cu),max(len(cr)+1,(3*len(cr)+1)//2),2*len(cr))
    q.check('area-weighted HSL transport',compare.colour_distance(cu,cr),2.5,3.5)
    a,b=aligned(r,worn),aligned(u,worn)
    v=np.asarray(a['vertices']+b['vertices'])
    for view in compare.VIEWS:
        masks=[compare.silhouette(m,view,v.min(0),384/max(1,np.ptp(v,axis=0).max()),400) for m in (a,b)]
        q.check(view+' outline IoU',overlap(*masks),.93 if worn else .97,.90 if worn else .95,False)

def surface(q,r,u):
    def fractions(m):
        w=area(m)
        textured=np.array(m['face_textures'])>=0
        alpha=np.array(m['alpha'])
        return np.array([w[k].sum()/w.sum() for k in (textured,alpha!=0,textured&(alpha!=0))]),shares(alpha,w)
    (sr,ar),(su,au)=map(fractions,(r,u))
    for name,x,y in zip(('textured','translucent','both'),sr,su):
        q.check(f'{name} 3D area share error (HD {x:.4f}, ours {y:.4f})',abs(x-y),.04,.07)
        if x>0 and y==0 or x==0 and y>.01:q.check(name+' channel presence mismatch',1,0,0)
    # Earth mover on raw alpha bytes. Measures distance even when labels shift a few steps.
    drift=np.cumsum([ar.get(i,0)-au.get(i,0) for i in range(256)])
    q.check('alpha area transport (byte steps)',abs(drift).sum(),5,8)

class Reference:
    def __init__(self,rev,kind,i):
        self.rev,self.kind=rev,kind
        self.d,self.parts=target(rev,kind,i)
        self.r=combined(self.parts)
        self.cam=framing(self.r['vertices'])
        self.draw=render(self.r,rev,self.cam,('lit','ids','alpha'))
        self.converted,self.mapping=conversion(rev,self.d,self.parts)
        self.ceiling=render(combined(self.converted),242,self.cam)['lit']
        self.motion=None
        if kind=='spot' and 'sequence' in self.d:
            _,poses=animation(rev,self.d['sequence'],self.r)
            self.anim_cam=framing([v for m in poses for v in m['vertices']])
            self.motion=[render(m,rev,self.anim_cam)['lit'][[0,5]] for m in poses]

    def score(self,path,out=None):
        raw=path.read_bytes()
        u=osrs.decode_model(raw)
        q=Checks()
        geometry(q,self.r,u,self.kind in ('item-worn','item-female'))
        surface(q,self.r,u)
        # Share framing between models, including body-space offsets.
        got=render(u,242,self.cam,('lit','ids','alpha'),raw=raw)
        real=self.draw
        by_id=real['ids']
        matches,reverse=correspond(self.r,u)
        weights=area(self.r)
        ids=np.asarray(self.r['face_textures'])
        sampled=np.asarray(u['face_textures'])[matches]
        for t in sorted(set(self.r['face_textures'])):
            faces=ids==t
            if weights[faces].sum()>0:
                assigned=((sampled[faces]>=0)==(t>=0)).mean(1)
                q.check(f'HD {t} 3D region texture presence by area',
                        np.average(assigned,weights=weights[faces]),.83,.72,False)
            mask=by_id==t
            if not mask.any():
                # Remove other materials from both models to reveal interior-facing triangles.
                cam=framing(np.asarray(self.r['vertices'])[np.asarray(self.r['faces'])[faces]].reshape(-1,3))
                cut=render(isolate(self.r,faces),self.rev,cam,('cut-ids','cut-lit'))
                selected=cut['cut-ids']==t
                if selected.any():
                    actual=render(isolate(u,faces[reverse]),242,cam,('cut-lit',))['cut-lit']
                    control=render(isolate(combined(self.converted),faces),242,cam,('cut-lit',))['cut-lit']
                    def error(arr):
                        return np.abs(cut['cut-lit'][selected].astype(float)-arr[selected]).mean()
                    base=error(control)
                    q.check(f'HD {t} cutaway RGB MAE (conversion {base:.2f}, {selected.sum()} px)',error(actual),base+10,base+18)
                else:q.info(f'HD {t} region','no front-facing pixels even in isolation')
                continue
            coverage=((got['ids']>=0) if t>=0 else (got['ids']==-1))[mask].mean()
            q.check(f'HD {t} projected region correct texture presence ({mask.sum()} px)',coverage,.83,.72,False)
            regions=[(f'HD {t}',mask)] if t>=0 else [(f'paint alpha {a}',mask&(real['alpha']==a)) for a in np.unique(real['alpha'][mask])]
            for name,region in regions:
                delta=np.abs(real['lit'][region].astype(float)-got['lit'][region]).mean()
                floor=np.abs(real['lit'][region].astype(float)-self.ceiling[region]).mean()
                q.check(f'{name} RGB MAE (conversion {floor:.2f}, {region.sum()} px)',delta,floor+10,floor+18)
            alpha_error=np.abs(got['alpha'][mask]-real['alpha'][mask]).mean()
            q.check(f'HD {t} visible alpha MAE',alpha_error,10,18)
        if self.motion is not None:
            _,poses=animation(self.rev,self.d['sequence'],u)
            ious=[]
            for ref,m in zip(self.motion,poses):
                images=render(m,242,self.anim_cam)['lit'][[0,5]]
                ious.extend(overlap((a!=BG).any(2),(b!=BG).any(2)) for a,b in zip(ref,images))
            q.check('reference sequence posed IoU mean',np.mean(ious),.78,.70,False)
            q.check('reference sequence posed IoU worst',min(ious),.50,.40,False)
        if out:
            out.mkdir(parents=True,exist_ok=True)
            from hdsheet import page,ORDER
            imgs=[Image.fromarray(frame[i]) for frame in (real['lit'],got['lit']) for i,_ in ORDER]
            page(imgs,[s+' '+label for s in ('HD','242') for _,label in ORDER],'Teacher comparison','Top two rows HD, bottom two rows rebuild',out/'comparison.png')
            (out/'score.txt').write_text('\n'.join(q.lines())+'\n')
        return q

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('command',choices=['rebuild'])
    p.add_argument('rev',type=int,choices=(561,601,614))
    p.add_argument('kind',choices=KINDS)
    p.add_argument('id',type=int)
    p.add_argument('ours',type=Path)
    p.add_argument('--out',type=Path)
    a=p.parse_args()
    ref=Reference(a.rev,a.kind,a.id)
    try:
        q=ref.score(a.ours,a.out)
    except (ValueError,KeyError,IndexError,subprocess.CalledProcessError) as e:
        print('candidate decode or render |',e,'| FAIL\nVERDICT FAIL')
        raise SystemExit(2)
    print('\n'.join(q.lines()))
    raise SystemExit(q.severity)

