"""Prepare item-specific inventory geometry before the shared poser and icon fitter.
Usage: inventory_parts.py longsword|glaive|barbute|gauntlets INPUT.bin OUTPUT.bin"""
import pathlib,sys,collections
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import itemlib as L

def select(m,faces):
    vertices,indices=np.unique(m['F'][faces],return_inverse=True)
    n=dict(m,V=m['V'][vertices],F=indices.reshape(-1,3))
    for key in ('colors','priorities','alpha','face_skins','render_types','depth_bias','face_textures'):
        if m.get(key) is not None:n[key]=np.array(m[key])[faces].tolist()
    n['vertex_skins']=None
    return n
def fill(m,slug):
    edges=collections.defaultdict(list)
    for i,tri in enumerate(m['F']):
        for a,b in zip(tri,np.roll(tri,-1)):
            edges[tuple(sorted((a,b)))].append((a,b,i))
    center=np.array([-30,-96 if slug=='longsword' else -97,0])
    rim=[v[0] for v in edges.values() if len(v)==1 and all(np.linalg.norm(m['V'][j]-center)<6.5 for j in v[0][:2])]
    by_start={a:(b,i) for a,b,i in rim}
    cycles=[]
    for a,_,_ in rim:
        if a in by_start:
            loop=[];v=a
            while v in by_start:
                loop.append(v);v,_=by_start.pop(v)
            if v==a and len(loop)==4:cycles.append(loop)
    if slug=='longsword':
        cycles=[loop for loop in cycles if m['V'][loop,2].tolist() in ([-2]*4,[4]*4)]
    assert len(cycles)==2, (slug,rim,cycles)
    a,b=cycles
    # Adjacent boundary windings run in opposite directions. Align corners by distance.
    b=b[::-1]
    b=min((b[n:]+b[:n] for n in range(4)),key=lambda x:np.linalg.norm(m['V'][a]-m['V'][x]))
    new=[];attributes=[]
    for k in range(4):
        j=(k+1)%4
        new.extend([[a[j],a[k],b[k]],[a[j],b[k],b[j]]])
        attributes.extend([edges[tuple(sorted((a[k],a[j])))][0][2]]*2)
    m['F']=np.vstack((m['F'],new))
    for key in ('colors','priorities','alpha','face_skins','render_types','depth_bias','face_textures'):
        if m.get(key) is not None:m[key]=list(m[key])+[m[key][i] for i in attributes]
    print(f'{slug}: joined the two open four-corner hand rings with eight wood triangles')
def main():
    slug,path,out=sys.argv[1:]
    m=L.load_bin(path)
    if slug in ('longsword','glaive'):fill(m,slug)
    if slug=='barbute':
        kit=L.model(230)
        sig=lambda v,f:tuple(sorted(tuple(p) for p in v[f].tolist()))
        body={sig(kit['V'],f) for f in kit['F']}
        keep=[i for i,f in enumerate(m['F']) if sig(m['V'],f) not in body]
        assert len(m['F'])-len(keep)==121
        m=select(m,keep)
        print('barbute: removed all 121 embedded body-kit triangles')
    if slug=='gauntlets':
        # Turn the cuffs toward one another; leave two separately readable hand silhouettes.
        for side in (-1,1):
            mask=m['V'][:,0]*side>0
            xy=m['V'][mask,:2].astype(float);c=xy.mean(0)
            theta=np.deg2rad(side*45)
            rot=np.array([[np.cos(theta),-np.sin(theta)],[np.sin(theta),np.cos(theta)]])
            m['V'][mask,:2]=np.rint((xy-c)@rot+c+[0,4*side]).astype(int)
        print('gauntlets: splayed the hands 45 degrees and staggered their cuffs')
    L.save_bin(out,m)
if __name__=='__main__':main()

