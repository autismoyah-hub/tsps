"""Separate HD mapping and attachment streams from OSRS-compatible mesh streams."""
from hdcache import *
def translate(data):
    if data[-2:]!=b'\xff\xff':
        return data,dict(version='legacy')
    footer=bytearray(data[-23:])
    nv,nf,nt,flags,p,a,fs,ft,vs,x,y,z,idx,coord=struct.unpack('>HH7B5H',footer[:21])
    ver=data[-24] if flags&8 else 12
    if flags&~15 or ver not in (12,14,15):raise ValueError((flags,ver))
    ts=data[:nt]
    if any(t>3 for t in ts):raise ValueError('mapping type')
    simple=ts.count(0)
    count=nt-simple
    start=nt+nv+nf*((flags&1)+1+(p==255)+fs+a+2*ft+2)+nv*vs+x+y+z+idx+coord+6*simple
    r=Cursor(data[start:-24 if flags&8 else -23])
    maps=[]
    complex_types=[t for t in ts if t]
    axes=[struct.unpack('>hhh',r.take(6)) for _ in complex_types]
    widths={12:(2,2,2),14:(2,3,2),15:(3,3,3)}[ver]
    scales=[tuple(r.num(k) for k in widths) for _ in complex_types]
    rot=r.take(count)
    direction=r.take(count)
    for i,t in enumerate(complex_types):
        maps.append(dict(type=t,axis=axes[i],scale=scales[i],rotation=rot[i],direction=direction[i],
                         speed=r.u8(),cube=list(r.take(2)) if t==2 else []))
    effects={}
    if flags&2:
        for key in ['emitters','effectors']:
            effects[key]=[struct.unpack('>HH',r.take(4)) for _ in range(r.u8())]
    if flags&4:
        effects['billboards']=[struct.unpack('>HHBb',r.take(6)) for _ in range(r.u8())]
    if r.pos!=len(r.data):raise ValueError(f'unparsed HD tail {len(r.data)-r.pos}')
    footer[5]&=1
    return (data[:start]+bytes(17*count+2*ts.count(2))+b'\0'+footer,
            dict(version=ver,flags=flags,mappings=maps,attachments=effects))
def decode(data):
    raw,hd=translate(data)
    m=osrs.decode_model(raw)
    m['hd']=hd
    return m

