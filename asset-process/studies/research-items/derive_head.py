"""Fit the Meridian B shell to the standard dialogue head. Usage: derive_head.py WORN.bin 0|1 OUT.bin.
The B shell encloses an embedded stock low-detail head. Recognize its triangles by their
male kit coordinates, discard them and use the complete dialogue face with its expression labels.
Scale the shell 4x, registering the embedded face to the dialogue face by bounding-box centre."""
import pathlib,sys
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
import itemlib as L
import numpy as np
def build(worn,gender,out):
    donor=L.model((63,120)[gender])
    stock=L.model(230)
    male=L.load_bin(L.HERE.parent/'meridian-helm-v2/meridian-helm-v2-b-labelled.bin')
    m=L.load_bin(worn)
    assert np.array_equal(m['F'],male['F'])
    def signature(verts,face):return tuple(sorted(tuple(v) for v in verts[face].tolist()))
    known={signature(stock['V'],f) for f in stock['F']}
    embedded=[i for i,f in enumerate(male['F']) if signature(male['V'],f) in known]
    assert len(embedded)==121, len(embedded)
    skin=m['V'][np.unique(m['F'][embedded])]
    center=lambda a:(a.max(0)+a.min(0))/2
    shift=np.rint(center(donor['V'])-4*center(skin)).astype(int)
    keep=np.array([i for i in range(len(m['F'])) if i not in set(embedded)])
    indices,faces=np.unique(m['F'][keep],return_inverse=True)
    head=dict(donor)
    head['V']=np.vstack((donor['V'],m['V'][indices]*4+shift))
    head['F']=np.vstack((donor['F'],faces.reshape(-1,3)+len(donor['V'])))
    for k in ('colors','render_types','alpha','depth_bias','face_skins','priorities'):
        a=list(donor[k]) if donor.get(k) is not None else [0]*len(donor['F'])
        b=np.array(m[k]) if m.get(k) is not None else np.zeros(len(m['F']),int)
        head[k]=a+[int(v) for v in b[keep]]
    head['vertex_skins']=list(donor['vertex_skins'])+[15]*len(indices)
    L.save_bin(out,head)
    print(f'{out}: replaced {len(embedded)} worn face triangles by {len(donor["F"])} dialogue triangles; {len(keep)} shell triangles, rigid dialogue label 15, scale 4, translation {shift.tolist()}')
if __name__=='__main__':build(sys.argv[1],int(sys.argv[2]),sys.argv[3])

