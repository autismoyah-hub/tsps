import sys,json
from pathlib import Path
import numpy as np
from PIL import Image
root=Path(__file__).resolve().parent
def read(path):return json.loads(path.read_text())
for n in (12596,14147):
    sh=root/'sheets'/str(n); private=root/'teacher-only'/str(n)
    assert all(p.suffix in {'.md','.png','.json'} for p in sh.rglob('*') if p.is_file())
    meta=read(sh/'sheet.json');poses=read(sh/'pose.json')
    assert len(meta['views'])==18 and meta['sheet_version']==3
    for v in meta['views']:
        for k in ['paint','lit','ink','light','tpaint','tlit','tink','tlight','alpha']:
            assert Image.open(sh/f"views/{v['name']}-{k}.png").size==tuple(v['size'])
    for yaw,pitch,rows in poses['cameras']:
        for row in rows:
            for k in ('paint','tpaint','lit','light'):
                assert Image.open(sh/f"pose/{row.replace(':','')}-y{yaw}-p{pitch}-{k}.png").size==tuple(poses['cell'])
    assert len(list((sh/'actions').glob('*.png')))==48
    ids=set()
    for path in (sh/'game-marks').glob('*-marks.png'):
        ids.update(int(v) for v in np.unique(np.array(Image.open(path))) if v)
    assert len(ids)==read(private/'detail/detail.json')['visible']
    catalog=read(sh/'marks/index.json')
    for mk in catalog:
        assert mk['pixels']>0,mk
        mask=np.asarray(Image.open(sh/'marks'/f"{mk['mark']:03}-mask.png"))
        assert mask.sum()==mk['pixels']
    files=list(sh.rglob('*.png'))
    for p in files:
        with Image.open(p) as im:im.verify()
    for j in meta['labels'].values():assert 'min' not in j and (j['vertices']>1 or 'centroid' not in j)
    d=read(private/'fixtures/inventory.json')
    part=meta['labels'][str(d['part'])]
    print(n,len(files),'pictures;',len(catalog),'mark masks;',len(ids),'qualified game marks.',d['part'],part)
    for suffix in 'ab':
        folder=root.parent/f'titan-rebuild-{n}-{suffix}'
        assert [p.name for p in folder.iterdir()]==['README.md']
        assert (folder/'README.md').read_text().count('teacher_gate.py rebuild-npc ' + str(n))==1
    for i in range(3):
        im=np.array(Image.open(sh/f'comparison-{i}.png'))
        board=Image.open(root/'royal-titans-and-tekton.png')
        x=0 if n==12596 else 1000
        y=i*592+32
        assert np.array_equal(im,np.array(board.crop((x,y,x+500,y+560))))

