import sys,pathlib,json,subprocess
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from models import *
from mapping import uvmap, NODE
import numpy as np
c=Cache(614);out=[]
for mid in (59609,51518,56819,55402,733,56815):
 raw=c.group(7,mid);m=decode(raw);ours=uvmap(m)
 read=subprocess.run([NODE,ROOT/'client_bridge.cjs','rawuv'],input=raw,stdout=subprocess.PIPE,check=True)
 other=np.frombuffer(read.stdout,'<f4').reshape(ours.shape)
 valid=np.isfinite(other).all(axis=(1,2))
 mismatched=int(np.any(abs(other[valid]-ours[valid])>1e-5,axis=(1,2)).sum())
 out.append(dict(model=mid,faces=len(ours),types=sorted(set(m['texture_types'])),different_uv_faces=mismatched,undefined_reference_faces=int((~valid).sum())))
print(out)
(ROOT/'uv-check.json').write_text(json.dumps(out,indent=1))

