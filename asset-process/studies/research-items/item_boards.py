"""Render flat-ground controls, compose the eight boards and count motion pixels before annotation."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import pathlib,sys
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from finish_items import *
from scipy.ndimage import label
from PIL import ImageOps
def font(size,bold=False):
    return ImageFont.truetype(_AP.fix('<FONT_DIR>/')+('segoeuib.ttf' if bold else 'segoeui.ttf'),size)
def crop(work,g,act,c,frame=0):
    with Image.open(work/f'{g}-{act}.bmp') as im:return im.crop((200*c,250*frame,200*c+200,250*frame+250)).convert('RGB')
def audit(row,work):
    m=L.load_bin(source(row)); f=L.load_bin(work/'female.bin')
    def normals(v):
        points=v['V'][v['F']]
        return np.cross(points[:,1]-points[:,0],points[:,2]-points[:,0])
    n,nf=normals(m),normals(f)
    valid=np.any(n,axis=1)
    collapsed=np.flatnonzero(valid&~np.any(nf,axis=1))
    reversed_=np.flatnonzero(np.sum(n*nf,axis=1)<0)
    def dup(v):return len(v)-len(np.unique(v,axis=0))
    report={'new_collapsed_faces':collapsed.tolist(),'over_90deg_normal_change_faces':reversed_.tolist(),'duplicate_coordinates_male_female':[dup(m['V']),dup(f['V'])],'motion':{}}
    for g in range(2):
        for act in ('stand','walk','run','slash'):
            for j in range(1 if act=='stand' else 2):
                b,r,o=[np.array(crop(work,g,act,c,j)) for c in range(3)]
                bg=b[0,0]
                changed=lambda a,b:int(np.any(a!=b,axis=2).sum())
                def gaps(a):
                    sky=np.all(a==bg,axis=2)
                    ids,_=label(sky)
                    return int(np.count_nonzero(sky & (ids!=ids[0,0])))
                report['motion'][f'{g}-{act}-{j}']={'real_minus_bare':changed(r,b),'ours_minus_bare':changed(o,b),'enclosed_background_real_ours':[gaps(r),gaps(o)]}
    write(work/'geometry-motion.json',report)
def board(row):
    slug,_,slot,_,kind,ref,stand,walk=row
    work=OUT/slug
    log=work/'ground.txt';log.write_text('')
    ids=json.loads((work/'inventory.view.json').read_text())['row']
    run([_AP.exe('meridian_ground.exe', L.EXE_DIR),L.CACHE,work/'ground.bmp',f'item:{ids[1]}',f'def:{work}/item.def+model:{work}/inventory.bin',f'item:{ids[-1]}'],log)
    audit(row,work)
    canvas=Image.new('RGB',(822,722),'#202421')
    d=ImageDraw.Draw(canvas)
    text=lambda xy,s,size=15,bold=False,color='#e5e5d8':d.text(xy,s,font=font(size,bold),fill=color)
    text((27,14),'Meridian '+slug,30,True)
    text((28,61),'Inventory',15,True)
    icons=Image.open(work/'icons.bmp').resize((648,64),Image.Resampling.NEAREST)
    canvas.paste(icons,(87,90))
    for i,id in enumerate(ids[:4]+[None]+ids[4:]):
        s=str(id) if id else 'Meridian'
        d.text((123+72*i,160),s,font=font(13,id is None),anchor='mt',fill='#ffd278' if id is None else '#c0c8bb')
    text((28,187),'Ground',15,True)
    text((123,188),f'{L.items()[ids[1]]["name"]}     /     Meridian {slug}     /     {L.items()[ids[-1]]["name"]}',14)
    canvas.paste(Image.open(work/'ground.bmp'),(28,215))
    for i,(g,act,j) in enumerate([(0,'stand',0),(0,'walk',1),(1,'stand',0),(1,'walk',1)]):
        x=11+200*i
        text((x+8,438),('Male' if g==0 else 'Female')+'   '+act,15,True)
        canvas.paste(crop(work,g,act,2,j),(x,465))
    canvas.save(RENDER/(slug+'.png'))
if __name__=='__main__':
    for row in DATA:
        if len(sys.argv)==1 or row[0] in sys.argv[1:]:board(row)

