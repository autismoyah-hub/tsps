import sys,pathlib,json,sqlite3,zlib,collections,struct
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from references import *
from PIL import Image
from definitions951 import BANK
c=Cache()
db=sqlite3.connect(ROOT/'assets.sqlite')
proof={}
for k,(a,s) in BANK.items():
    flags,rows=headers(c,a)
    for g,(crc,digest,(packed,expanded),plain) in rows.items():
        d=c.group(a,g)
        assert len(d)==expanded and zlib.crc32(d)==plain,(a,g)
        ix=c.idx[a][6*g:6*g+6]
        assert int.from_bytes(ix[:3],'big')==packed+2,(a,g,packed)
    proof[k]=dict(groups_checked=len(rows),definitions=db.execute("select count(*),count(name),sum(models!='[]'),sum(stop is null) from assets where kind=?",(k,)).fetchone())
rows=headers(c,26)[1]
d=c.group(26,0)
assert (len(d),zlib.crc32(d))==(rows[0][2][1],rows[0][3])
proof['material_checks']={}
m=json.loads((ROOT/'materials.json').read_bytes())
tex=set(c.index(52))
for v in (0,1,2):
    arr=[row for row in m.values() if row['layout']==v]
    refs=[t for r in arr for bit,tag,t in r['references']]
    proof['material_checks'][v]=dict(materials=len(arr),refs=len(refs),unique=len(set(refs)),unmatched=len(set(refs)-tex))
proof['unknown_materials']={k:r for k,r in m.items() if r['layout']>=2}
proof['npc_variants']=[list(row) for row in db.execute("select id,name,models,properties from assets where kind='npc' and id in (28069,28070,28526,28527)")]
proof['item_variants']=[list(row) for row in db.execute("select id,name,models,properties from assets where kind='item' and id in (51772,51819,47470,47475)")]
proof['figures']={}
for name in ('creatures','weapons','armour','effects'):
    im=Image.open(ROOT/(name+'.png')).convert('RGB')
    visible=[]
    for y in range(55,im.height,128):
        crop=im.crop((390,y+2,550,y+126))
        count=sum(rgb!=(32,39,44) for rgb in crop.get_flattened_data())
        visible.append(count)
    proof['figures'][name]=visible
assets=json.loads((ROOT/'pairs.json').read_bytes())
for row in assets:
    k={'Creatures':'npc','Armour':'item','Weapons':'item','Effects':'spot'}[row['category']]
    a,s=BANK[k]
    raw=c.files(a,row['id']>>s)[row['id']%2**s]
    if k!='spot':assert b'\x02'+row['name'].encode('cp1252')+b'\x00' in raw
    encoded=b''.join((g if g<32767 else (0x80000000|g)).to_bytes(2 if g<32767 else 4,'big') for g in row['models'])
    prefix=b'\x01'+(bytes([len(row['models'])]) if k=='npc' else b'')
    assert prefix+encoded in raw
assert all(all(v>0 for v in row) for row in proof['figures'].values())
modelrefs=headers(c,47)[1]
proof['study_model_sizes']={g:modelrefs[g][2][1] for row in assets for g in row['models'] if g>=0}
(ROOT/'verification.json').write_text(json.dumps(proof,indent=1))
print(json.dumps(proof)[:4400])

