"""Read back the finished definitions, sprites, ground rows and boards. Keep exact test commands and counts."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import pathlib,sys,tempfile,re
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from finish_items import *
from item_boards import crop
from PIL import ImageChops
def same(a,b):
    a=np.array(a.convert('RGB'));b=np.array(b.convert('RGB'))
    assert a.shape==b.shape
    return int(np.any(a!=b,axis=2).sum())
def main():
    log=OUT/'delivery.txt';log.write_text(subprocess.list2cmdline([sys.executable,'-I',__file__])+'\n')
    rows=[]
    for row in DATA:
        slug=row[0];work=OUT/slug;id=60000+DATA.index(row)
        definition=work/'item.def'
        d=C.client_decode([(id,definition.read_bytes())],plain=True)[0]
        spec=json.loads((work/'spec.json').read_text())
        assert d==json.loads((work/'decoded.json').read_text())
        assert all(d[k]==v for k,v in (C.DEFAULT|spec).items())
        view=json.loads((work/'inventory.view.json').read_text())
        assert all(view[k]==d[k] for k in ('zoom','angles','offset','ambient','contrast'))
        manifest=json.loads((work/'models.json').read_text())
        ids={d['model'],*(i for a in d['wear_models']+d['head_models'] for i in a if i>=0)}
        assert ids==set(map(int,manifest))
        for file in manifest.values():L.load_bin(file)
        board=Image.open(RENDER/(slug+'.png'));assert board.size==(822,722)
        strip=Image.open(work/'icons.bmp')
        checks=[same(board.crop((87,90,735,154)),strip.resize((648,64),Image.Resampling.NEAREST)),
            same(board.crop((28,215,794,425)),Image.open(work/'ground.bmp'))]
        for i,(g,act,j) in enumerate([(0,'stand',0),(0,'walk',1),(1,'stand',0),(1,'walk',1)]):
            x=11+200*i;checks.append(same(board.crop((x,465,x+200,715)),crop(work,g,act,2,j)))
        assert checks==[0]*6
        with log.open('a') as fp:print(f'{slug}: ID {id}, {len(ids)} resolved models, {len(C.DEFAULT)} fields, strip/ground/four pose region pixel differences {checks}',file=fp)
        g=re.search(r'^def:.*?: (\d+) changed ground pixels', (work/'ground.txt').read_text(),re.M)
        geom=json.loads((work/'geometry-motion.json').read_text());motion=geom['motion']
        assert not geom['new_collapsed_faces'] and not geom['over_90deg_normal_change_faces']
        counts=[motion[k]['ours_minus_bare'] for k in ('0-stand-0','0-walk-1','1-stand-0','1-walk-1')]
        rows.append([slug,id,view['final_pixels'],view['final_box'],int(g[1]),counts,definition.stat().st_size])
    write(OUT/'counts.json',rows)
    with tempfile.TemporaryDirectory(dir=OUT) as tmp:
        p=pathlib.Path(tmp)
        work=OUT/'longsword';view=json.loads((work/'inventory.view.json').read_text())
        for name,spec in [('real',['--item',1291]),('ours',['--model',work/'inventory.bin','--view',work/'inventory.view.json'])]:
            run(py('item_view.py','icon',p/(name+'.png'),*spec),log)
            im=Image.open(p/(name+'.png')).convert('RGBA')
            composed=Image.new('RGB',im.size,(62,53,41));composed.paste(im,mask=im.getchannel('A'))
            col=0 if name=='real' else 4
            assert same(composed,Image.open(work/'icons.bmp').crop((col*36,0,col*36+36,32)))==0
            with log.open('a') as fp:print(name,'native transparent PNG composited onto board background: 0/1152 different;',np.count_nonzero(np.array(im)[:,:,3]),'opaque',file=fp)
        run(py('item_view.py','ground',p/'ground.png','--model',work/'inventory.bin','--view',work/'inventory.view.json','--neighbours',1293,1305),log)
        assert same(Image.open(p/'ground.png'),Image.open(work/'ground.bmp'))==0
        # Legacy and research male figure draws with identical inputs, before annotations.
        for name,exe in [('male',_AP.exe('asset_pose.exe', L.EXE_DIR)),('research',L.RESEARCH_POSE)]:
            run([exe,L.CACHE,p/(name+'.bmp'),'seq=819','frames=bind,1,4','draw=actor','cell=200x250','none','item:1127','item:4131'],log)
        delta=same(Image.open(p/'male.bmp'),Image.open(p/'research.bmp'))
        assert delta==0
        with log.open('a') as fp:print('ground PNG 0/160860 different; male default compatibility',delta,'of 450000 different',file=fp)
    run([*L.PY,ROOT/'tools/tsps_anims.py',1303,3204,1381],log)
    print(log)
if __name__=='__main__':main()

