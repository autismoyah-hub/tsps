import sys,pathlib,json,collections
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from models import *
summary={}
for rev in [239,242,561,601,614]:
 c=Cache(rev);details=dict(indexes={},versions={},triangle_opcodes=[],vertex_flags=[],texture_types=[],render_values=[],attachment_models={},examples={})
 for a in c.idx:
    if a!=255:
        ix=c.index(a);details['indexes'][a]=[len(ix),sum(len(f) for f in ix.values())]
 if rev in (239,242):
    details['config_groups']={g:len(c.index(2)[g]) for g in (6,9,10,12,13)}
    details['texture_sizes']=dict(collections.Counter(map(len,c.files(9,0).values())))
 else:
    versions=collections.Counter();ops=set();vflags=set();types=set();render=set();atts=collections.Counter();examples={}
    for i in c.index(7):
        d=c.group(7,i);_,hd=translate(d);versions[str(hd['version'])]+=1
        ext=hd['version']!='legacy';f=d[-23:] if ext else d[-18:]
        nv,nf,nt,flag,prio,al,fs=struct.unpack('>HH5B',f[:9])
        vstart=nt if ext else 0;vflags.update(d[vstart:vstart+nv])
        if ext:types.update(d[:nt])
        elif nt:types.add(0)
        opstart=vstart+nv+nf*int(ext and flag&1)
        ops.update(d[opstart:opstart+nf])
        if flag&1:
            p=nt+nv if ext else nv+nf*(1+(prio==255)+fs)
            render.update(d[p:p+nf] if ext else [v&1 for v in d[p:p+nf]])
        for k,rows in hd.get('attachments',{}).items():
            if rows:atts[k]+=1;examples.setdefault(k,[i,rows])
    details.update(versions=versions,triangle_opcodes=sorted(ops),vertex_flags=sorted(vflags),texture_types=sorted(types),
                   render_values=sorted(render),attachment_models=atts,examples=examples)
 summary[rev]=details
 print(rev,details['versions'],details['attachment_models'],details.get('texture_sizes'))
(ROOT/'census.json').write_text(json.dumps(summary,indent=1))

