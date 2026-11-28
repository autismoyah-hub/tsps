import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from titan import *
import ortho
from PIL import ImageFont
def font(s):
    return ImageFont.truetype(_AP.fix('<FONT_DIR>/segoeui.ttf'),s)
for n in ATTACKS:
    setup(n,'materials')
    sh=ROOT/'sheets'/str(n)
    m,_=tl.real_mesh(n);a=tl.arrays(m)
    lo,hi=a['V'].min(0),a['V'].max(0)
    vals=np.unique(a['A'])
    page=Image.new('RGB',(220*4,340*((len(vals)+3)//4)),(31,34,40)); pen=ImageDraw.Draw(page)
    for k,v in enumerate(vals):
        x,y=k%4*220,k//4*340
        sel=np.where(a['A']==v)[0]
        for j,(_,c) in enumerate([ortho.VIEWS[0],ortho.VIEWS[2]]):
            fr=ortho.frame(lo,hi,c,.46)
            ids,_,_=ortho.raster(a['V'],a['F'],c,fr,.46,sel,cull=False)
            pic=np.full((*ids.shape,3),[31,34,40],np.uint8)
            pic[ids>=0]=tl.TABLE[a['C'][ids[ids>=0]]]
            im=Image.fromarray(pic);im.thumbnail((106,270))
            page.paste(im,(x+110*j+(106-im.width)//2,y+42))
        pen.text((x+8,y+8),f'Raw {v}   ·   {len(sel)} faces',font=font(14),fill='white')
    page.save(sh/'alpha-bands.png')
    # Registered game-size footprints, using detailgate's actual visibility tests.
    w=ROOT/'teacher-only'/str(n)/'detail'
    out=sh/'game-marks';out.mkdir(exist_ok=True)
    ref=np.load(w/'px'/f'real-{n}.npz')
    stats=json.loads((w/'results'/f'ceiling-{n}.json').read_text())
    _,marks=dlib.mesh_marks(a,120)
    lut=np.zeros(len(m['faces'])+1,dtype='uint16')
    for j,mk in enumerate(marks,1):
        if not mk['translucent']:lut[mk['faces']]=j
    for k in ref.files:
        if not k.startswith('game|') or not k.endswith('|face'):continue
        ids=ref[k];cn,y,row,_=k.split('|')
        active=np.zeros(len(marks)+1,dtype=bool)
        for j,rec in enumerate(stats['real'],1):
            for inst in rec['game_inst']:
                if inst['view']==f'{y}|{row}' and inst['px']>=dlib.PX_MIN and inst['effect']>=dlib.C_MIN:active[j]=True
        img=lut[ids];img[~active[img]]=0
        basename=f"{row.replace(':','')}-y{y}"
        Image.fromarray(img).save(out/f'{basename}-marks.png')
        lit=ref[k[:-4]+'lit']; overlay=lit.copy();overlay[img>0]=[255,125,32]
        Image.fromarray(np.concatenate([lit,overlay],axis=1)).save(out/f'{basename}-key.png')
    print(n,'game instance example',next((x['game_inst'][:1] for x in stats['real'] if x['game_inst']),[]))
pages=[Image.open(ROOT/f'board-{n}-tekton.png') for n in ATTACKS]
assert np.array_equal(np.array(pages[0])[:,500:],np.array(pages[1])[:,500:])
board=Image.new('RGB',(1500,1776))
for i,(page,box) in enumerate([(pages[0],(0,0,500,1776)),(pages[0],(500,0,1000,1776)),(pages[1],(0,0,500,1776))]):
    board.paste(page.crop(box),(500*i,0))
board.save(ROOT/'royal-titans-and-tekton.png')
board.thumbnail((750,888));board.save(ROOT/'teacher-only/board-preview.png')

