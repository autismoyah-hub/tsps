"""Check the command-line entry points and independent legacy controls."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import numpy as np
from PIL import Image
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
import quality as Q
import scene_quality as S

def run(script,*args):
    subprocess.run([sys.executable,"-I",Q.HERE/script,*map(str,args)],check=True,capture_output=True)
def array(path):
    with Image.open(path) as im:
        return np.array(im.convert("RGB"))
def diff(a,b):
    return int((a!=b).any(axis=2).sum())

def main(work):
    root=Q.ROOT
    result={}
    model=root/"builds/boss-warden-general-3/boss-warden-general-3.json"
    for mode in ("faithful","magnified","hires"):
        run("quality.py",work/(mode+".png"),7541,model,"--seed",4111,"--preset",mode)
    base=array(work/"faithful.png")
    result["magnified_cli_differing_pixels"]=diff(base.repeat(2,0).repeat(2,1),array(work/"magnified.png"))
    with Image.open(root/"sittings/phase1-hq/boss/lineup.webp") as motion:
        result["motion_vs_still_differing_pixels"]=diff(base,np.array(motion.convert("RGB")))
    legacy=root/"night/armoury2/boards/cal-return-blind"
    args=[legacy/"content.txt",Q.L.cmesh.CACHE,"20.5","60.95703","0","330","250","142","classic","auto"]
    for exe,name in ((Path(Q.L.cmesh.EXE).with_name("authored_scene_snapshot.exe"),"stock"),
                     (_AP.exe('quality_scene.exe', Q.HERE),"adapter")):
        subprocess.run([exe,*args,work/name],check=True,capture_output=True,
                       env={**os.environ,"REVIEW_W":"1280","REVIEW_H":"900","REVIEW_FOCAL":"1000"})
    checks=[(work/x/"view.bmp").read_bytes() for x in ("stock","adapter")]
    result["stock_authored_bmp_equal"]=checks[0]==checks[1]
    result["stock_authored_bmp_sha256"]=hashlib.sha256(checks[0]).hexdigest()
    result["historical_shield_bmp_equal"]=checks[0]==(legacy/"e0v0/view.bmp").read_bytes()
    for factor in (1,2,3,4):
        mode="faithful" if factor==1 else "hires"
        output=work/f"shield-{factor}.png"
        run("scene_quality.py",legacy/"content.txt",output,"--authored","--tile",20.5,60.95703,"--preset",mode,"--factor",factor)
        result[f"authored_{factor}x_size"]=Image.open(output).size
        assert result[f"authored_{factor}x_size"]==(512*factor,334*factor)
    # The film API accepts any final RGB panels; check a multi-colour 3-frame loop.
    movie=[Image.open(Q.HERE/"real-tekton-before-gif.png").convert("RGB").crop((i,0,512+i,334)) for i in (0,8,16)]
    result["film_webp"]=Q.save_motion(work/"film.webp",movie,[100,100,100])
    assert result["stock_authored_bmp_equal"] and not result["magnified_cli_differing_pixels"] and not result["motion_vs_still_differing_pixels"]
    (Q.HERE/"preset-checks.json").write_text(json.dumps(result,indent=2))
    print(result)
if __name__=="__main__":
    with TemporaryDirectory(dir=Q.HERE) as tmp:
        main(Path(tmp))
