import sys,subprocess,tempfile,json
from pathlib import Path
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here))
from proof import *
with tempfile.TemporaryDirectory(dir=here) as tmp,(here/'material-edit.log').open('w') as log:
    tmp=Path(tmp);i=13228
    j=scene.object_json(i,0);mid,=j['shape_model_ids'];custom=120000
    definition=scene.object_def(i)
    def record(replace):
        return bytes([6,len(j['model_ids'])])+b''.join((custom if replace and s==0 else n).to_bytes(4,'big')+bytes([s]) for n,s in zip(j['model_ids'],j['model_shapes']))
    assert definition.count(record(False))==1
    def capture(label,b,d):
        content=scene.build(tmp/label,None,[(-16298,51,50,0)],[(custom,b,d,50,50,1,0)])
        return images(scene.capture(content,'front',[50.5,50.8,0,360,220,95]))
    cache=capture('cache',o.unpack(o.group(7,mid)),definition)
    run([sys.executable,'-I','-B',TOOLS/'real_dump.py','object',i,tmp/'d.json','--shape',0],log)
    run([BLENDER,'-b','-t','2','--python-exit-code','1','-P',TOOLS/'real_to_blender.py','--',tmp/'d.json',tmp/'o.blend'],log)
    for mode in ('roundtrip','alpha','texture'):
        dst=tmp/(mode+'.bin')
        script=TOOLS/'blender_export.py' if mode=='roundtrip' else here/'paint_check.py'
        run([BLENDER,'-b',tmp/'o.blend','-t','2','--python-exit-code','1','-P',script,'--',dst,*([mode] if mode!='roundtrip' else [])],log)
        frame=capture(mode,dst.read_bytes(),definition.replace(record(False),record(True)))
        delta=diff(cache,frame)
        print(mode,delta)
        assert (delta==0)==(mode=='roundtrip')
