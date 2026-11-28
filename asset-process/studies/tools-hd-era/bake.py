"""Bake the original procedural graphs with the local TSPS client, independently of 239 texture IDs."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys,pathlib,json,base64,tempfile,zipfile,subprocess
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from hdcache import *
from PIL import Image
import io
NODE=Path(_AP.fix('<NODE>'))
def bake(rev):
 c=Cache(rev)
 def bank(a):return {i:base64.b64encode(c.group(a,i)).decode() for i in c.index(a)}
 with tempfile.TemporaryDirectory(dir=ROOT) as tmp:
    p=Path(tmp)
    (p/'inputs.json').write_text(json.dumps(dict(textures=bank(9),materials=bank(26),sprites=bank(8))))
    subprocess.run([NODE,ROOT/'client_bridge.cjs',p,str(rev)],check=True)
    errors=json.loads((p/'errors.json').read_text())
    (ROOT/f'texture-errors-{rev}.json').write_text(json.dumps(errors,indent=1))
    with zipfile.ZipFile(ROOT/f'textures-{rev}.zip','w') as pack:
        for f in sorted(p.glob('*.rgba')):
            data=io.BytesIO();Image.frombytes('RGBA',(128,128),f.read_bytes()).save(data,format='PNG')
            pack.writestr(f.stem+'.png',data.getvalue())
        pack.write(p/'meta.json','materials.json',zipfile.ZIP_DEFLATED)
    print(rev,len(c.index(9))-len(errors),'baked',len(errors),'errors',list(errors.items())[:4],flush=True)
if __name__=='__main__':
 for rev in [561,601,614]:bake(rev)

