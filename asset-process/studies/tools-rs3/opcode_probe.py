import pathlib,sys,sqlite3,re
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from definitions951 import *
db=sqlite3.connect(ROOT/'assets.sqlite')
c=Cache()
for k,ops in [('item',[201,205,203,190]),('npc',[170,187]),('object',[196,207,188]),('spot',[46])]:
 for op in ops:
    print(k,op)
    a,s=BANK[k]
    for i,stop,name in db.execute("select id,stop,name from assets where kind=? and stop like ? and name is null limit 3",(k,f'op {op} at %')):
        d=c.files(a,i>>s)[i%2**s]
        at=int(re.search(r'at (\d+)',stop)[1])
        print(i,d[at:at+64].hex(' '))

