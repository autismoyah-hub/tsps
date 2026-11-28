import sys,pathlib,sqlite3,collections
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import *
c=Cache()
db=sqlite3.connect(ROOT/'assets.sqlite')
for kind in ('item','object','npc','spot'):
 print(kind,db.execute("select count(*),count(name),sum(models!='[]'),sum(stop is null) from assets where kind=?",(kind,)).fetchone())
 print(db.execute("select substr(stop,1,instr(stop,' at ')-1),count(*) as n from assets where kind=? and stop is not null group by 1 order by n desc limit 12",(kind,)).fetchall())
for term in ['arch-glacor','Croesus','%ambassador%','%raksha%','%seiryu%','%kerapac%','%zamorak, lord%','%zuks%','%zuk','%solak%','%dragonkin%','%masterwork platebody%','%fractured staff%','%eldritch crossbow%','%noxious%','%cryptbloom%','%tectonic%','%sirenic%','%vestments of havoc%','%roar of awakening%','%dark shard of leng%']:
 print(term,db.execute("select id,name,models from assets where name like ? and models!='[]' limit 4",(term,)).fetchall())
fs=c.files(26,0)
for k,count in collections.Counter((d[:3],len(d)) for d in fs.values()).most_common(22):
 i,d=next((i,d) for i,d in fs.items() if (d[:3],len(d))==k)
 print(i,count,d.hex(' '))

