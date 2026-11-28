"""Draw a 36x32 item icon or a three-item ground row. Supply a real ID or a model and its view JSON.
For models, view keys are zoom, angles [xan,yan,zan], offset [x,y], optional scale, ambient, contrast, recolor and retexture.
Ground takes two real neighbour IDs. Output is a PNG at the renderer's native resolution."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse,pathlib,sys,json,tempfile,subprocess
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
import itemlib as L,itemcodec
from PIL import Image
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('mode',choices=['icon','ground']);p.add_argument('png',type=pathlib.Path)
    g=p.add_mutually_exclusive_group(required=True);g.add_argument('--item',type=int);g.add_argument('--model',type=pathlib.Path)
    p.add_argument('--view',type=pathlib.Path);p.add_argument('--neighbours',nargs=2,type=int)
    p.add_argument('--outline',type=int,choices=[0,1,2],default=1);p.add_argument('--shadow',default='302020')
    a=p.parse_args()
    if a.mode=='ground' and not a.neighbours:p.error('ground requires --neighbours LEFT_ID RIGHT_ID')
    if bool(a.model)!=bool(a.view):p.error('--model and --view go together')
    with tempfile.TemporaryDirectory(dir=L.HERE/'completed') as t:
        bmp=pathlib.Path(t)/'view.bmp'
        if a.model:
            v=json.loads(a.view.read_text(encoding='utf8'))
            d=pathlib.Path(t)/'item.def';d.write_bytes(itemcodec.encode(dict(v,model=1000100)))
            s=f'def:{d}+model:{a.model.resolve()}'
        else:s=f'item:{a.item}'
        if a.mode=='icon':args=[L.RESEARCH_ITEMS,L.CACHE,'icons',str(bmp),f'outline={a.outline}',f'shadow={a.shadow}','bg=ff00ff',s]
        else:args=[str(_AP.exe('meridian_ground.exe', L.EXE_DIR)),L.CACHE,str(bmp),'item:'+str(a.neighbours[0]),s,'item:'+str(a.neighbours[1])]
        print(subprocess.list2cmdline(args),flush=True);subprocess.run(args,check=True)
        with Image.open(bmp) as im:
            if a.mode=='icon':
                # Obtain transparency by comparing the same drawing over a second background.
                pixels=list(im.convert('RGB').getdata())
                args[args.index('bg=ff00ff')]='bg=00ff00'
                other=pathlib.Path(t)/'second.bmp';args[3]=str(other)
                print(subprocess.list2cmdline(args),flush=True);subprocess.run(args,check=True)
                with Image.open(other) as second:
                    rgba=Image.new('RGBA',im.size)
                    rgba.putdata([(*v,255) if v==w else (0,0,0,0) for v,w in zip(pixels,second.convert('RGB').getdata())])
                    rgba.save(a.png)
            else:im.save(a.png)
    print(a.png)
if __name__=='__main__':main()

