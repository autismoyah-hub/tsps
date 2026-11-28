"""Reproduce the takeover in isolated output directories. Run from this folder with python -I."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json
import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageChops

here = Path(__file__).resolve().parent
sys.path.insert(0, str(here))
import animlib as A

work = here / "verification"
render = here.parents[1] / "render/research-animation-verified"
work.mkdir(exist_ok=True)
render.mkdir(exist_ok=True)
exe = _AP.bindir()
py = [sys.executable, "-I", "-B"]
sword = _AP.fix("model:<ROOT>/builds/meridian-sword/meridian-sword.bin@weapon")
glaive = sword.replace("sword", "glaive")
clips = [("meridian_crosscut", 390, sword), ("meridian_fall", 440, glaive),
         ("warden_salute", 2112, "none"), ("cow_flyshake", 5854, "npc:2790")]

def run(*args):
    args = list(map(str, args))
    print("$", subprocess.list2cmdline(args), flush=True)
    result = subprocess.run(args, cwd=here, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout.decode(errors="replace"), end="", flush=True)
    result.check_returncode()

def script(name, *args):
    run(*py, here / (name + ".py"), *args)

def census():
    script("roundtrip_all")
    script("probe1")
    keyed = ["sequence\tarchive22_group\tfile\tstart\tend"]
    for sid, raw in sorted(A.sequences().items()):
        seq = A.decode_sequence(raw)
        asset = seq["cached_animation"]
        if asset is not None and asset < 2**31:
            start, end = seq["cached_range"] or (0, 0)
            keyed.append(f"{sid}\t{asset>>16}\t{asset&65535}\t{start}\t{end}")
    (work / "skeletal-sequences.tsv").write_text("\n".join(keyed) + "\n")
    print("Runtime-eligible opcode 13 IDs:", len(keyed)-1)

def measurements():
    run(*py, here.parents[1] / "tools/tsps_anims.py", "--name", "halberd")
    script("measure", "seq", 390, 386, 440, 2080, 2081, 2082, 2112, 858)
    (work / "mesh").mkdir(exist_ok=True)
    for name in ["creature", "animlib", "measure", "author", "rigs"]:
        shutil.copyfile(here / (name+".py"), work / (name+".py"))
    for npc in [3029, 2790]:
        run(*py, work / "creature.py", npc)

def pixels():
    entries = ["none", "item:1323", sword, "item:1127+item:1079+item:1163"]
    for sid in [819, 390]:
        folder = work / str(sid)
        script("seqtool", "export", sid, folder)
        seq = A.sequence(sid)
        count = len(seq["frames"])
        frames = "frames=" + ",".join(map(str, range(count)))
        def draw(program, source, tag):
            file = render / f"{sid}-{tag}.bmp"
            run(exe / (program+".exe"), A.CACHE, file, source, frames, *entries)
            im = Image.open(file).convert("RGB")
            file.unlink()
            im.save(file.with_suffix(".png"))
            return im
        reference = draw("asset_pose", f"seq={sid}", "cache")
        def compare(tag):
            other = draw("asset_anim", f"anim={folder}", tag)
            assert reference.size == other.size == (1200, 400*count)
            diff = ImageChops.difference(reference, other)
            counts = []
            for i in range(count):
                row = []
                for j in range(4):
                    box = (300*j, 400*i, 300*j+300, 400*i+400)
                    cell = reference.crop(box)
                    assert sum(p != cell.getpixel((0,0)) for p in cell.getdata()) > 2000
                    row.append(sum(p != (0,0,0) for p in diff.crop(box).getdata()))
                counts.append(row)
            print(sid, tag, "differing pixels by frame x equipment:", counts, flush=True)
            return sum(map(sum, counts))
        assert compare("encoded") == 0
        if sid == 390:
            fid = seq["frames"][1]
            path = folder / f"frame_{fid>>16}_{fid&65535}.dat"
            original = path.read_bytes()
            base = A.frame_base(0)
            frame = A.decode_frame(original, base)
            frame["groups"][30][0] += 1
            try:
                path.write_bytes(A.encode_frame(dict(base=0, groups=frame["groups"]), base))
                assert compare("control") > 0
            finally:
                path.write_bytes(original)

def authoring():
    for name, ref, entry in clips:
        spec = json.loads(next((here/"anims").glob(name+"*.json")).read_text())
        # End keys at the exclusive duration were never sampled by the inherited files.
        spec["keys"][-1] = {"frame": len(spec["lengths"])-1, "ease":"inout", "pose":"rest"}
        src = work / (name+".json")
        src.write_text(json.dumps(spec, indent=2))
        script("author", src, work/name)
        for size, zoom, cell in [("game", 2000, "164x166"), ("close", 520 if name!="meridian_fall" else 650, "480x530")]:
            rows = [f"anim={work/name};{entry};{name}", f"seq={ref};{entry};reference {ref}"]
            if name=="meridian_crosscut":
                rows.append(f"seq=386;{entry};reference 386")
            script("apose", render / f"{name}-{size}.png", *rows, "--zoom", zoom, "--cell", cell,
                   "--draw", "auto" if entry.startswith("npc") else "actor")

def pack():
    output = work / "new-cache"
    assert not output.exists(), "choose a fresh output before re-running packcache"
    jobs = [f"verification/{name}:{i}" for i,(name,_,_) in enumerate(clips,20001)]
    script("packcache", output, *jobs)

def packed_pixels():
    packed = work / "new-cache"
    for i,(name,ref,entry) in enumerate(clips,20001):
        for cache, source, program, tag in [(packed,f"seq={i}","asset_anim" if entry.startswith("npc") else "asset_pose","packed"),
                                           (A.CACHE,f"anim={work/name}","asset_anim","loose")]:
            seq = A.decode_sequence((work/name/"seq.dat").read_bytes())
            frames = "frames="+",".join(map(str,range(len(seq["frames"]))))
            path = render / f"{i}-{tag}.bmp"
            run(exe / (program+".exe"), cache, path, source, frames, "draw=auto" if entry.startswith("npc") else "draw=actor",entry)
            im = Image.open(path).convert("RGB")
            path.unlink()
            if tag=="packed":
                reference = im
            else:
                assert reference.size == im.size
                count = sum(p!=(0,0,0) for p in ImageChops.difference(reference,im).getdata())
                print(i, "packed/loose:", count, "differing pixels of", im.width*im.height, flush=True)
                assert count==0

if __name__=="__main__":
    mode = sys.argv[1]
    with (work / (mode+".log")).open("w", encoding="utf-8") as log:
        from contextlib import redirect_stdout
        with redirect_stdout(log):
            print("cwd", here, "\n$", subprocess.list2cmdline([*py,__file__,mode]), flush=True)
            {"pixels":pixels,"census":census,"measurements":measurements,"authoring":authoring,
             "pack":pack,"packed_pixels":packed_pixels}[mode]()
