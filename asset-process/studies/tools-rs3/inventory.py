import sys,pathlib,json
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import *
c=Cache()
rows={}
for a in sorted(c.idx):
    if a==255: continue
    try:
        ix=c.index(a)
        g=next(iter(ix))
        row=dict(groups=len(ix),files=sum(map(len,ix.values())),sample_id=g)
        rows[a]=row
        if any(c.idx[a][6*g:6*g+3]):
            d=c.group(a,g)
            row.update(sample_bytes=len(d),prefix_hex=d[:64].hex())
        else:row['sample_status']='absent from disk index'
    except Exception as e: rows[a]=str(e)
print(json.dumps(rows,indent=1))
(ROOT/'inventory.json').write_text(json.dumps(rows,indent=1))

