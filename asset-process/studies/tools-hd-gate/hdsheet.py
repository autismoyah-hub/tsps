"""Generate author-safe views, surface statistics and posed frames."""
import argparse
import colorsys
import math
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from hdasset import *
from PIL import ImageDraw

ORDER=[(0,'Front (-Z)'),(2,'Back (+Z)'),(3,'Left (-X)'),(1,'Right (+X)'),(5,'Three quarter'),(4,'Top')]
def ink(t):
    if t==-1:return (126,132,140)
    return tuple(round(x*255) for x in colorsys.hsv_to_rgb((t*.61803398875)%1,.75,.95))

def page(images,labels,title,detail,path,columns=3):
    w,h=images[0].size
    rows=math.ceil(len(images)/columns)
    board=Image.new('RGB',(columns*w,56+rows*(h+24)),(24,31,40))
    draw=ImageDraw.Draw(board)
    draw.text((9,8),title,fill='white')
    draw.text((9,31),detail,fill='#b8c8db')
    for j,(im,label) in enumerate(zip(images,labels)):
        x=j%columns*w
        y=56+j//columns*(h+24)
        board.paste(im,(x,y))
        draw.text((x+8,y+h+3),label,fill='white')
    board.save(path)

def make(rev,kind,number,out):
    out.mkdir(parents=True,exist_ok=True)
    d,parts=target(rev,kind,number)
    m=combined(parts)
    camera=framing(m['vertices'])
    modes=['lit','unlit','flat-lit','flat-unlit','ids','alpha']
    results=render(m,rev,camera,['ortho-'+s for s in modes])
    title=f"r{rev}   {kind} {number}   {d['name']}"
    detail=f"{len(parts)} part(s)    {len(m['vertices'])} vertices    {len(m['faces'])} faces    |    {475/512/camera[3]:.3f} game units per pixel"
    counts={}
    weights=area(m)
    stats=dict(revision=rev,kind=kind,id=number,name=d['name'],parts=len(parts),
        vertices=len(m['vertices']),faces=len(m['faces']),bounds_min=np.min(m['vertices'],axis=0).tolist(),
        bounds_max=np.max(m['vertices'],axis=0).tolist(),part_face_counts=[len(p['faces']) for p in parts])
    for field in ('colors','face_textures','alpha'):
        keys,counts=np.unique(m[field],return_counts=True)
        stats[field]=dict(distinct=len(keys),face_counts=dict(zip(map(int,keys),map(int,counts))),area_shares=shares(m[field],weights))
    stats['texture_count']=len(set(m['face_textures'])-{-1})
    stats['nonzero_alpha_count']=len(set(m['alpha'])-{0})
    for mode in modes:
        arr=results['ortho-'+mode]
        if mode in ('alpha','ids'):
            floor=-2 if mode=='ids' else -1
            vis=np.full((*arr.shape,3),40,np.uint8)
            for val in np.unique(arr[arr>floor]):
                color=ink(val) if mode=='ids' else (255-int(val),)*3
                vis[arr==val]=color
            arr=vis
        page([Image.fromarray(arr[i]) for i,_ in ORDER],[s for _,s in ORDER],title+'   |   '+mode,detail,out/(mode+'.png'))
    swatches=out/'swatches'
    swatches.mkdir(exist_ok=True)
    stats['map_key']={'background':'#282828','untextured':'#7e848c','alpha':'grey = 255 minus raw transparency','texture_rgb':{}}
    for t in sorted(set(m['face_textures'])-{-1}):
        preview(cache(rev),t)[0].save(swatches/f'{t}.png')
        stats['map_key']['texture_rgb'][t]=ink(t)
    if stats['texture_count']:
        ts=sorted(set(m['face_textures'])-{-1})
        tiles=[]
        for t in ts:
            im=Image.open(swatches/f'{t}.png').convert('RGBA')
            tile=Image.new('RGB',(128,140),(40,40,40))
            tile.paste(im,(0,0),im)
            ImageDraw.Draw(tile).rectangle((0,128,127,139),fill=ink(t))
            tiles.append(tile)
        page(tiles,[f'HD {t}' for t in ts],title,'Band below each swatch is its colour in ids.png',out/'materials.png',min(5,len(ts)))
    if kind=='spot' and 'sequence' in d:
        for channel in ('vertex','face'):
            groups=m[channel+'_skins']
            if groups is None or (channel=='face' and not m['has_face_skin']):
                continue
            labels=groups
            if channel=='vertex':
                labels=[]
                for f in m['faces']:
                    a,b,c=(groups[i] for i in f)
                    labels.append(b if b==c else a)
            group_map=render(dict(m,colors=labels),rev,camera,('ortho-groups',))['ortho-groups']
            vis=np.full((*group_map.shape,3),40,np.uint8)
            stats[channel+'_label_rgb']={}
            for g in sorted(set(groups)):
                vis[group_map==g]=ink(g)
                stats[channel+'_label_rgb'][int(g)]=ink(g)
            page([Image.fromarray(vis[i]) for i,_ in ORDER],[s for _,s in ORDER],title+'   |   '+channel+' labels',
                 'Colour key in sheet.json. Vertex map uses the majority label on each triangle, first corner on ties.',out/(channel+'-labels.png'))
        holds,poses=animation(rev,d['sequence'],m)
        cam=framing([v for p in poses for v in p['vertices']])
        raw=[render(p,rev,cam)['lit'][5] for p in poses]
        foreground=np.any(np.any(np.asarray(raw)!=(38,42,48),axis=3),axis=0)
        ys,xs=np.where(foreground)
        frames=[Image.fromarray(a) for a in raw]
        if len(xs):
            span=max(np.ptp(xs),np.ptp(ys))+12
            left=int((xs.max()+xs.min()-span)/2)
            top=int((ys.max()+ys.min()-span)/2)
            frames=[im.crop((left,top,left+span,top+span)) for im in frames]
        frames=[im.resize((160,160),Image.Resampling.LANCZOS) for im in frames]
        page(frames,[f'{i+1}   hold {h}' for i,h in enumerate(holds)],title,'Sequence '+str(d['sequence'])+'   |   fixed camera across all frames',out/'sequence.png',min(5,len(holds)))
        stats['sequence']=dict(id=d['sequence'],frames=len(holds),holds=holds,cycles=sum(holds))
    (out/'sheet.json').write_text(json.dumps(stats,indent=2))
    print(out,flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('rev',type=int,choices=(561,601,614))
    p.add_argument('kind',choices=KINDS)
    p.add_argument('id',type=int)
    p.add_argument('out',type=Path)
    a=p.parse_args()
    if a.kind=='item':
        for k in ('item-ground','item-worn','item-female'):
            if k=='item-ground' or any(j in itemdefs.definition(cache(a.rev),'item',a.id).get('wear',{}) for j in ((23,24,78) if k=='item-worn' else (25,26,79))):
                make(a.rev,k,a.id,a.out/k)
    else:make(a.rev,a.kind,a.id,a.out)

