"""Full-frame comparisons at identical world coordinates, against an empty scene as visibility control."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
from pathlib import Path
import json,sys,subprocess,tempfile
from PIL import Image
import numpy as np
HERE=Path(__file__).resolve().parent
TOOLS=HERE.parents[1]/'tools'
sys.path.insert(0,str(TOOLS))
import scene,osrscache as o
BLENDER=_AP.fix('<BLENDER>')
TESTS={'object':[13228,13229,1734,10325,11721,11388,12045,13219,10043,10172,10335,10060],
       'worn':[6570,25712,22388]}
def run(cmd,log):
    res=subprocess.run(list(map(str,cmd)),capture_output=True,text=True,errors='replace')
    log.write(res.stdout+res.stderr);log.flush()
    if res.returncode:raise ValueError(res.stdout[-1500:]+res.stderr[-1000:])
def images(p):
    return np.asarray(Image.open(p).convert('RGB'))
def diff(a,b):
    return int(np.any(a!=b,axis=2).sum())
def test(kind,i):
    folder=HERE/'proof'/f'{kind}-{i}'
    folder.mkdir(exist_ok=True,parents=True)
    dump=folder/'mesh.json';blend=folder/'mesh.blend';binary=folder/'mesh.bin'
    log=(folder/'run.log').open('w')
    if kind=='object':
        j=scene.object_json(i,0)
        mid,=j['shape_model_ids']
        # The real definition stays on both placements and applies its transforms once.
        source=('model',mid)
    else:
        j=json.loads(next((scene.ARCH/'items').glob(f'*/{i}.json')).read_bytes())
        source=(kind,i)
    run([sys.executable,'-I','-B',TOOLS/'real_dump.py',*source,dump,'--faithful'],log)
    run([BLENDER,'-b','-t','2','--python-exit-code','1','-P',TOOLS/'real_to_blender.py','--',dump,blend],log)
    run([BLENDER,'-b',blend,'-t','2','--python-exit-code','1','-P',TOOLS/'blender_export.py','--',binary],log)
    m=o.decode_model(binary.read_bytes());d=json.loads(dump.read_bytes())
    for key in ('faces','face_textures','alpha','texture_coords','texture_triangles','render_types','priorities'):
        assert json.loads(json.dumps(m[key]))==d[key],key
    counts=[];visible=[];thumbs=[]
    with tempfile.TemporaryDirectory(dir=folder) as tmp:
        tmp=Path(tmp)
        if kind=='object':
            captures=[]
            for label,data in [('empty',None),('cache',o.unpack(o.group(7,mid))),('export',binary.read_bytes())]:
                cell = 5 if data is None else 50
                custom=120000
                definition=scene.object_def(i)
                if label=='export':
                    ids=j['model_ids']; shapes=j['model_shapes']
                    def record(replace):
                        return bytes([6,len(ids)])+b''.join((custom if replace and s==0 else n).to_bytes(4,'big')+bytes([s]) for n,s in zip(ids,shapes))
                    assert definition.count(record(False))==1,'model opcode not found'
                    definition=definition.replace(record(False),record(True))
                entries=[(custom,data or o.unpack(o.group(7,mid)),definition,cell,cell,1,0)]
                content=scene.build(tmp/label,None,[(-16298,51,50,0)],entries)
                captures.append([images(scene.capture(content,str(yaw),[50.5,50.8,yaw,360,220,95])) for yaw in (0,2048,14336)])
            for empty,a,b in zip(*captures):
                counts.append(diff(a,b));visible.append(diff(a,empty))
                thumbs.append(Image.fromarray(b).resize((320,225)))
        else:
            path=tmp/'figure.bmp'
            slots=[s for s in j['wear_positions'] if s>=0]
            spec='model:'+binary.as_posix()+'@'+'+'.join(map(str,slots))
            run([_AP.bindir() / 'asset_figure.exe',o.CACHE,path,'none','item:'+str(i),spec,'draw=actor'],log)
            for row in np.vsplit(images(path),3):
                empty,a,b=np.hsplit(row,3)
                counts.append(diff(a,b));visible.append(diff(a,empty))
                thumbs.append(Image.fromarray(b))
    w,h=thumbs[0].size;im=Image.new('RGB',(3*w,h))
    for k,t in enumerate(thumbs):im.paste(t,(w*k,0))
    im.save(folder/'views.png')
    return dict(kind=kind,id=i,name=j['name'],textures=sorted(set(m['face_textures'])-{-1}),alpha=sorted(set(m['alpha'])-{0}),different=counts,visible=visible)
if __name__=='__main__':
    file=HERE/'proof.json'
    rows=json.loads(file.read_bytes()) if file.exists() else []
    for kind,ids in TESTS.items():
        for i in ids:
            if '--fresh' not in sys.argv and any(r.get('id')==i and 'different' in r for r in rows):continue
            rows=[r for r in rows if r.get('id')!=i]
            try:r=test(kind,i)
            except (Exception,SystemExit) as e:r=dict(kind=kind,id=i,error=repr(e))
            rows.append(r);file.write_text(json.dumps(rows,indent=1));print(r,flush=True)
