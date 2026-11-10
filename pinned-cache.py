#!/usr/bin/env python3
"""Draft HF cache 2757 binding; trusted exclusive stable input/output namespace required."""
import hashlib,json,os,pathlib,stat,sys,urllib.request,zipfile
P=pathlib.Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
inv=(P/'cache242-static-inventory.json').read_bytes()
if sha(inv)!='053b5b0f3e449b0554934107185520d8d48782c3d811a62c00a0ab62d3a36aa0':raise ValueError('inventory pin')
rows=json.loads(inv)['files'];target='osrs-242_2026-11-03'; root=pathlib.Path(sys.argv[1]);parent=root.parent
if not root.is_absolute() or parent.is_symlink() or not parent.is_dir() or parent.stat().st_uid!=os.geteuid() or stat.S_IMODE(parent.stat().st_mode)!=0o700:raise ValueError('private parent required')
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs): return None
opener=urllib.request.build_opener(NoRedirect)
def download(url,limit):
 req=urllib.request.Request(url,headers={'User-Agent':'tsps-ew3788-case'})
 with opener.open(req,timeout=120) as r:
  if r.status!=200:raise ValueError('http')
  h=hashlib.sha256();n=0
  while True:
   b=r.read(1048576)
   if not b:break
   n+=len(b)
   if n>limit:raise ValueError('download limit')
   h.update(b);yield b,n,h
root.mkdir(mode=0o700,exist_ok=False)
try:
 archive=root/'disk.zip';last=0;digest=None
 with archive.open('xb') as f:
  for b,last,h in download('https://archive.openrs2.org/caches/runescape/2757/disk.zip',212685624):f.write(b);digest=h
 if last!=212685624 or digest is None or digest.hexdigest()!='940e86f321c8094ad845eaf39398747343f72d14acb30ecb95973a170e3fd6e9':raise ValueError('archive pin')
 keys=b''.join(b for b,_,_ in download('https://archive.openrs2.org/caches/runescape/2757/keys.json',3))
 if len(keys)!=3 or sha(keys)!='5984eac0c5c6d947241e29dd5671b81a1546cedf77e08d38438ac47029969afa':raise ValueError('raw keys pin')
 cache=root/'caches'; cache.mkdir(mode=0o700);sub=cache/target;sub.mkdir(mode=0o700);seen=set();seen_dirs=set()
 with zipfile.ZipFile(archive) as z:
  for i in z.infolist():
   if i.is_dir():
    typ=(i.external_attr>>16)&0o170000
    if i.filename!='cache/' or i.filename in seen_dirs or typ not in (0,stat.S_IFDIR):raise ValueError('directory')
    seen_dirs.add(i.filename);continue
   parts=i.filename.split('/')
   typ=(i.external_attr>>16)&0o170000
   if len(parts)!=2 or parts[0]!='cache' or not parts[1] or parts[1] in ('.','..') or i.filename in seen or typ not in (0,stat.S_IFREG):raise ValueError('zip member')
   seen.add(i.filename);key='caches/'+target+'/'+parts[1];row=rows.get(key)
   if not row or i.file_size!=row['bytes']:raise ValueError('zip inventory')
   h=hashlib.sha256();n=0
   with z.open(i) as src,(sub/parts[1]).open('xb') as dst:
    while True:
     b=src.read(1048576)
     if not b:break
     n+=len(b)
     if n>row['bytes']:raise ValueError('expanded size')
     h.update(b);dst.write(b)
   os.chmod(sub/parts[1],0o600)
   if n!=row['bytes'] or h.hexdigest()!=row['sha256']:raise ValueError('cache content')
 for dest,source in [(cache/'caches.json','derived-caches.json'),(sub/'info.json','derived-info.json'),(sub/'keys.json','derived-keys.json')]:
  b=(P/source).read_bytes();row=rows['caches/'+dest.relative_to(cache).as_posix()]
  if len(b)!=row['bytes'] or sha(b)!=row['sha256']:raise ValueError('metadata pin')
  with dest.open('xb') as f:f.write(b)
  os.chmod(dest,0o600)
 actual={};actual_dirs=set()
 for q in cache.rglob('*'):
  st=q.lstat()
  if stat.S_ISDIR(st.st_mode):
   actual_dirs.add(q.relative_to(cache).as_posix())
   if stat.S_IMODE(st.st_mode)!=0o700:raise ValueError('directory mode')
  elif stat.S_ISREG(st.st_mode):
   if st.st_nlink!=1 or stat.S_IMODE(st.st_mode)!=0o600:raise ValueError('file mode or links')
   h=hashlib.sha256();n=0
   with q.open('rb') as f:
    for b in iter(lambda:f.read(1048576),b''):n+=len(b);h.update(b)
   actual['caches/'+q.relative_to(cache).as_posix()]={'bytes':n,'sha256':h.hexdigest()}
  else:raise ValueError('special node')
 if actual_dirs!={target}:raise ValueError('directory inventory')
 if actual!=rows:raise ValueError('final inventory')
 print(json.dumps({'event':'cache-input-verified','cacheId':2757,'archiveSHA256':'940e86f321c8094ad845eaf39398747343f72d14acb30ecb95973a170e3fd6e9','files':len(actual),'scope':'initial cache inventory only; trusted stable namespace, total deadlines, post-use seal and process closure required'}))
except BaseException:
 try:print(json.dumps({'event':'cache-staging-failed','partialRoot':str(root),'disposition':'quarantine'}),file=sys.stderr)
 except BaseException:pass
 raise
