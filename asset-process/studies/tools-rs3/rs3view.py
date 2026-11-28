"""Render six views of a build-951 model or definition. usage: rs3view.py ID OUT.png [--blend F.blend]

ID: archive-47 model number, or model:N, item:N, item-worn:N, item-female:N, npc:N, or a decoded JSON path.
Decodes from the flat-file export, writes the interchange JSON with diffuse PNGs to a temporary folder,
and imports it with rs3blend.py (Blender, CPU Cycles, OSRS scale: RS3 units / 4, +Y up -> Blender +Z).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse, json, pathlib, sys, tempfile, subprocess
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
from rs3flat import ROOT
from PIL import Image, ImageDraw
BLENDER = _AP.fix('<BLENDER>')

def run(source, out, blend=None, title=None):
    with tempfile.TemporaryDirectory(dir=ROOT) as tmp:
        p = pathlib.Path(tmp)
        subprocess.run([BLENDER, '-b', '--factory-startup', '--disable-autoexec', '-t', '4', '--python-exit-code', '1',
                        '-P', str(ROOT/'rs3blend.py'), '--', str(source), str(blend or p/'inspect.blend'), str(p)],
                       check=True, stdout=subprocess.DEVNULL)
        sheet = Image.new('RGB', (660, 505), '#20272c')
        draw = ImageDraw.Draw(sheet)
        draw.text((8, 8), title or json.loads(source.read_bytes())['name'], fill='white')
        for i, label in enumerate(['Front', 'Right', 'Back', 'Left', 'Top', 'Three-quarter']):
            x = i % 3*220
            y = 25+i//3*240
            im = Image.open(p/f'{i}.png').convert('RGBA')
            sheet.paste(im, (x, y), im)
            draw.text((x+8, y+218), label, fill='white')
        sheet.save(out)

if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('model')
    ap.add_argument('out', type=pathlib.Path)
    ap.add_argument('--blend', type=pathlib.Path)
    a = ap.parse_args()
    if a.model.endswith('.json'):
        run(pathlib.Path(a.model).resolve(), a.out, a.blend)
    else:
        import rs3mesh
        with tempfile.TemporaryDirectory(dir=ROOT) as tmp:
            path, info = rs3mesh.export_json(a.model, tmp)
            title = f"{info['spec']}  {info['name']}  models {info['models']}  {info['vertices']} v  {info['faces']} f  mats {len(info['materials'])}  max bones {info['max_bones']}"
            run(path, a.out, a.blend, title[:110])
            print(json.dumps(info))
