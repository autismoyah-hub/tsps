import hashlib
import json
from pathlib import Path
import sys
import numpy as np
from PIL import Image
sys.dont_write_bytecode = True
sys.path.insert(0,str(Path(__file__).resolve().parent))
import quality as Q

def rgb_key(im):
    return hashlib.sha256(im.convert("RGB").tobytes()).hexdigest()

def main():
    shots=[rgb_key(Image.open(Q.HERE/f"edge-frame-{i}.png")) for i in (0,1)]
    results={}
    for tag in ("boss","watcher","hound"):
        frames=[]
        for folder in ("phase1","phase1-hq"):
            ext="gif" if folder=="phase1" else "webp"
            p=Q.ROOT/"sittings"/folder/tag/("lineup."+ext)
            with Image.open(p) as im:
                holds=[]
                keys=[]
                edge_pixels=0
                for k in range(im.n_frames):
                    im.seek(k)
                    rgb=im.convert("RGB")
                    holds.append(im.info["duration"])
                    if folder=="phase1-hq":
                        keys.append(rgb_key(rgb))
                        a=np.asarray(rgb)[26:]
                        w=a.shape[1]//2
                        edge=np.concatenate([a[0],a[-1],a[:,0],a[:,-1],a[:,w-1],a[:,w]])
                        edge_pixels=max(edge_pixels,int((edge!=Q.BG).any(1).sum()))
                frames.append(dict(file=p.as_posix(),frames=len(holds),duration=sum(holds),
                                   bytes=p.stat().st_size,border_pixels_max=edge_pixels if keys else None))
                if keys and tag=="boss":
                    results["edge_frames"]=[keys.index(shot) if shot in keys else None for shot in shots]
        results[tag]=frames
        assert frames[1]["border_pixels_max"] == 0, f"{tag} touches a cell edge"
        assert frames[0]["duration"] == frames[1]["duration"], f"{tag} clock mismatch"
    assert None not in results["edge_frames"] and len(set(results["edge_frames"])) == 2, "Edge playback mismatch"
    print(json.dumps(results,indent=2))
    (Q.HERE/"delivery.json").write_text(json.dumps(results,indent=2))

if __name__=="__main__":
    main()
