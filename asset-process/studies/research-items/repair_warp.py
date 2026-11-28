"""Repair rounding-induced folds in a same-topology female derivative, keeping measured seam points fixed.
Search only a two-unit neighbourhood of the initial prediction. Rerun pose checks after moving vertices."""
import itertools,pathlib,sys
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
import itemlib as L, female_warp as W
import numpy as np
def repair(m,f):
    F=m['F']; v=f['V'].copy(); initial=v.copy()
    normal=lambda a:np.cross(a[:,1]-a[:,0],a[:,2]-a[:,0])
    n=normal(m['V'][F]);valid=np.any(n,axis=1)
    fixed={i for i,(x,y,z) in enumerate(m['V']) if (x,-y,z) in W.SEAMS}
    def score():
        b=normal(v[F])
        dot=(n*b).sum(1)
        bad=valid & (dot<=0)
        cos=dot/np.maximum(1,np.linalg.norm(n,axis=1)*np.linalg.norm(b,axis=1))
        dup=len(v)-len(np.unique(v,axis=0))
        return (int(bad.sum()),dup,float(np.square(np.maximum(0,0.45-cos[valid])).sum()+0.02*np.square(v-initial).sum())),bad
    before=score()[0]
    for _ in range(16):
        best,bad=score()
        uniq,inv,counts=np.unique(v,axis=0,return_inverse=True,return_counts=True)
        candidates=set(F[bad].flatten())|set(np.flatnonzero(counts[inv]>1))
        move=None
        for i in sorted(candidates-fixed):
            old=v[i].copy()
            for d in itertools.product((-1,0,1),repeat=3):
                v[i]=old+d
                if np.abs(v[i]-initial[i]).max()>2:continue
                s,_=score()
                if s<best:best=s;move=(i,v[i].copy())
            v[i]=old
        if move is None:break
        v[move[0]]=move[1]
    print('non-positive normal dot / duplicate positions / shape penalty:',before,'->',score()[0])
    print('adjusted vertices',np.flatnonzero(np.any(v!=initial,axis=1)).tolist(),'; maximum per-axis correction',np.abs(v-initial).max())
    return v
if __name__=='__main__':
    male,raw,out=sys.argv[1:]
    m,f=L.load_bin(male),L.load_bin(raw)
    L.save_bin(out,f,V=repair(m,f))

