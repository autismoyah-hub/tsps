"""Read back the delivered model, comparisons, sample atlas and links."""
import hashlib,json,re,sys
from pathlib import Path
from PIL import Image
HERE=Path(__file__).resolve().parent
TOOLS=HERE.parents[1]/'tools'
sys.path.insert(0,str(TOOLS))
import osrscache
def rows(file):
    return [json.loads(line) for line in (HERE/file).read_text().splitlines()]
m=osrscache.decode_model((HERE/'lantern.bin').read_bytes())
assert (len(m['faces']),m['face_textures'].count(0),m['alpha'].count(100))==(216,176,16)
assert sum(t==0 and a==48 for t,a in zip(m['face_textures'],m['alpha']))==16
assert (HERE/'gate.log').read_text().strip().endswith('VERDICT PASS')
cases=json.loads((HERE/'proof.json').read_bytes())
assert len(cases)==15 and all(r['different']==[0]*3 and min(r['visible'])>0 for r in cases)
inv=json.loads((HERE/'inventory.json').read_bytes())
final={r['hash']:r for r in rows('binary-regression.jsonl')}
assert set(inv)==set(final) and all(final[h]['exact'] and final[h]['count']==len(paths) for h,paths in inv.items())
src=rows('blend-regression.jsonl')
assert len(src)==762 and sum(map(lambda r:len(r['trials']),src))==821 and all(all(r['trials']) for r in src)
report=(HERE/'REPORT.md').read_text(encoding='utf-8')
assert (HERE/'gate.log').read_text().strip() in report
for doc in ('REPORT.md','FORMAT.md'):
    for target in re.findall(r'\]\(([^)]+)\)',(HERE/doc).read_text(encoding='utf-8')):
        assert (HERE/target).is_file(),target
for name,size in [('board-textures.png',(864,3348)),('lantern-classic.png',(960,320))]:
    with Image.open(HERE/name) as im:
        assert im.size==size
        im.verify()
print('Verified 45 view differences,',sum(map(len,inv.values())),'stored binaries,',len(src),'Blender sources, lantern channels, gate output, PNGs and report links.')
