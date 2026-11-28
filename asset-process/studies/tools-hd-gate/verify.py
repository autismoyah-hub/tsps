"""Check the public commands, published author packets and hdview rendering baseline."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).parent))
from hdasset import *
TARGETS=[('object',22772),('item-ground',7535),('item-worn',7535),('item-female',7535),('spot',2768)]
from PIL import Image
from tempfile import TemporaryDirectory

def verify():
    rows=[]
    for kind,i in TARGETS:
        name='lampstall' if kind=='object' else 'spot2768' if kind=='spot' else 'diving'
        author=ROOT.parent/('hd-rebuild-'+name)
        folder=author/'sheet'
        if kind.startswith('item'):folder/=kind
        assert (author/'README.md').is_file()
        assert all(f.suffix.lower() in ('.png','.md','.json') for f in author.rglob('*') if f.is_file())
        meta=json.loads((folder/'sheet.json').read_text())
        for p in folder.rglob('*.png'):
            with Image.open(p) as im:im.verify()
        for p in (folder/'swatches').glob('*.png'):
            assert Image.open(p).size==(128,128)
        assert len(list((folder/'swatches').glob('*.png')))==meta['texture_count']
        for mode in ('lit','unlit','flat-lit','flat-unlit','ids','alpha'):
            assert Image.open(folder/(mode+'.png')).size==(768,616)
        control=ROOT/'private-proof'/kind/'ceiling.bin'
        dest=control.parent/'cli'
        proc=subprocess.run([sys.executable,'-I','-B',ROOT/'hdgate.py','rebuild','614',kind,str(i),control,'--out',dest],capture_output=True,text=True)
        if proc.returncode:raise RuntimeError(proc.stdout+proc.stderr)
        assert proc.stdout.splitlines()==(control.with_suffix('.txt')).read_text().splitlines()==(dest/'score.txt').read_text().splitlines()
        Image.open(dest/'comparison.png').verify()
        d,parts=target(614,kind,i)
        m=combined(parts)
        # Run the unmodified hdview executable against its ordinary input layout in private scratch.
        with TemporaryDirectory(dir=ROOT) as tmp:
            p=Path(tmp)
            (p/'model.bin').write_bytes(hdview.packed(parts))
            for t in set(m['face_textures'])-{-1}:(p/f't{t}.rgba').write_bytes(preview(cache(614),t)[0].tobytes())
            if 'uv' in m:(p/'projected.uv').write_bytes(m['uv'].astype('<f4').tobytes())
            subprocess.run([_AP.exe('hdclassic.exe', ERA / 'target/release'),p],check=True)
            ref=np.stack([np.frombuffer((p/f'{j}.rgb').read_bytes(),np.uint8).reshape(256,256,3) for j in range(6)])
        actual=render(m,614,framing(m['vertices']))['lit']
        difference=int((ref!=actual).any(3).sum())
        rows.append(dict(kind=kind,hdclassic_differing_pixels=difference,compared=6*256**2,cli_verdict='PASS',sheet_pngs=len(list(folder.rglob('*.png')))))
        print(rows[-1],flush=True)
    # Smoke a raw model sheet and gate on both other HD disk layouts.
    for rev,i in [(561,44579),(601,25)]:
        with TemporaryDirectory(dir=ROOT) as tmp:
            p=Path(tmp)
            subprocess.run([sys.executable,'-I','-B',ROOT/'hdsheet.py',str(rev),'model',str(i),p/'sheet'],check=True)
            d,parts=target(rev,'model',i)
            (p/'control.bin').write_bytes(hdview.packed(conversion(rev,d,parts)[0]))
            proc=subprocess.run([sys.executable,'-I','-B',ROOT/'hdgate.py','rebuild',str(rev),'model',str(i),p/'control.bin'],capture_output=True,text=True)
            if proc.returncode:raise RuntimeError(proc.stdout+proc.stderr)
            rows.append(dict(revision=rev,model=i,cli_verdict=proc.stdout.splitlines()[-1]))
    (ROOT/'verification.json').write_text(json.dumps(rows,indent=2))

if __name__=='__main__':verify()

