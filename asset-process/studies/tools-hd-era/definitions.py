"""Extract study and model fields. Unknown opcodes stop that record at its byte offset."""
from hdcache import *
BANK={'object':(16,8),'npc':(18,7),'item':(19,8),'sequence':(20,7),'spot':(21,8)}
FIXED={
'npc':{0:[93,99,107,109,111,141,143,158,159,162],1:[100,101,119,125,128,140,163,165,168],2:[13,14,15,16,97,98,102,103,122,123,137,138,139,142,164],3:[135,136],4:[134],8:[17]},
'item':{0:[11,16,65,96,157],1:[27,42,113,114,115,134],2:[4,5,6,7,8,23,24,25,26,78,79,90,91,92,93,95,97,98,110,111,112,121,122,139,140],3:[125,126,127,128,129,130],4:[12,18]},
'object':{0:[17,18,21,22,23,27,62,64,73,74,82,88,89,90,91,94,96,97,98,103,105,168,169,177,189],1:[14,15,19,28,29,39,69,75,81,101,104,178,186],2:[24,60,65,66,67,68,70,71,72,93,95,102,107,167],3:[78,99,100,173],4:[163],0:[]},
'spot':{0:[9,10,11,12,13,14],1:[7,8,16],2:[4,5,6,15]},
}
FIXED['object'][0]=[17,18,21,22,23,27,62,64,73,74,82,88,89,90,91,94,96,97,98,103,105,168,169,177,189]
FIXED['npc'][4]+=[113,155]
FIXED['npc'][2]+=[114,115]
FIXED['object'][2].remove(95)
FIXED['object'][0].append(95)
FIXED['item'][0].remove(96)
FIXED['item'][1].append(96)
def parse(kind,data,rev=614):
 r=Cursor(data)
 d=dict(name='',models=[],size=1,combat=0,recolor=[],retexture=[],scale=[128]*3)
 while True:
    at=r.pos;op=r.u8()
    if op==0:
        if r.pos!=len(data):raise ValueError(f'{kind} trailing after {at}')
        return d
    if kind=='sequence' and op==1:
        d['frames']=r.u16();return d
    if kind=='sequence':
        if op in (4,14,15,16):continue
        if op in (5,8,9,10,11):r.take(1);continue
        if op in (2,6,7):r.take(2);continue
        if op==3:r.take(r.u8());continue
        raise ValueError(f'sequence prelude {op}')
    if op==249:
        for _ in range(r.u8()):
            s=r.u8();r.take(3)
            if s:r.string()
            else:r.take(4)
    elif op in (40,41):d['recolor' if op==40 else 'retexture']=[(r.u16(),r.u16()) for _ in range(r.u8())]
    elif op==2 and kind!='spot':d['name']=r.string()
    elif kind=='npc' and op in [1,60]:
        ids=[r.u16() for _ in range(r.u8())]
        if op==1:d['models']=ids
    elif kind=='npc' and op==12:d['size']=r.u8()
    elif kind=='npc' and op==95:d['combat']=r.u16()
    elif kind=='npc' and op==127:d['bas']=r.u16()
    elif kind=='npc' and op==121:d['offsets']=[(r.u8(),*struct.unpack('bbb',r.take(3))) for _ in range(r.u8())]
    elif kind=='npc' and op in (97,98):
        val=r.u16()
        if op==97:d['scale'][0]=d['scale'][2]=val
        else:d['scale'][1]=val
    elif kind=='npc' and op==134:r.take(9)
    elif kind=='npc' and op in [106,118] or kind=='object' and op in [77,92]:
        r.take(6 if op in (118,92) else 4);r.take(2*(1+r.u8()))
    elif kind=='object' and op in (1,5):
        if rev>=601:
            rows=[]
            for _ in range(r.u8()):
                shape=r.u8()
                rows.extend((r.u16(),shape) for _ in range(r.u8()))
            if op==5:
                for _ in range(r.u8()):
                    r.u8();r.take(2*r.u8())
        else:rows=[(r.u16(),r.u8() if op==1 else 10) for _ in range(r.u8())]
        if rows:
            shape=10 if any(s==10 for _,s in rows) else rows[0][1]
            d.update(shape=shape,models=[i for i,s in rows if s==shape])
    elif kind=='object' and op in (65,66,67):d['scale'][op-65]=r.u16()
    elif kind=='object' and op==62:d['mirror']=True
    elif kind=='object' and op in (70,71,72):d.setdefault('offset',[0]*3)[op-70]=int.from_bytes(r.take(2),'big',signed=True)
    elif kind=='spot' and op in (4,5):
        val=r.u16()
        if op==4:d['scale'][0]=d['scale'][2]=val
        else:d['scale'][1]=val
    elif kind=='spot' and op==6:d['rotation']=r.u16()
    elif kind=='item' and op in (110,111,112):d['scale'][op-110]=r.u16()
    elif kind in ('item','spot') and op==1:d['models']=[r.u16()]
    elif kind=='spot' and op==2:d['sequence']=r.u16()
    elif kind in ('object','npc','item') and (30<=op<35 or 150<=op<155 and kind!='item' or 35<=op<40 and kind=='item'):r.string()
    elif op==42 and kind in ('object','npc','item'):r.take(r.u8())
    elif kind=='item' and 100<=op<=109:r.take(4)
    elif kind=='object' and op==95 and rev>=601:r.take(2)
    elif kind=='object' and op==160:r.take(r.u8()*2)
    elif kind=='object' and op==79:r.take(5);r.take(r.u8()*2)
    elif kind=='object' and op==106:r.take(r.u8()*3)
    elif kind=='npc' and op==160 or kind=='item' and op==132:r.take(r.u8()*2)
    else:
        for size,ops in FIXED[kind].items():
            if op in ops:r.take(size);break
        else:raise ValueError(f'{kind} opcode {op} at {at}')
def definition(cache,kind,i):
 a,s=BANK[kind]
 return parse(kind,cache.files(a,i>>s)[i%2**s],cache.rev)
def records(cache,kind):
 a,s=BANK[kind]
 for g in cache.index(a):
    for f,data in cache.files(a,g).items():
        yield (g<<s)+f,data

