"""Film the revision-239 Tekton encounter. Run with no arguments for the complete set."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "research-effects2/fight"))
import fight
from PIL import Image, ImageDraw

F = fight.load(HERE / "fight-tekton.json")
S = F["stage"]
OUT = Path(F["output"])
A = F["attacks"]
FX = fight.fx

def asset(key):
    return F["assets"][key]["id"]

def scene(name):
    encounter = name in ("encounter", "probe")
    timeline = F["timeline"] if encounter else [(0, name)]
    cycles = 1500 if encounter else (375 if name == "repair_sparks" else 90)
    px, py = S["place"][0] - 1, S["place"][1]
    nx, ny = px - 1, py + 1
    form = S["npc"] if encounter else A[name]["form"]
    angle = 0
    player_angle = 1024
    labels = []
    poses, reactions, body = [], [], []
    if encounter:
        for tick, action, *args in F["staging"]:
            c = tick * 30
            if action == "step":
                body.append(f"step {c} " + " ".join(map(str, args)))
                continue
            body.append(f"form {c} {asset(args[0])}")
            if action == "stance":
                poses.append(f"seq={asset(args[1])}@{c+args[2]}")
            title = {"fighting":"Combat stance", "walking":"Return to anvil",
                     "anvil":"Anvil repair" if tick == 31 else "Rise from anvil",
                     "enraged_walk":"Enraged approach", "enraged":"Enraged stance"}[args[0]]
            labels.append((c, title, ""))
    elif name == "repair_sparks":
        nx, ny = S["home"]
        angle = 1024
        poses.append(f"seq={asset('pose.anvil_pose')}@15")
        labels.append((0, "Anvil repair", ""))
        timeline = [(1, name)]
        labels.append((333, "Anvil repair", "sparks fade"))
    elif name == "stomp":
        # A player west of the original footprint is under the first return
        # step to the anvil, then clear of the second one.
        px, py = nx-1, ny+1
        player_angle = 1536
        body += [f"step 30 {nx-1} {ny+1} 768", f"step 60 {nx-1} {ny+2} 1024"]
        labels.append((60, "Walking stomp", "clear of footprint"))
    for tick, key in timeline:
        attack = A[key]
        c = tick * 30
        if key == "repair_sparks":
            labels.append((c, attack["label"], "two sparks"))
            # fight.script supplies the stock by-ID commands. Tekton locks both ends to tiles.
            lines, _, _ = fight.script(F, [key], origin=S["place"], start=c)
            shots = iter(b for b in attack["beats"] if b["do"] == "projectile")
            for line in lines[2:]:
                if line.startswith("proj "):
                    b = next(shots)
                    dx, dy = b["targetOffset"]
                    words = line.split()
                    words[2] = ",".join(map(str, b["sourceTile"]))
                    words[3] = f"{px+dx},{py+dy}"
                    line = " ".join(words)
                body.append(line)
            landing = min(fight.at(b) for b in attack["beats"] if b["do"] == "tile_graphic")
            labels.append((c + landing, attack["label"], "landing"))
        else:
            lines, _, _ = fight.script(F, [key], origin=S["place"], start=c)
            poses.extend(w for w in lines[0].split() if w.startswith("seq="))
            reactions.extend(w for w in lines[1].split() if w.startswith("seq="))
            hit = next(b for b in attack["beats"] if b["do"] == "hit")
            labels.extend([(c,attack["label"],"step" if key == "stomp" else "swing"),
                           (c+fight.at(hit),attack["label"],"underfoot" if key == "stomp" else "contact")])
    body.append(f"loc {asset('fire')} 3279 5281 10 0 0 - sync")
    if encounter:
        body += [f"loc {asset('block')} {nx} {ny} 10 0 0 810 sync",
                 f"loc {asset('block')} {nx} {ny} 10 0 1410 - sync"]
    head = ["size " + " ".join(map(str,S["size"])),f"plane {S['plane']}",f"focal {S['focal']}",
            "camera " + " ".join(map(str,S["camera"])),"pin",
            " ".join([f"npc {asset(form)} {nx} {ny} {angle}", *poses]),
            " ".join([f"player {px} {py} 0 {player_angle} {FX.PLAYER_ITEMS}", *reactions])]
    return head+body, cycles, sorted(labels, key=lambda e:e[0])

def film(name):
    script, cycles, labels = scene(name)
    times = [0,100,160,220,850,930,1000,1040,1080,1160,1250,1495] if name=="probe" else list(range(0,cycles,5))
    with tempfile.TemporaryDirectory(dir=HERE) as tmp:
        log = FX.run(script+[f"render {','.join(map(str,times))} {Path(tmp).as_posix()}/f"], FX.SRC)
        frames = FX.frames(Path(tmp).as_posix()+"/f", times)
    assert "animated map objects started" in log
    if name in ("probe","encounter","repair_sparks"):
        assert "proj 660 at" in log and "spot 659 frame" in log
    for im,c in zip(frames,times):
        _, title, detail = next(e for e in reversed(labels) if e[0] <= c)
        draw = ImageDraw.Draw(im)
        draw.rectangle((0,0,608,23),fill=(18,24,36))
        draw.text((9,4),f"{c/50:04.1f}s    {title}    {detail}",fill=(242,226,183),font=FX.font(14))
    sheet = Image.new("RGB",(1216,648))
    for i in range(12):
        sheet.paste(frames[round((len(frames)-1)*i/11)].resize((304,216)),(304*(i%4),216*(i//4)))
    if name=="probe":
        sheet.save(HERE/"probe.png")
        return
    sheet.save(OUT/(name+".png"))
    dest = OUT/(name+".gif")
    frames[0].save(dest,save_all=True,append_images=frames[1:],duration=100,loop=0)
    with Image.open(dest) as check:
        assert check.size==(608,432) and check.n_frames==len(times)
        for i in range(check.n_frames):
            check.seek(i)
            assert check.info["duration"]==100
    (OUT/(name+".log")).write_text("\n".join(script)+f"\n{len(times)} frames at 100 ms.\n"+log,encoding="utf-8")
    print(f"{name}  {len(times)} frames  {cycles/50:g} s",flush=True)

if __name__=="__main__":
    FX.EXE_FX = _AP.exe('arena_film.exe', HERE)
    if not FX.EXE_FX.exists():
        sys.path.insert(0,str(HERE))
        from build_view import build
        build()
    assert Path(F["cache"]).resolve()==Path(FX.SRC).resolve()
    assert not fight.check(F), fight.check(F)
    OUT.mkdir(exist_ok=True)
    tempfile.tempdir = str(HERE)
    for name in sys.argv[1:] or ["encounter",*A]:
        film(name)
