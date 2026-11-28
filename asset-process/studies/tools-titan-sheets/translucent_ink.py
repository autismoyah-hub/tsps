"""Rest light and paint boundaries on the foremost translucent surface."""
import sys
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from supplement import *
n=int(sys.argv[1]);setup(n,'translucent-ink')
sh=ROOT/'sheets'/str(n); meta=json.loads((sh/'sheet.json').read_text())
a=tl.arrays(tl.real_mesh(n)[0]);d=tl.real_def(n)
cl,lit=lighting(a,d['ambient'],d['contrast'])
for (name,c),view in zip(ortho.VIEWS,meta['views']):
    fr=(*view['origin'],*view['size'])
    _,(ids,ba)=sheet_boss.layers(a,c,fr,meta['pixels_per_unit'])
    ok=ids>=0
    ordinary=ok&(a['A'][ids]<254)
    ln=ortho.interp(ids,ba,cl)
    pixels=np.zeros(ids.shape+(2,),np.uint8)
    pixels[...,0]=np.clip(np.rint(np.nan_to_num(ln)),0,255)
    pixels[...,1]=ordinary*255
    Image.fromarray(pixels,'LA').save(sh/f'views/{name}-tlight.png')
    paint=a['C'][ids];raw=a['A'][ids]
    ink=np.full(ids.shape+(3,),[16,18,22],dtype=np.uint8);ink[ok]=[44,48,54]
    for left,right in [(np.s_[:,:-1],np.s_[:,1:]),(np.s_[:-1,:],np.s_[1:,:])]:
        both=ok[left]&ok[right];screen=ink[left]
        screen[ok[left]^ok[right]]=[150,154,162]
        screen[both&(raw[left]!=raw[right])]=[244,177,64]
        screen[both&(paint[left]!=paint[right])]=255
    Image.fromarray(ink).save(sh/f'views/{name}-tink.png')

