import sys,pathlib,sqlite3,json
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import ROOT
db=sqlite3.connect(ROOT/'assets.sqlite')
for name in ['raksha','seiryu','ambassador','kerapac','rasial','zuk','solak','sanctum','Vorkath','Black stone dragon','Verak Lith','Masuta','Nakatra','Amascut','Hermod']:
 print(name,db.execute("select id,name,models,properties from assets where kind='npc' and name like ? and models!='[]' order by json_extract(properties,'$.combat') desc limit 2",('%'+name+'%',)).fetchall().__str__()[:400])
for k in (90,91,92,93,94,95,96):
 print(db.execute("select id,models,properties from assets where kind='spot' and id=?",(k,)).fetchone())
for k in [3,11]:
 from rs3cache import Cache
 for i,d in Cache().files(26,0).items():
    if d[0]==0 and int.from_bytes(d[3:7],'big')==k:print(k,i,d.hex(' '));break

