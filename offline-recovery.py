"""No-auth offline recovery; invoke only after iterator-ended seal exists."""
import hashlib, os, sys
from pathlib import Path
root=Path(__file__).resolve().parent
collector=root/'candidate'/'collector.py'
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def require(ok,msg):
 if not ok: raise ValueError(msg)
require(sys.flags.ignore_environment and sys.flags.no_site and sys.flags.dont_write_bytecode and not sys.flags.optimize,'interpreter flags')
require(Path(sys.executable).resolve()==Path('/usr/bin/python3.12').resolve(),'interpreter')
require(digest(Path(sys.executable).resolve())=='c2c20b4745d447551221ec3d4e70f92c270c4609fe3df34fc52ea6dd46e92273','interpreter hash')
for name in ('/','/usr','/usr/bin','/usr/lib','/usr/lib/python3.12',str(Path(sys.executable).resolve())):
 s=Path(name).stat(); require(s.st_uid==0 and not s.st_mode&0o022,'system chain')
s=root.stat();require(s.st_uid==os.getuid() and not s.st_mode&0o022,'control root')
require(digest(collector)=='fc08b552ba44e8f2085204631b1001d7c1fcf763f3ba1523ea263b3d6ed3d3ac','collector pin')
require(len(sys.argv)==1,'unexpected arguments')
os.execve(sys.executable,[sys.executable,'-E','-S','-B',str(collector),'--recover-offline'],{'CCC_SESSION_ID':os.environ.get('CCC_SESSION_ID',''),'PATH':'/usr/bin:/bin'})
