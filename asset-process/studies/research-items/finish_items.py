"""Build review packages and record each child command, working directory and output.
Run with the specified isolated Python. Stages: verify, assets [slug ...], motion [slug ...].
All IDs in the manifests are local preview IDs."""
import sys, pathlib, json, subprocess, statistics, collections, hashlib
P = pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(P))
import itemlib as L, itemcodec as C, female_warp as W, make_inventory as I
import numpy as np
from PIL import Image, ImageDraw, ImageFont
ROOT=P.parents[1]
OUT=P/'completed'
OUT.mkdir(exist_ok=True)
RENDER=ROOT/'render'/'research-items-meridian'
RENDER.mkdir(exist_ok=True)
DATA=[
 ('longsword','sword','weapon','weapon','longsword',1303,808,819),
 ('glaive','glaive','weapon','weapon','halberd',3204,809,1146),
 ('staff','staff','weapon','weapon','staff',1381,813,1146),
 ('cuirass','cuirass-v2','body+arms','lay','platebody',1127,808,819),
 ('gauntlets','gauntlets','hands','gloves','gloves/gauntlets',7462,808,819),
 ('legs','legs','legs','lay','platelegs',1079,808,819),
 ('boots','boots','feet','upright','boots',4131,808,819),
 ('barbute','helm-v2','head+hair','upright','med helm',1147,808,819)]
def run(args,log):
    args=list(map(str,args))
    result=subprocess.run(args,cwd=P,capture_output=True,text=True,errors='replace')
    with log.open('a',encoding='utf8') as f:
        f.write('\nPS '+str(P)+'> '+subprocess.list2cmdline(args)+'\n'+result.stdout+result.stderr+'exit '+str(result.returncode)+'\n')
    if result.returncode: raise RuntimeError(result.stderr+result.stdout)
    return result.stdout
def py(name,*args): return [*L.PY,P/name,*args]
def write(path,data): path.write_text(json.dumps(data,indent=2),encoding='utf8')
def source(row):
    slug,folder,slots,*_=row
    suffix='' if slots=='weapon' else '-b-labelled' if slug=='barbute' else '-labelled'
    return ROOT/'builds'/('meridian-'+folder)/('meridian-'+folder+suffix+'.bin')
def verify():
    log=OUT/'verification.txt'
    log.write_text('')
    run([L.RESEARCH_ITEMS,L.CACHE,'verify',1303,3204,1381,1127,7462,1079,4131,1147,6570,9747],log)
    run(py('itemcodec.py','roundtrip'),log)
    run(py('q5.py'),log)
    run(py('female_warp.py','check','head','body','legs','hands','feet','--n',8),log)
    run(py('capes/cape_measure.py'),log)
def asset(row):
    slug,folder,slots,pose,kind,ref,stand,walk=row
    idx=DATA.index(row)
    work=OUT/slug
    work.mkdir(exist_ok=True)
    log=work/'build.txt'
    log.write_text('')
    male=source(row)
    female=work/'female.bin'
    inv=work/'inventory.bin'
    rowids=json.loads((P/'data/kinds.json').read_text())[kind][:8]
    if slug=='boots':rowids=[4119,4121,4123,4125,4127,4129,4131,11840]
    if slug=='gauntlets':rowids=list(range(7455,7463))
    if slug=='staff':rowids=[1379,1381,1383,1385,1387,1391,1393,1395]
    raw=female if slots=='weapon' else work/'female-first-pass.bin'
    run(py('female_warp.py','derive',male,slots.split('+')[0],raw),log)
    if raw!=female:
        run(py('repair_warp.py',male,raw,female),log)
    ingredient=male
    if slug in ('longsword','glaive','barbute','gauntlets'):
        ingredient=work/'inventory-source.bin'
        run(py('inventory_parts.py',slug,male,ingredient),log)
    run(py('make_inventory.py',ingredient,pose,inv,'--like',ref,'--row',','.join(map(str,rowids))),log)
    run(py('fit_row.py',inv),log)
    view=json.loads(inv.with_suffix('.view.json').read_text())
    base=1000100+idx*10
    models={base:inv,base+1:male,base+2:female}
    spec={k:view[k] for k in ('zoom','angles','offset','ambient','contrast')}
    spec.update(model=base,name='Meridian '+slug,examine='A piece of Meridian equipment.',wear_models=[[base+1,-1,-1],[base+2,-1,-1]],wear_positions=([3,5,-1] if slug=='glaive' else ([W.SLOTS[slots.split('+')[0]]] + ([6] if slug=='cuirass' else [8] if slug=='barbute' else [])+[-1,-1])[:3]),inventory_operations=[None,'Wield' if slots=='weapon' else 'Wear',None,None,'Drop'])
    if slug=='barbute':
        spec['head_models']=[]
        for g,wear in enumerate((male,female)):
            head=work/('head-'+str(g)+'.bin')
            run(py('derive_head.py',wear,g,head),log)
            spec['head_models'].append([base+3+g,-1]);models[base+3+g]=head
    write(work/'spec.json',spec)
    definition=work/'item.def'
    run(py('itemcodec.py','encode',work/'spec.json',definition),log)
    decoded=C.client_decode([(60000+idx,definition.read_bytes())])[0]
    expected=dict(C.DEFAULT,**{})|spec
    assert all(decoded[k]==v for k,v in expected.items())
    write(work/'decoded.json',decoded)
    write(work/'models.json',{k:str(p) for k,p in models.items()})
    with log.open('a') as f:f.write(f'item {60000+idx}: {len(expected)} decoded fields equal; source sha256 {hashlib.sha256(male.read_bytes()).hexdigest()}\n')
    refs=['item:'+str(i) for i in rowids]
    run([L.RESEARCH_ITEMS,L.CACHE,'icons',work/'icons.bmp',*refs[:4],f'def:{definition}+model:{inv}',*refs[4:]],log)
def motion(row):
    slug,folder,slots,pose,kind,ref,stand,walk=row
    work=OUT/slug
    log=work/'motion.txt';log.write_text('')
    for g,m in enumerate((source(row),work/'female.bin')):
        outfit=f'model:{m}@{slots}'
        if slug=='glaive':outfit+='+shield'
        for action,seq,frames in [('stand',stand,'0'),('walk',walk,'1,4'),('run',1210 if walk==1146 else 824,'1,3'),('slash',390,'1,3')]:
            args=[L.RESEARCH_POSE,L.CACHE,work/f'{g}-{action}.bmp',f'gender={g}',f'seq={seq}',f'frames={frames}','draw=actor','cell=200x250','zoom=530','none',f'item:{ref}',outfit]
            if slug in ('glaive','staff'):
                args[args.index('zoom=530')]='zoom=810'
                args.insert(-3,'aim=132')
            run(args,log)
        if slug=='barbute':
            run([L.RESEARCH_POSE,L.CACHE,work/f'chat-{g}.bmp','head=1',f'gender={g}','draw=widget','yaw=1882','pitch=40','zoom=796','cell=200x200',f'item:{ref}',f'model:{work}/head-{g}.bin@head+hair'],log)
def main():
    stage=sys.argv[1]
    if stage=='verify':return verify()
    for row in DATA:
        if len(sys.argv)>2 and row[0] not in sys.argv[2:]:continue
        print(stage,row[0],flush=True)
        {'assets':asset,'motion':motion}[stage](row)
if __name__=='__main__':main()

