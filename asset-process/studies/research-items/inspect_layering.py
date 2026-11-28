"""Enlarge the rear-quarter run-frame discrepancy for the female legs, keeping actual pixel boundaries."""
import pathlib,sys
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from finish_items import *
from scipy.ndimage import label,find_objects
p=OUT/'legs'
a,b=[np.array(Image.open(p/f'qa-run-{s}.bmp'))[:,400:] for s in ('actor','depth')]
m=np.any(a!=b,axis=2);ids,n=label(m)
regions=sorted(zip(np.bincount(ids.ravel())[1:],find_objects(ids)),key=lambda pair:-pair[0])
out=Image.new('RGB',(600,236),'#272923');draw=ImageDraw.Draw(out)
heat=a.copy();heat[m]=[255,76,79]
for i,(v,title) in enumerate(zip((a,b,heat),('Actor','Depth','Changed pixels'))):
    out.paste(Image.fromarray(v).crop((75,67,141,137)).resize((198,210),Image.Resampling.NEAREST),(i*200,26))
    draw.text((i*200+6,6),title,fill='white')
out.save(p/'run-layering.png')
msg=f'{int(m.sum())} whole-cell pixels differ. Largest four-connected patches (pixels, y interval, x interval, exclusive upper bound):\n'
for size,(y,x) in regions[:5]:msg+=f'{size} [{y.start},{y.stop}) [{x.start},{x.stop})\n'
print(msg)
(p/'run-layering.txt').write_text(subprocess.list2cmdline([sys.executable,'-I',__file__])+'\n'+msg)

