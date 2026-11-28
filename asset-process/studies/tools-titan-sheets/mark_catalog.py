"""Human-readable contact sheets of the phase-local mark masks."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
from pathlib import Path
import json
from PIL import Image,ImageDraw,ImageFont
root=Path(__file__).resolve().parent
font=ImageFont.truetype(_AP.fix('<FONT_DIR>/segoeui.ttf'),15)
for n in (12596,14147):
    sh=root/'sheets'/str(n)/'marks'
    records=json.loads((sh/'index.json').read_text())
    for start in range(0,len(records),24):
        group=records[start:start+24]
        page=Image.new('RGB',(1000,126*((len(group)+3)//4)),(27,31,38))
        draw=ImageDraw.Draw(page)
        for j,r in enumerate(group):
            x=j%4*250;y=j//4*126
            draw.text((x+5,y+3),f"{r['mark']:03d}   {r['view']}",font=font,fill='white')
            for i,kind in enumerate(('paint','mask')):
                im=Image.open(sh/f"{r['mark']:03d}-{kind}.png").convert('RGB')
                scale=min(112/im.width,86/im.height)
                im=im.resize((max(1,round(im.width*scale)),max(1,round(im.height*scale))),Image.Resampling.NEAREST)
                page.paste(im,(x+i*122+(122-im.width)//2,y+32+(86-im.height)//2))
        page.save(sh/f'catalog-{start+1:03d}.png')

