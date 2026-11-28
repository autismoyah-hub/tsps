"""Check report links and reusable evidence on disk."""
import pathlib,re,json,sqlite3
from PIL import Image
p=pathlib.Path(__file__).resolve().parent
for name in ('REPORT.md','STUDY.md','RS3.md','TECHNIQUES.md'):
    text=(p/name).read_text(encoding='utf8')
    for dest in re.findall(r'\]\(([^)]+)\)',text):
        assert (p/dest).is_file(),(name,dest)
for art in ['creatures.png','weapons.png','armour.png','effects.png','viewer-check.png']:
    with Image.open(p/art) as im:im.verify()
d=json.loads((p/'disk-audit.json').read_bytes())
assert d['unaccounted']==0 and d['archives']['47']['stored']==0
pairs=json.loads((p/'pairs.json').read_bytes())
assert len(pairs)==33
db=sqlite3.connect((p/'assets.sqlite').as_uri()+'?mode=ro',uri=True)
assert db.execute('pragma quick_check').fetchone()==('ok',)
assert db.execute('select count(*) from assets').fetchone()==(248207,)
print('Four documents, all links, five PNGs, disk audit, 33 pairs and 248207 SQLite rows verified.')

