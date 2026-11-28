"""Render a synthetic textured, weighted wedge. No game geometry is used."""
import sys,pathlib,tempfile,json
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3view import *
with tempfile.TemporaryDirectory(dir=ROOT) as tmp:
    p=pathlib.Path(tmp)
    im=Image.new('RGBA',(2,2))
    im.putdata([(255,255,255,255),(10,135,220,220),(30,180,140,255),(244,192,38,255)])
    im.save(p/'1.png')
    data=dict(name='Synthetic inspection fixture',vertices=[[0,0,0],[16,0,0],[0,20,0],[0,0,12]],faces=[[0,2,1],[0,1,3],[0,3,2],[1,2,3]],rgba=[[1,1,1,1],[1,.2,.1,1],[.1,1,.4,1],[.7,.8,1,.9]],alpha=[0,32,0,96],material=[1,-1,1,1],uv=[[[0,0],[1,0],[0,1]]]*4,weights=[[[0,255]],[[0,128],[1,127]],[[1,255]],[]])
    (p/'fixture.json').write_text(json.dumps(data))
    run(p/'fixture.json',ROOT/'viewer-check.png',ROOT/'viewer-check.blend')

