"""Surface correspondence for material use and cutaway inspection, independent of face order."""
import numpy as np
from scipy.spatial import cKDTree

def probes(m):
    tri=np.asarray(m['vertices'],float)[m['faces']]
    normal=np.cross(tri[:,1]-tri[:,0],tri[:,2]-tri[:,0])
    normal/=np.maximum(1e-12,np.linalg.norm(normal,axis=1,keepdims=True))
    # Four interior samples per triangle. Normals disambiguate opposite sides of thin surfaces.
    bary=np.array([[1/3]*3,[.6,.2,.2],[.2,.6,.2],[.2,.2,.6]])
    points=np.einsum('sk,fkd->fsd',bary,tri)
    return np.concatenate((points,np.broadcast_to(normal[:,None],points.shape)),axis=2).reshape(-1,6)

def correspond(real,ours):
    a,b=probes(real),probes(ours)
    return (cKDTree(b).query(a)[1].reshape(-1,4)//4,
            cKDTree(a).query(b[::4])[1]//4)

def isolate(m,keep):
    return dict(m,render_types=np.where(keep,m['render_types'],2).tolist(),
                alpha=np.where(keep,m['alpha'],255).tolist())

