"""Read back the published sheets, model conversion, texture atlases and later-cache controls."""
import sys,pathlib,json,zipfile,io
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from models import *
from PIL import Image
import numpy as np
obj={}
for rev in (561,601,614):
 c=Cache(rev)
 with zipfile.ZipFile(ROOT/f'textures-{rev}.zip') as z:
    expected=set(c.index(9));actual={int(s[:-4]) for s in z.namelist() if s.endswith('.png')}
    assert expected==actual
    for i in actual:
        im=Image.open(io.BytesIO(z.read(f'{i}.png')));im.load();assert im.size==(128,128) and im.mode=='RGBA'
    obj[str(rev)]=len(actual)
c=Cache(242);ref=Cache(239);r=Cache(561)
converted=osrs.decode_model((ROOT/'seed-classic.bin').read_bytes());hd=decode(r.group(7,44579))
for k in ('vertices','faces','colors','face_textures','texture_coords','render_types','alpha','priorities','vertex_skins','face_skins'):
 assert hd[k]==converted[k],k
from bake import NODE
import subprocess
def target_decode(raw):
 return json.loads(subprocess.run([NODE,ROOT/'client_bridge.cjs','decode'],input=raw,stdout=subprocess.PIPE,check=True).stdout)
seed=target_decode((ROOT/'seed-classic.bin').read_bytes())
assert seed==target_decode(c.group(7,53387))
obj['target_decoder_seed_equal']=True
joined=[]
for row in json.loads((ROOT/'pixel-proof.json').read_text()):
 j=row['osrs'];a=osrs.decode_model(ref.group(7,j));b=osrs.decode_model(c.group(7,j))
 joined.append(dict(model=j,identical_242=all(a[k]==b[k] for k in ('vertices','faces','colors'))))
counts={}
for f in (ROOT/'sheets').glob('*.png'):
 im=Image.open(f).convert('RGB');assert im.width==750 and (im.height-72)%184==0
 pixels=np.asarray(im);ink=[]
 for j in range((im.height-72)//184):
    y=69+184*j+25
    ink.append([int(np.any(pixels[y:y+150,x:x+150]!=[38,42,48],axis=2).sum()) for x in (17,382)])
 counts[f.name]=ink
obj['sheet_foreground']=counts;obj['target_controls']=joined
(ROOT/'verification.json').write_text(json.dumps(obj,indent=1))
print('atlases',obj['561'],obj['601'],obj['614'],'controls',sum(r['identical_242'] for r in joined),'/',len(joined),'empty cells',[(f,j,k) for f,rs in counts.items() for j,row in enumerate(rs) for k,val in enumerate(row) if val==0])

