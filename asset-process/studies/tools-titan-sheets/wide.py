"""Retain the complete silhouette in Branda's widest summon and death pictures."""
import sys
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from titan import *
n=12596;setup(n,'wide')
sh=ROOT/'sheets'/str(n); seq=L.sequences(n)
def full(row,y,cam,seq):
    for _ in range(6):
        cs,log=tl.draw([L.entry_real(n,seq)],[row],cam,y);im=cs[0][0]
        assert 'NOT APPLIED' not in log and 'NO `' not in log
        m=tl.mask(im)
        if m.any() and not any(edge.any() for edge in (m[0],m[-1],m[:,0],m[:,-1])):return Image.fromarray(im),cam
        cam=dict(cam,zoom=round(cam['zoom']*1.3))
    raise ValueError(row)
framing={}
seq['q11976']=11976
for row in ('q11976:9','q11976:20'):
    for y in (192,1216):
        im,cam=full(row,y,tl.camera(n,(540,540)),seq)
        name=f"{row.replace(':','-')}-y{y}.png"
        im.save(sh/'actions'/name);framing[name]=cam
im,cam=full('death:31',1088,tl.camera(n,(800,800)),seq)
im.save(sh/'death31-wide.png');framing['death31-wide.png']=cam
write(sh/'wide-cameras.json',framing)

