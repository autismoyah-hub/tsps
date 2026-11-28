"""Review camera presets. Run with -I. Inputs use native cache units and native RGB."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse
import bisect
import itertools
import math
import os
from pathlib import Path
import random
import subprocess
import sys
from tempfile import TemporaryDirectory

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "studies/research-creatures"))
import rebuild_lib as L

# Native flat-terrain pitch 1024 / 8, fixed viewport height 334.
PITCH = 128
DISTANCE = 600 + 3 * PITCH
FOCAL = 256
AIM = 50
YAW = 192
BG = (48, 43, 37)

def raster_factor(preset, factor):
    if factor < 1 or (preset == "hires" and factor not in (2, 3, 4)):
        raise ValueError("Choose a positive integer; hires supports 2, 3, 4")
    return factor if preset == "hires" else 1

def magnify(im, preset, factor):
    if preset == "magnified":
        return im.resize((im.width * factor, im.height * factor), Image.Resampling.NEAREST)
    return im

def draw(entries, rows, scratch, *, focal=FOCAL, distance=DISTANCE, aim=AIM,
         size=(300, 300), reference=False):
    dest = scratch / "capture.bmp"
    exe = L.cmesh.EXE if reference else _AP.exe('quality_npc.exe', HERE)
    args = [exe, L.cmesh.CACHE, "draw", dest, "rows=" + ",".join(rows), f"zoom={distance}",
            f"yaw={YAW}", f"pitch={PITCH}", f"cell={size[0]}x{size[1]}", f"aim={aim}", *entries]
    result = subprocess.run(args, env={**os.environ, "NPC_REVIEW_FOCAL": str(focal)},
                            capture_output=True, text=True, check=True)
    if "NOT APPLIED" in result.stdout or "WARNING" in result.stdout:
        print(result.stdout, file=sys.stderr)
    with Image.open(dest) as capture:
        return capture.convert("RGB")

def pair(real, model, seed, scratch):
    seqs = L.sequences(real)
    entries = [L.entry_real(real, seqs), L.entry_ours(model, seqs, scratch)]
    random.Random(seed).shuffle(entries)
    return entries, seqs

def schedule(seqs, step=60):
    rows, holds = [], []
    for name in ("stand", "walk", "attack", "death"):
        if name in seqs:
            n, length, _, _ = L.seq_frames(seqs[name])
            for i in range(n):
                rows.append(f"{name}:{i}")
                holds.append(max(20, int(20 * length / n)))
            if n:
                holds[-1] += 500
    ends = list(itertools.accumulate(holds))
    indices = [bisect.bisect_right(ends, t) for t in range(0, ends[-1], step)]
    return [(rows[i], step * len(list(group))) for i, group in itertools.groupby(indices)]

def numbered(im):
    frame = Image.new("RGB", (im.width, im.height + 26), (20, 18, 16))
    frame.paste(im, (0, 26))
    pen = ImageDraw.Draw(frame)
    for k in range(2):
        pen.text((im.width // 2 * k + 8, 2), str(k + 1), fill=(245,)*3,
                 font=ImageFont.truetype("segoeuib.ttf", 20))
    return frame

def actor_strips(entries, rows, scratch, res, vertical_offset):
    # An off-centre crop of a taller raster keeps tall heads in the review window.
    # It moves the viewport on the picture plane and preserves perspective and scale.
    width, height = 300 * res, 600 * res
    image = draw(entries, rows, scratch, size=(width, height), focal=FOCAL*res)
    top = (150 - vertical_offset) * res
    return [image.crop((0, k*height + top, 2*width, k*height + top + width))
            for k in range(len(rows))]

def crop_offset(real, scratch):
    cached = L.cmesh.DATA / "merged" / f"npc-{real}.txt"
    if not cached.is_file():
        cached = scratch / "rest.txt"
        subprocess.run([L.cmesh.EXE,L.cmesh.CACHE,"merged",f"npc:{real}",cached],
                       check=True,capture_output=True)
    mesh = L.cmesh.load_file(cached)
    rest_y = mesh["V"][np.unique(mesh["F"]),1]
    height = np.ptp(rest_y) * L.npc_def(real)["scale"][1] / 128
    # Translate the crop only. Never change model translation or the camera distance.
    return min(150, max(0, round((height/2 - AIM)*FOCAL/DISTANCE)))

def verify_motion(path, frames, holds):
    with Image.open(path) as dec:
        if dec.info.get("loop") != 0:
            raise ValueError("Loop metadata mismatch")
        t = 0
        ends = list(itertools.accumulate(holds))
        for i in range(dec.n_frames):
            dec.seek(i)
            rgb = dec.convert("RGB")
            end = t + dec.info["duration"]
            while t < end:
                idx = bisect.bisect_right(ends, t)
                if frames[idx].tobytes() != rgb.tobytes():
                    raise ValueError(f"WebP RGB mismatch at {t} ms")
                t = min(end, ends[idx])
        if t != ends[-1]:
            raise ValueError("WebP duration mismatch")
        return dec.n_frames, t

def save_motion(path, frames, holds):
    """Encode final RGB frames, including captions, and verify the entire timeline."""
    if len(frames) != len(holds) or not frames or min(holds) < 1:
        raise ValueError("Supply a positive duration for every frame")
    path = Path(path)
    if path.exists():
        raise FileExistsError(path)
    with TemporaryDirectory(dir=path.parent) as scratch:
        staged = Path(scratch) / "motion.webp"
        frames[0].save(staged, format="WEBP", save_all=True, append_images=frames[1:],
                       duration=holds, loop=0, lossless=True, method=4)
        result = verify_motion(staged, frames, holds)
        staged.rename(path)
        return result

def render_npc(real, model, seed, preset, factor, output, pose):
    res = raster_factor(preset, factor)
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    with TemporaryDirectory(dir=output.parent) as folder:
        work = Path(folder)
        entries, seqs = pair(real, model.resolve(), seed, work)
        motion = output.suffix.lower() == ".webp"
        plan = schedule(seqs) if motion else [(pose, 1)]
        frames = []
        offset = crop_offset(real, work)
        for start in range(0, len(plan), 8):
            chunk = plan[start:start+8]
            for strip in actor_strips(entries, [row for row, _ in chunk], work, res, offset):
                frames.append(magnify(numbered(strip), preset, factor))
        tmp = work / output.name
        if motion:
            holds = [ms for _, ms in plan]
            count, total = save_motion(tmp, frames, holds)
            print(f"{output}: {count} decoded frames, {total} ms, exact RGB at all holds")
        else:
            frames[0].save(tmp, format="PNG")
        tmp.rename(output)

def arena_settings(preset, factor, tile, yaw):
    res = raster_factor(preset, factor)
    angle = PITCH * math.tau / 2048
    return [f"size {512*res} {334*res}", f"focal {FOCAL*res}",
            f"camera {tile[0]} {tile[1]} {yaw} {DISTANCE*math.cos(angle):.8f} {DISTANCE*math.sin(angle):.8f} {AIM}"]

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("out", type=Path, help="new .png (pose) or .webp (motion)")
    parser.add_argument("real", type=int)
    parser.add_argument("model", type=Path)
    parser.add_argument("--seed", type=int, required=True)
    parser.add_argument("--preset", choices=["faithful","magnified","hires"], default="faithful")
    parser.add_argument("--factor", type=int, default=2)
    parser.add_argument("--pose", default="stand:0")
    args = parser.parse_args()
    if args.out.suffix.lower() not in (".png",".webp"):
        parser.error("Use .png or .webp")
    render_npc(args.real,args.model,args.seed,args.preset,args.factor,args.out,args.pose)
