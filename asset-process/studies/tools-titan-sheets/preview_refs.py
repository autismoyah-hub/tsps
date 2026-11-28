import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from titan import *
sys.path.insert(0,str(BUILDS/'research-effects'))
import fxlib
for n in ATTACKS:
    data=json.loads((ROOT/'teacher-only'/str(n)/'fixtures/inventory.json').read_text())
    names=fxlib.names(7)
    seqs={s['id']:names.get(s['id'],'unknown') for s in data['sequences']}
    print(n,seqs,data['alpha'],data['part'],data['paint_faces'])
    write(ROOT/'teacher-only'/str(n)/'sequence-names.json',seqs)
    sh=ROOT/'sheets'/str(n)
    lit=Image.open(sh/'views/front-lit.png').convert('RGBA')
    lit.alpha_composite(Image.open(sh/'views/front-tlit.png'))
    lit.thumbnail((400,600))
    bg=Image.new('RGB',lit.size,(54,48,43)); bg.paste(lit,mask=lit.getchannel('A'))
    bg.save(ROOT/f'preview-{n}.png')

