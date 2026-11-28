"""Use raw female cache meshes as file inputs and compare them to the client's real item path.
Also render rear-quarter actor/depth pairs for all female originals in walk, run and slash."""
import pathlib,sys
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from finish_items import *
from PIL import ImageChops
def controls():
    log=OUT/'pose-controls.txt';log.write_text('')
    for id in (1303,1381,1079,1147,4131,7462,1127):
        d=L.items()[id];ids=L.worn_ids(d,1)
        m=dict(L.model(ids[0])) if len(ids)==1 else L.concat(ids)
        if len(ids)>1:
            assert all(not any(t!=-1 for t in (L.model(i).get('face_textures') or [])) for i in ids)
            m['face_textures']=None
        col=list(m['colors'])
        for a,b in d['recolor']:col=[b if c==a else c for c in col]
        file=OUT/'control.bin'
        L.save_bin(file,m,V=m['V']+[0,d['wear_offset'][1],0],colors=col)
        slots='+'.join(L.SLOT_NAMES[k] for k in d['wear_positions'] if k>=0)
        args=py('fpose.py',OUT/'control.png',f'item:{id},model:{file}@{slots}','--gender',1,'--seq',819,'--frames','bind,1,4','--diff','1:2','--cell','200x250')
        stdout=run(args,log)
        if stdout.count(': 0 of 50000 pixels differ')!=3:raise RuntimeError(stdout)
    for name in ('control.png','control.bin','control.key.txt'):(OUT/name).unlink()
def review():
    stats={}
    for row in DATA:
        slug,_,slot,_,_,ref,stand,walk=row
        work=OUT/slug
        log=work/'layering.txt';log.write_text('')
        report={}
        panels=[]
        for act,seq,f in [('walk',walk,1),('run',1210 if walk==1146 else 824,3),('slash',390,3)]:
            imgs=[]
            for mode in ('actor','depth'):
                p=work/f'qa-{act}-{mode}.bmp'
                run([L.RESEARCH_POSE,L.CACHE,p,'gender=1',f'seq={seq}',f'frames={f}','yaw=1320',f'draw={mode}','cell=200x250','zoom=800' if slug in ('glaive','staff') else 'zoom=530','aim=132' if slug in ('glaive','staff') else 'aim=98','none',f'item:{ref}',f'model:{work}/female.bin@{slot}'],log)
                imgs.append(np.array(Image.open(p)))
            a,b=imgs
            report[act]=[int(np.any(a[:,i:i+200]!=b[:,i:i+200],axis=2).sum()) for i in (0,200,400)]
            fore=Image.open(work/f'1-{act}.bmp').crop((400,(0 if act=='walk' else 1)*250,600,(1 if act=='walk' else 2)*250))
            panels.extend([fore,Image.fromarray(a[:,400:]),Image.fromarray(b[:,400:])])
        sheet=Image.new('RGB',(600,810),'#292b28');d=ImageDraw.Draw(sheet)
        for j,act in enumerate(('Walk','Run','Slash')):
            for i,v in enumerate(('Front quarter','Rear quarter','Rear quarter - depth')):
                d.text((i*200+8,j*270+2),act+' / '+v,fill='white')
                sheet.paste(panels[j*3+i],(i*200,j*270+20))
        sheet.save(work/'female-motion.png')
        stats[slug]=report
    write(OUT/'layering.json',stats)
def chat():
    work=OUT/'barbute'
    for g in (0,1):
        log=work/'motion.txt'
        run(py('fpose.py',work/f'dialogue-{g}.png',f'item:1147,model:{work}/head-{g}.bin@head+hair','--gender',g,'--seq',591,'--frames','bind,1,3','--head',1,'--draw','widget','--yaw',1882,'--pitch',40,'--zoom',796,'--cell','200x200'),log)
if __name__=='__main__':
    for action in sys.argv[1:]:{'controls':controls,'review':review,'chat':chat}[action]()

