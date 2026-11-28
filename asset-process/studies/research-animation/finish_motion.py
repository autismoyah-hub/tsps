"""Refine the overhead pose and record checks on the inherited and revised material."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json
import sys
from pathlib import Path
from contextlib import redirect_stdout
sys.path.insert(0, str(Path(__file__).resolve().parent))
from verify_takeover import A, clips, exe, glaive, here, py, render, run, script, work
import measure

def refine():
    spec = json.loads((work/"meridian_fall.json").read_text())
    for key in spec["keys"]:
        pose = key["pose"]
        if isinstance(pose,dict) and "r_shoulder" in pose:
            pose["r_shoulder"][1] -= 25
    spec["grip"]["along"] = [-45,-18]
    spec["grip"]["blend_in"] = 2
    spec["grip"]["blend_out"] = 2
    src = work/"meridian_fall-grip.json"
    src.write_text(json.dumps(spec,indent=2))
    script("author",src,work/"meridian_fall-grip")

def audit():
    old = A.Cache(here/"cache-packed")
    for sid,(name,ref,entry) in enumerate(clips,20001):
        folder = here/"out"/name
        seq = A.decode_sequence((folder/"seq.dat").read_bytes())
        assert old.file(2,12,sid)==(folder/"seq.dat").read_bytes()
        for fid in seq["frames"]:
            assert old.file(0,fid>>16,fid&65535)==(folder/f"frame_{fid>>16}_{fid&65535}.dat").read_bytes()
        print("Inherited packed exact loose definition and all frame references:",sid,len(seq["frames"]))
    allseq = A.sequences()
    for tag,cache in [("inherited",old),("rebuilt",A.Cache(work/"new-cache"))]:
        data = cache.files(2,12)
        print(tag,"stock definitions changed:",sum(raw!=data.get(k) for k,raw in allseq.items()),"of",len(allseq))
    s = Path(_AP.fix(r"<TSPS_CHECKOUT>/server/data/definitions"))
    style=json.loads((s/"item-combat-styles.json").read_text())
    for name in style["weaponInterfaces"]["HALBERD"]["fightTypes"]:
        print(name,style["fightTypes"][name])
    table = ["sequence\tgroup\tjoint\tfrom_frame\tto_frame\thold_cycles\tdegrees"]
    for sid in [390,386,440,2080,2081,2082,2112,858]:
        seq, rows = measure.joint_table(sid,quiet=True)
        for row in rows:
            if row["kind"] != "rotate" or not row["moving"]: continue
            vals = row["values"]
            for i,(a,b) in enumerate(zip(vals,vals[1:])):
                table.append(f'{sid}\t{row["group"]}\t{row["name"]}\t{i}\t{i+1}\t{seq["lengths"][i]}\t{measure.angle_between(a,b):.2f}')
    (work/"joint-steps.tsv").write_text("\n".join(table)+"\n")
    print("Per-joint rotation steps written:",len(table)-1)
    spec = {"name":"Goblin toe tap","base":1415,"like":6181,"rest":{"seq":6181,"frame":0},
            "frame_group":20005,"id":20005,"lengths":[4,3,5,3,4],
            "keys":[{"frame":0,"pose":"rest"},{"frame":2,"pose":{"g9":[-18,0,0]}},{"frame":4,"pose":"rest"}]}
    path=work/"goblin.json"
    path.write_text(json.dumps(spec,indent=2))
    script("author",path,work/"goblin")
    script("apose",render/"goblin-check.png",f"anim={work/'goblin'};npc:3029;Goblin toe tap","--draw","auto")

def final_checks():
    from PIL import Image, ImageChops
    for size, zoom, cell in [("game",2000,"164x166"),("close",650,"480x530")]:
        script("apose",render/f"meridian_fall-grip-{size}.png",f"anim={work/'meridian_fall-grip'};{glaive};Meridian overhead",
               f"seq=440;{glaive};Halberd sweep 440","--zoom",zoom,"--cell",cell)
    packed=work/"final-cache"
    final=[(20001,"meridian_crosscut",clips[0][2]),(20002,"meridian_fall-grip",glaive),
           (20003,"warden_salute","none"),(20004,"cow_flyshake","npc:2790"),(20005,"goblin","npc:3029")]
    if not packed.exists():
        script("packcache",packed,*[f"verification/{name}:{i}" for i,name,_ in final])
    def draw(program,cache,source,frames,entry):
        path=render/"verification-check.bmp"
        run(exe/(program+".exe"),cache,path,source,frames,entry,"draw=auto" if entry.startswith("npc:") else "draw=actor")
        im=Image.open(path).convert("RGB")
        path.unlink()
        return im
    def diff(a,b):
        assert a.size==b.size
        hist=ImageChops.difference(a,b).convert("RGB")
        return sum(p!=(0,0,0) for p in hist.getdata())
    for i,name,entry in final:
        raw=(work/name/"seq.dat").read_bytes()
        assert A.Cache(packed).file(2,12,i)==raw
        seq=A.decode_sequence(raw)
        frames="frames="+",".join(map(str,range(len(seq["frames"]))))
        a=draw("asset_anim" if entry.startswith("npc:") else "asset_pose",packed,f"seq={i}",frames,entry)
        b=draw("asset_anim",A.CACHE,f"anim={work/name}",frames,entry)
        n=diff(a,b)
        print("FINAL packed/loose",i,n,"differing of",a.width*a.height)
        assert n==0
        start=b.crop((0,0,b.width,400))
        end=b.crop((0,b.height-400,b.width,b.height))
        n=diff(start,end)
        print("FINAL first/last",i,n)
        assert n==0
        if i in range(20001,20005):
            previous=here/"out"/clips[i-20001][0]
            im=draw("asset_anim",A.CACHE,f"anim={previous}",f"frames=0,{len(seq['frames'])-1}",entry)
            print("INHERITED first/last",i,diff(im.crop((0,0,300,400)),im.crop((0,400,300,800))))
    for i in [819,390]:
        frames="frames="+",".join(map(str,range(len(A.sequence(i)["frames"]))))
        a,b=[draw("asset_pose",cache,f"seq={i}",frames,"item:1323") for cache in (packed,A.CACHE)]
        n=diff(a,b)
        print("FINAL stock cache comparison",i,n,"of",a.width*a.height)
        assert n==0
    path=render/"skeletal-example.bmp"
    run(_AP.exe('asset_npc.exe', exe),A.CACHE,"draw",path,"rows=bind,stand:0,stand:10","cell=300x400","npc:15740")
    im=Image.open(path).convert("RGB")
    path.unlink()
    im.save(render/"skeletal-example.png")
    n=diff(im.crop((0,400,300,800)),im.crop((0,800,300,1200)))
    print("Cached stand samples 0 vs 10:",n,"differing pixels")
    assert n>0

def review():
    import hashlib
    from PIL import Image, ImageChops
    files = [p for p in here.glob("*.py") if p.name not in {"verify_takeover.py","finish_motion.py"}]
    for folder in ["anims","rt","out","mesh","cache-packed","__pycache__"]:
        files.extend(p for p in (here/folder).rglob("*") if p.is_file())
    manifest = ["sha256\tbytes\tfile"]
    for p in sorted(files):
        with p.open("rb") as f:
            digest = hashlib.file_digest(f,"sha256").hexdigest()
        manifest.append(f"{digest}\t{p.stat().st_size}\t{p.relative_to(here)}")
    (work/"inherited-inventory.tsv").write_text("\n".join(manifest)+"\n")
    print("Inherited files inventoried:",len(files))
    print("Player base groups",len(A.frame_base(0)["types"]))
    for stem,ref,entry in [("meridian_crosscut",390,clips[0][2]),("meridian_fall-grip",440,glaive),("warden_salute",2112,"none")]:
        path = render/(stem+"-track.bmp")
        seq=A.decode_sequence((work/stem/"seq.dat").read_bytes())
        frames="frames="+",".join(map(str,range(len(seq["frames"]))))
        run(_AP.exe('asset_anim.exe', exe),A.CACHE,path,f"anim={work/stem}",frames,"track=45,46",entry)
        path.unlink()
        im=Image.open(render/(stem+"-game.png")).convert("RGB")
        crop=im.crop((0,22,164,22+144))
        print(stem,"first game cell foreground box excluding caption",ImageChops.difference(crop,Image.new("RGB",crop.size,crop.getpixel((0,0)))).getbbox())
    final=A.Cache(work/"final-cache")
    old=A.sequences()
    packed=final.files(2,12)
    print("Final cache:",len(packed),"definitions;",sum(v!=packed.get(k) for k,v in old.items()),"stock definitions differ")

if __name__=="__main__":
    mode=sys.argv[1]
    with (work/(mode+".log")).open("w",encoding="utf-8") as log, redirect_stdout(log):
        print("cwd",here,"command",*py,__file__,mode,flush=True)
        {"audit":audit,"refine":refine,"final_checks":final_checks,"review":review}[mode]()
