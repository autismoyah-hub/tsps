"""Rebuild indexed models using their stream flags and triangle strip instructions.
Weighted and complex imports use an extended layout. --preserve-packing reuses the
source layout and rejects edits incompatible with it. Native authoring uses Mesh.encode.
"""
import struct
from author import native_codec

def canonical_plan(g,weights):
    flags=[]
    previous=(0,0,0)
    for p in g['vertices']:
        flags.append(sum(1<<i for i in range(3) if p[i]!=previous[i]))
        previous=p
    return dict(format='skinned-extended',priority=255 if 'priorities' in g else g['priority'],
                render='render_types' in g, alpha='alpha' in g,
                texture='face_textures' in g,fskin='face_skins' in g,vskin='vertex_skins' in g,bones=weights,
                flags=flags,ops=[1]*len(g['faces']),metadata='00',depth_marker=[int('face_flags' in g)])

def encode(g, plan, complex_sections=()):
    fmt = plan['format']
    extended = fmt in ('extended','skinned-extended')
    skinned = fmt.startswith('skinned')
    vs,fs=g['vertices'],g['faces']
    nv,nf=len(vs),len(fs)
    flags,ops=bytes(plan['flags']),bytes(plan['ops'])
    if len(flags)!=nv or len(ops)!=nf:
        raise ValueError('stored packing requires original topology')
    xyz=[bytearray() for _ in range(3)]
    previous=(0,0,0)
    for p,mask in zip(vs,flags):
        for axis in range(3):
            delta=p[axis]-previous[axis]
            if mask & (1<<axis):xyz[axis].extend(native_codec.smart(delta))
            elif delta:raise ValueError('vertex no longer fits original delta mask')
        previous=p
    indices=bytearray()
    olda=oldb=last=0
    for (a,b,c),op in zip(fs,ops):
        if op==1:
            ds=(a-last,b-a,c-b)
        else:
            pair={2:(olda,last),3:(last,oldb),4:(oldb,olda)}[op]
            if (a,b)!=pair:raise ValueError('changed triangle strip')
            ds=(c-last,)
        for delta in ds:indices.extend(native_codec.smart(delta))
        olda,oldb,last=a,b,c
    def stream(name,enabled):
        values=g.get(name,[0]*nf)
        if not enabled and any(values):raise ValueError('changed '+name)
        return bytes(values) if enabled else b''
    render=stream('render_types',plan['render'])
    alpha=stream('alpha',plan['alpha'])
    prio=plan['priority']
    priorities=bytes(g['priorities']) if prio==255 else b''
    if prio!=255 and any(p!=prio for p in g.get('priorities',[])):
        raise ValueError('changed uniform priority')
    skins=stream('face_skins',plan['fskin'])
    vertex=bytearray(g.get('vertex_skins',[]))
    weights=plan['bones']
    if weights is not None:
        for row in weights:
            vertex.append(len(row))
            for bone,weight in row:vertex.extend((bone,weight))
    types=bytes(g.get('texture_types',[]))
    triangles=b''.join(struct.pack('>3H',*tri) for t,tri in zip(types,g.get('texture_triangles',[])) if t==0)
    complex_data=b''.join(bytes.fromhex(s) for s in complex_sections)
    ids=g.get('face_textures',[-1]*nf)
    coords=g.get('texture_coords',[-1]*nf)
    colors=list(g['colors'])
    tex=coord=b''
    if extended and plan['texture']:
        tex=struct.pack('>'+str(nf)+'H',*(t+1 for t in ids))
        if types:coord=bytes(c+1 for c,t in zip(coords,ids) if t>=0)
    elif any(t>=0 for t in ids):
        if extended or not render:raise ValueError('texture stream absent in source layout')
        render=bytearray(render)
        for i,t in enumerate(ids):
            if t>=0:
                if not 0<=coords[i]<64:raise ValueError('legacy mapping index')
                colors[i]=t
                render[i]|=2|(coords[i]<<2)
    colors=struct.pack('>'+str(nf)+'H',*colors)
    prefix=(types if extended else b'')+flags+(render+ops+priorities+skins if extended else ops+priorities+skins+render)
    geometry=b''.join(xyz)+triangles+complex_data if extended else triangles+b''.join(xyz)
    body=prefix+vertex+alpha+indices+tex+coord+colors+geometry+bytes.fromhex(plan['metadata'])
    marker=bytes(plan['depth_marker'])
    body+=marker+stream('face_flags',marker==b'\1')
    header=struct.pack('>HHBBBBB',nv,nf,len(types),plan['render'],prio,plan['alpha'],plan['fskin'])
    if extended:header+=bytes([plan['texture']])
    header+=bytes([plan['vskin']])
    if skinned:header+=bytes([weights is not None])
    header+=struct.pack('>4H',*(len(s) for s in xyz),len(indices))
    if extended:header+=struct.pack('>H',len(coord))
    if skinned:header+=struct.pack('>H',len(vertex))
    suffix={'legacy':b'','extended':bytes([255,255]),'skinned-legacy':bytes([255,254]),'skinned-extended':bytes([255,253])}
    return body+header+suffix[fmt]
