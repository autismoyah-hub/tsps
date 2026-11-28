"""Apply the local client's cylindrical, box, spherical and planar mapping formulas."""
import json,subprocess
import numpy as np
from hdcache import ROOT
from bake import NODE
def uvmap(m):
 n=len(m['faces']);shift=2 if m.get('hd',{}).get('version') in (14,15) else 0
 data=dict(faceCount=n,faceTextures=m['face_textures'],textureCoords=m['texture_coords'],
           textureFaceCount=len(m['texture_types']),textureRenderTypes=m['texture_types'])
 for i in range(3):
    data['vertices'+'XYZ'[i]]=[p[i]>>shift for p in m['vertices']]
    data['indices'+str(i+1)]=[p[i] for p in m['faces']]
 fields=['MappingP','MappingM','MappingN','ScaleX','ScaleY','ScaleZ','Rotation','Direction','Speed','TransU','TransV']
 rows=[];records=iter(m.get('hd',{}).get('mappings',[]))
 for t,tri in zip(m['texture_types'],m['texture_triangles']):
    if not t:rows.append([*tri,*([0]*8)]);continue
    r=next(records)
    sc=[s>>(shift if i<2 or t!=1 else 0) for i,s in enumerate(r['scale'])]
    signed=lambda b:(b+128)%256-128
    rows.append([*r['axis'],*sc,signed(r['rotation']),signed(r['direction']),signed(r['speed']),*[signed(v) for v in (r['cube'] or [0,0])]])
 for i,f in enumerate(fields):data['texture'+f]=[row[i] for row in rows]
 out=subprocess.run([NODE,ROOT/'client_bridge.cjs','uv'],input=json.dumps(data).encode(),stdout=subprocess.PIPE,check=True)
 uv=np.frombuffer(out.stdout,'<f4').reshape(n,3,2).copy()
 # A degenerate mapping plane has no inverse. Preserve the input, expose fallback faces in metadata.
 invalid=~np.isfinite(uv).all(axis=(1,2))
 m['degenerate_uv_faces']=np.flatnonzero(invalid).tolist()
 uv[invalid]=[[0,0],[1,0],[0,1]]
 return uv

