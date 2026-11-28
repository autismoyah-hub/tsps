"""Show the visibility classes side by side under the same applied animation."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from titan import *
from PIL import ImageFont
n=int(sys.argv[1]); w=setup(n,'phase-poses');sh=ROOT/'sheets'/str(n)
cls=np.load(w.parent/'phases/visibility.npy')
seq=L.sequences(n);m,kv=tl.real_mesh(n);d=dict(tl.real_def(n),zbuffer=kv['zbuffer']=='1')
classes=[p for p in ('both','rest only','live only','death only','never') if np.any(cls==p)]
entries=[L.entry_real(n,seq)]
for i,p in enumerate(classes):
    v=tl.clone(m)
    v['alpha']=[x if keep else 255 for x,keep in zip(v['alpha'],cls==p)]
    entries.append(tl.entry(tl.write_variant(f'phase-{i}',v,d,'isolated'),seq))
rows=['bind','stand:4','walk:4','attack:8','death:22','death:31']
for yaw in (192,1216):
    grid,msg=tl.draw(entries,rows,tl.camera(n,(410,410)),yaw)
    assert 'NOT APPLIED' not in msg and 'NO `' not in msg
    page=Image.new('RGB',(len(entries)*410,len(rows)*440),(30,33,39))
    pen=ImageDraw.Draw(page)
    for y,(row,cs) in enumerate(zip(rows,grid)):
        for x,(p,c) in enumerate(zip(['Complete']+classes,cs)):
            page.paste(Image.fromarray(c),(410*x,440*y+30))
            pen.text((410*x+8,440*y+4),f'{row}    {p}',font=ImageFont.truetype(_AP.fix('<FONT_DIR>/segoeui.ttf'),16),fill='white')
    page.save(sh/f'phase-poses-y{yaw}.png')

