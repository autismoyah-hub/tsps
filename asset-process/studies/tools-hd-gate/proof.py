"""Generate private controls. Each damaged fixture differs from the conversion in the named way."""
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from hdgate import *

TARGETS=[('object',22772),('item-ground',7535),('item-worn',7535),('item-female',7535),('spot',2768)]
def controls(ref):
    m=combined(ref.converted)
    yield 'ceiling',m
    points=np.asarray(m['vertices']).copy()
    select=points[:,0]>np.median(points[:,0])
    points[select,0]+=max(12,np.ptp(points[:,0])//3)
    yield 'right half displaced outward',dict(m,vertices=points.tolist())
    ts=np.array(ref.r['face_textures'])
    aa=np.array(ref.r['alpha'])
    if (ts>=0).any():
        share=shares(ts,area(ref.r))
        t=max((k for k in share if k>=0),key=share.get)
        damaged=np.array(m['face_textures'])
        damaged[ts==t]=-1
        coords=np.array(m['texture_coords'])
        coords[ts==t]=-1
        yield f'HD {t} material region stripped',dict(m,face_textures=damaged.tolist(),texture_coords=coords.tolist())
        sig=signature(preview(cache(ref.rev),t)[0])
        worst=max(catalogue(),key=lambda k:np.abs(sig-catalogue()[k]).mean())
        yield f'HD {t} region painted with distant 242 texture {worst}',dict(m,face_textures=np.where(ts==t,worst,m['face_textures']).tolist())
    else:
        share=shares(aa,area(ref.r))
        a=max((k for k in share if k),key=share.get)
        yield f'alpha {a} material band stripped',dict(m,alpha=np.where(aa==a,0,aa).tolist())
    if aa.any():
        yield 'transparency removed',dict(m,alpha=[0]*len(aa))
    else:
        yield 'transparency introduced on opaque equipment',dict(m,alpha=[160]*len(aa),has_alpha=True)
    if len(ref.parts)>1:
        parts=ref.converted.copy()
        centers=[np.mean(p['vertices'],axis=0).round().astype(int) for p in parts[:2]]
        for j,p in enumerate(parts[:2]):
            parts[j]=dict(p,vertices=(np.asarray(p['vertices'])+centers[1-j]-centers[j]).tolist())
        yield 'first two assembly parts exchanged positions',combined(parts)
    elif ref.kind.startswith('item'):
        other='item-worn' if ref.kind=='item-ground' else 'item-ground'
        d,p=target(ref.rev,other,7535)
        yield 'swapped entire equipment part with '+other,combined(conversion(ref.rev,d,p)[0])
    else:
        # Exchange the two most populated animation labels. All rest pixels stay identical.
        labels=np.asarray(m['vertex_skins'])
        a,b=[x for x,n in collections.Counter(labels).most_common(2)]
        yield f'animated parts swapped (vertex labels {a} and {b})',dict(m,vertex_skins=np.where(labels==a,b,np.where(labels==b,a,labels)).tolist())

def main():
    for kind,i in TARGETS:
        if sys.argv[1:] and kind not in sys.argv[1:]:continue
        ref=Reference(614,kind,i)
        folder=ROOT/'private-proof'/kind
        folder.mkdir(parents=True,exist_ok=True)
        (folder/'mapping.json').write_text(json.dumps(ref.mapping,indent=2))
        for name,m in controls(ref):
            path=folder/(name+'.bin')
            path.write_bytes(hdview.packed([m]))
            q=ref.score(path)
            (path.with_suffix('.txt')).write_text('\n'.join(q.lines())+'\n')
            print(kind,name,q.lines()[-1],flush=True)
            if (name=='ceiling')!=(q.severity==0):
                raise RuntimeError('unexpected result '+str(path))
if __name__=='__main__':main()

