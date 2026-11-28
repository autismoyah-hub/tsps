"""Import an HD model into a self-contained .blend via the installed background Blender."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys,pathlib,tempfile,base64
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from hdview import *
from mapping import uvmap
def build(rev,i,out):
 c=Cache(rev);raw=c.group(7,i);m=decode(raw)
 with tempfile.TemporaryDirectory(dir=ROOT) as tmp:
    p=Path(tmp)
    from materials import metadata
    m['uv']=uvmap(m).tolist()
    m['material_records']={t:metadata(rev)[str(t)] for t in set(m['face_textures']) if t>=0}
    m.update(revision=rev,id=i,source=base64.b64encode(raw).decode(),texture_issues={},recipes={})
    for t in sorted(set(m['face_textures'])-{-1}):
        im,err=preview(c,t);im.save(p/f'{t}.png')
        if err:m['texture_issues'][t]=err
        m['recipes'][t]=base64.b64encode(c.group(9,t)).decode()
    (p/'model.json').write_text(json.dumps(m))
    subprocess.run([_AP.fix('<BLENDER>'),'-b','--factory-startup',
        '--python-exit-code','1','-P',str(ROOT/'blender_import.py'),'--',str(p),str(out.resolve())],check=True)
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('id',type=int);p.add_argument('out',type=Path)
 p.add_argument('--cache',type=int,choices=[561,601,614],default=614);a=p.parse_args();build(a.cache,a.id,a.out)

