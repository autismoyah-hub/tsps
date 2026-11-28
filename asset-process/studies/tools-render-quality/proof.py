"""Reproduce the scale board, controls, RGB loss measurements and source-to-page checks."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import hashlib
import json
import math
from pathlib import Path
import re
import sys
from tempfile import TemporaryDirectory

import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageChops
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import quality as Q
import scene_quality as S

H, R = Q.HERE, Q.ROOT
P = R / "sittings/phase1"
CASES = [("boss",7541,"boss-warden-general-3",4111), ("watcher",12812,"creature-stone-watcher-3",4112),
         ("hound",11068,"creature-star-hound-3",4113)]
def rgb(path):
    with Image.open(path) as im:
        return im.convert("RGB")

def stats(a, b):
    diff = np.abs(np.asarray(a, dtype=np.int16) - np.asarray(b, dtype=np.int16))
    return dict(pixels=a.width*a.height, changed=int(np.any(diff,axis=2).sum()),
                original=len(a.getcolors(a.width*a.height)), encoded=len(b.getcolors(b.width*b.height)),
                mae=float(diff.mean()), worst=int(diff.max()))

def font(n, bold=False):
    return ImageFont.truetype("segoeuib.ttf" if bold else "segoeui.ttf", n)

def cut(im, k):
    w = im.width // 2
    cell = im.crop((k*w,0,(k+1)*w,im.height))
    box = ImageChops.difference(cell, Image.new("RGB",cell.size,cell.getpixel((0,0)))).getbbox()
    return cell.crop(box)

def board(strips):
    parts = [(title,scale,[cut(strip,k) for k in (0,1)]) for title,scale,strip in strips]
    cw = max(im.width for _,_,pair in parts for im in pair)+28
    heights = [max(114,max(im.height for im in pair))+68 for _,_,pair in parts]
    width = max(860,2*cw+256)
    page = Image.new("RGB",(width,138+sum(heights)),(26,30,35))
    d = ImageDraw.Draw(page)
    d.text((20,12),"Tekton and the Warden-General",font=font(32,True),fill="white")
    d.text((21,61),"Same standing frame and yaw. Open at 100% for one image pixel per screen pixel.",font=font(17),fill="#d2d9dc")
    for x,label in [(16,"Tekton · NPC 7541"),(cw+16,"Warden-General III"),(width-230,"1:1 detail · left / right")]:
        d.text((x,102),label,font=font(18,True),fill="#a9d6d2")
    y = 138
    for (title,scale,pair),rowh in zip(parts,heights):
        d.line((15,y,width-15,y),fill="#536064")
        d.text((16,y+6),f"{title}    /    {scale:.1f} px per tile",font=font(19,True),fill="white")
        for k,im in enumerate(pair):
            page.paste(im,((cw-im.width)//2+k*cw,y+40))
            x = im.width//2
            h = round(im.height*.24)
            box=(x-48,h-48,x+48,h+48)
            detail = Image.new("RGB",(96,96),Q.BG)
            clipped=(max(0,box[0]),max(0,box[1]),min(im.width,box[2]),min(im.height,box[3]))
            detail.paste(im.crop(clipped),(clipped[0]-box[0],clipped[1]-box[1]))
            page.paste(detail,(width-228+105*k,y+43))
        y += rowh
    page.save(H/"board-quality.png")

def motion_tests(tmp):
    measures = {}
    for tag,npc,model,seed in CASES:
        entries,seq = Q.pair(npc,R/"builds"/model/(model+".json"),seed,tmp)
        low,high,_,_ = Q.L.size_after_scale(npc)
        dist=int(512*(high-low).max()*1.75/300)
        aim=int((high[1]-low[1])/2)
        args=dict(distance=dist,aim=aim,focal=512)
        old=Q.draw(entries,["stand:0"],tmp,reference=True,**args)
        gif=rgb(P/tag/"lineup.gif")
        before=Q.numbered(old)
        before.save(H/(tag+"-before-gif.png"))
        test=tmp/"test.gif"
        before.save(test,optimize=False)
        measures[tag] = dict(distance=dist,aim=aim, tile=128*512/dist,
                             gif=stats(before,gif), reconstructed_gif=stats(rgb(test),gif)["changed"])
        if tag=="boss":
            offset=Q.crop_offset(npc,tmp)
            base=Q.actor_strips(entries,["stand:0"],tmp,1,offset)[0]
            strips=[("[quoted words withheld]",128*512/dist,gif.crop((0,26,600,326))),
                    ("Faithful 1×",128*256/984,base),("Magnified 2× · nearest",128*512/984,Q.magnify(base,"magnified",2))]
            for res in (2,3,4):
                highres=Q.actor_strips(entries,["stand:0"],tmp,res,offset)[0]
                strips.append((f"High-resolution {res}×",128*256*res/984,highres))
                if res==2:
                    direct=Q.draw(entries,["stand:0"],tmp,reference=True,size=(600,1200))
                    y=(150-offset)*2
                    direct=direct.crop((0,y,1200,y+600))
                    measures["stock_npc_512_diff"]=stats(direct,highres)["changed"]
                    measures["nearest_vs_raster_2x"]=stats(strips[2][2],highres)["changed"]
            board(strips)
    return measures

def world_tests(tmp, data):
    body=["pin","npc 7541 3221 3218 0"]
    img=S.scene(body,tmp,[3222.5,3219.5],0,"faithful",1,Q.L.cmesh.CACHE)
    # Independently specify the reference lens, viewport and orbit. Rounded to a micro-unit.
    direct=S.capture(["size 512 334","focal 256","camera 3222.5 3219.5 0 909.097460 376.560497 50",*body],tmp)
    img.save(H/"faithful-scene.png")
    direct.save(H/"client-scene.png")
    data["scene"]=stats(direct,img)
    combined=rgb(R/"sittings/encounter-side-by-side.gif")
    for k,(tag,exe,cache) in enumerate([
        ("real-tekton",_AP.exe('arena_film.exe', R / 'studies/tools-real-fight'),Q.L.cmesh.CACHE),
        ("arena-arena1",S.ARENA,json.loads((R/"studies/tools-fight-arena/fight-arena.json").read_text())["cache"])]):
        folder=R/"render"/("fight-"+tag)
        lines=(folder/"encounter.log").read_text().splitlines()
        end=next(i for i,s in enumerate(lines) if s.startswith("300 frames"))
        frame=S.capture(lines[:end],tmp,exe,cache)
        # Exclude caption bands, which are composited after the renderer.
        crop=(0,24,608,432)
        raw=frame.crop(crop)
        raw.save(H/(tag+"-before-gif.png"))
        data[tag]=dict(first_generation=stats(raw,rgb(folder/"encounter.gif").crop(crop)),
                       published=stats(raw,combined.crop((620*k,58,620*k+608,466))))
def page_checks():
    rows=[]
    for letter in "abcd":
        id="2026-11-30-"+letter
        spec=json.loads((R/"sittings"/(id+".json")).read_text())
        page=Path(_AP.fix("<REVIEW_DIR>/sittings"))/id
        html=(page/"index.html").read_text(encoding="utf8")
        public=json.JSONDecoder().raw_decode(html.split("const S=")[1])[0]
        names={it["id"]:it["image"] for it in public["items"] if "image" in it}
        for item in spec["items"]:
            if "image" not in item: continue
            src=Path(item["image"])
            dest=page/names[item["id"]]
            digest=lambda p: hashlib.file_digest(p.open("rb"),"sha256").hexdigest()
            with Image.open(dest) as im:
                rows.append(dict(page=id,source=str(src),copy=dest.name,shape=im.size,
                                 identical=digest(src)==digest(dest) if src.exists() else None))
    return rows

if __name__=="__main__":
    with TemporaryDirectory(dir=H) as folder:
        tmp=Path(folder)
        data=motion_tests(tmp)
        world_tests(tmp,data)
        data["pages"]=page_checks()
        (H/"measurements.json").write_text(json.dumps(data,indent=2))
        print(json.dumps({k:v for k,v in data.items() if k!="pages"},indent=2))
