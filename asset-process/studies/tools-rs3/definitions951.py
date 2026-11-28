"""Extract direct display fields from opcode-aligned definition prefixes. Report stops."""
import sys,pathlib,json,sqlite3,argparse
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import Cache,Cursor,ROOT
from definitions import FIXED
BANK={'item':(19,8),'npc':(18,7),'object':(16,8),'spot':(21,8)}
def model(r):
    if r.data[r.pos]&128:return r.num(4)&0x7fffffff
    value=r.u16()
    return -1 if value==32767 else value

def fields(kind,data):
    r=Cursor(data)
    name=None
    ids=[]
    props={}
    stop=None
    while r.pos<len(data):
        at=r.pos
        op=r.u8()
        try:
            if op==0:
                if r.pos!=len(data):stop=f'tail at {at}'
                break
            if op==1:
                if kind=='object':
                    shapes=[]
                    for _ in range(r.u8()):
                        shape=r.u8()
                        shapes.append([shape,[model(r) for _ in range(r.u8())]])
                    props['shapes']=shapes
                    ids=next((x for s,x in shapes if s==10),shapes[0][1] if shapes else [])
                elif kind=='npc':ids=[model(r) for _ in range(r.u8())]
                else:ids=[model(r)]
            elif op==2:
                if kind=='spot':props['sequence']=model(r)
                else:name=r.string()
            elif op in (40,41):
                props['recolour' if op==40 else 'retexture']=[[r.u16(),r.u16()] for _ in range(r.u8())]
            elif op==42:r.take(r.u8())
            elif 30<=op<=34 or kind=='item' and 35<=op<=39 or kind!='item' and 150<=op<=154:
                props.setdefault('actions',[]).append(r.string())
            elif op==249:
                for _ in range(r.u8()):
                    string=r.u8()
                    if string>1:raise ValueError('param type')
                    r.take(3)
                    if string:r.string()
                    else:r.take(4)
            elif kind=='item' and op in (23,24,25,26,78,79,90,91,92,93):
                props[str(op)]=model(r)
            elif kind=='item' and op in (13,14,69,178,165,15,181,94,44,45,144):
                # Identified fixed widths in 951, including 64-bit price at 181.
                r.take({13:1,14:1,69:4,178:1,165:1,15:0,181:8,94:2,44:2,45:2,144:2}[op])
            elif kind=='item' and op==9:
                # Empty payload in 951. Frequently immediately followed by model opcode 1.
                pass
            elif kind=='item' and 201<=op<=206:
                props[str(op)]=r.num(3)
            elif kind=='item' and 190<=op<=199:
                props.setdefault('quantity_variants',[]).append([r.num(3),r.u16()])
            elif kind=='item' and 100<=op<=109:r.take(4)
            elif kind=='npc' and op==95:props['combat']=r.u16()
            elif kind=='npc' and op==12:props['size']=r.u8()
            elif kind=='npc' and op==60:props['head']=[model(r) for _ in range(r.u8())]
            elif kind=='npc' and op in (26,27,44,45,127,170,182):
                r.take(0 if op==182 else 2)
            elif kind=='npc' and op==160:r.take(2*r.u8())
            elif kind=='object' and op==190:r.take(2)
            elif kind=='object' and op==196:r.take(1)
            elif kind=='object' and op==188:pass
            elif kind=='spot' and op==46:pass
            else:
                sizes=[n for n,ops in FIXED[kind].items() if op in ops]
                if not sizes:raise ValueError('unknown')
                r.take(sizes[0])
        except (IndexError,ValueError) as e:
            stop=f'op {op} at {at}: {e}'
            break
    else:
        stop=f'missing terminator at {r.pos}'
    return name,ids,props,stop

def build():
    db=sqlite3.connect(ROOT/'assets.sqlite')
    with db:
        db.execute('DROP TABLE IF EXISTS assets')
        db.execute('CREATE TABLE assets(kind TEXT,id INTEGER,name TEXT,models TEXT,properties TEXT,stop TEXT,PRIMARY KEY(kind,id))')
        cache=Cache()
        for kind,(a,shift) in BANK.items():
            for g in cache.index(a):
                for f,data in cache.files(a,g).items():
                    name,ids,props,stop=fields(kind,data)
                    db.execute('INSERT INTO assets VALUES(?,?,?,?,?,?)',(kind,(g<<shift)+f,name,json.dumps(ids),json.dumps(props),stop))
        db.execute('CREATE INDEX names ON assets(name COLLATE NOCASE)')
    db.close()
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build',action='store_true')
    p.add_argument('--name')
    args=p.parse_args()
    if args.build:build()
    if args.name:
        db=sqlite3.connect((ROOT/'assets.sqlite').as_uri()+'?mode=ro',uri=True)
        for row in db.execute('SELECT kind,id,name,models,stop FROM assets WHERE name LIKE ? ORDER BY kind,id',('%'+args.name+'%',)):print(*row,sep='\t')

