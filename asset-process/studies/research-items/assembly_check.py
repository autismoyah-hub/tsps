"""Check adjoining pieces in the full outfit and compare the cape re-encode to the game path."""
import pathlib,sys
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from finish_items import *
from PIL import ImageChops
def main():
    log=OUT/'assembly.txt';log.write_text('')
    # Separately reproduce inherited red cape equivalence on four run frames.
    run(py('fpose.py',OUT/'cape-check.png',f'item:1007,model:{P}/capes/red-cape-reencoded.bin@cape,none','--seq',824,'--frames','bind,1,3,5','--yaw',1024,'--diff','1:2','--cell','300x400'),log)
    for n in ('q1.py','q2.py','q3.py','q4.py'):
        run(py(n),log)
    for g in (0,1):
        slots={}; spec=[]
        for row in DATA:
            slug,_,s,*_=row
            f=source(row) if g==0 else OUT/slug/'female.bin'
            if s=='weapon' and slug!='longsword':continue
            slot=s.split('+')[0]
            m=L.load_bin(f)
            slots[slot]={tuple(v):int(l) for v,l in zip(m['V'].tolist(),m['vertex_skins'])}
            spec.append(f'model:{f}@{s}')
        with log.open('a') as fp:
            for a,b in [('head','body'),('body','hands'),('body','legs'),('legs','feet')]:
                shared=sorted(slots[a].keys() & slots[b].keys())
                print('gender',g,a,b,'exact common rest coordinates',len(shared),[(p,slots[a][p],slots[b][p]) for p in shared],file=fp)
        for act,seq in [('walk',819),('run',824),('slash',390)]:
            run(py('fpose.py',OUT/f'set-{g}-{act}.png',f'item:1163+item:1127+item:7462+item:1079+item:4131+item:1303,'+'+'.join(spec),'--seq',seq,'--frames','1,3,5','--gender',g,'--cell','260x280','--zoom',630,'--yaw',192),log)
    print(log)
if __name__=='__main__':main()

