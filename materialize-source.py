#!/usr/bin/env python3
"""Draft: requires exclusive, trusted, stable parent namespace; no hostile-FS guarantee."""
import hashlib,json,os,pathlib,stat,sys,tarfile
p=pathlib.Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
a=p/'main-99808.tar.gz'; manifest=p/'composed-source-manifest.json'
if a.stat().st_size!=41252714 or H(a.read_bytes())!='1fc07cd0607cdd133f1c698546e0a8dae1524c56b51edb20cec11383a2be2e4f':raise ValueError('archive pin')
if H(manifest.read_bytes())!='3c75cf3a085bbe5ea4e5c2c955e22cd92de4e52e2db037c5a6d909d6857f56ac':raise ValueError('manifest pin')
files=json.loads(manifest.read_bytes())['files']; dirs={x[:i] for x in files for i,c in enumerate(x) if c=='/'}
out=pathlib.Path(sys.argv[1]); parent=out.parent
if not out.is_absolute() or parent.is_symlink() or not parent.is_dir():raise ValueError('trusted absolute parent required')
if parent.stat().st_uid!=os.geteuid() or stat.S_IMODE(parent.stat().st_mode)!=0o700:raise ValueError('private owned 0700 parent required')
fix={'tools/quest-helper-bridge/gradlew.bat':'gradlew.bat.gitblob','server/plugins/quests/quests/ElementalWorkshopII.Quest.js':'quest.js','server/tests/qh728-q031.test.cjs':'test.cjs'}
prefix='tsps-99808d6ee04ffc4931c62bcbc356836c1439363c'; seen=set(); seen_dirs=set(); top=False
out.mkdir(mode=0o700,exist_ok=False)
try:
 os.chmod(out,0o700)
 with tarfile.open(a,'r:gz') as t:
  for m in t:
   if m.name==prefix:
    if top or not m.isdir():raise ValueError('root member')
    top=True;continue
   if not m.name.startswith(prefix+'/'):raise ValueError('archive prefix')
   name=m.name[len(prefix)+1:].rstrip('/')
   if not name or any(x in ('..','.','') for x in name.split('/')):raise ValueError('unsafe path')
   if m.isdir():
    if name not in dirs or name in seen_dirs:raise ValueError('directory '+name)
    seen_dirs.add(name);continue
   if m.name.endswith('/') or not m.isfile() or name in seen or name not in files:raise ValueError('leaf '+name)
   row=files[name]; mode=0o755 if row['mode']=='100755' else 0o644
   if ('100755' if m.mode&0o111 else '100644')!=row['mode']:raise ValueError('archive mode '+name)
   b=(p/fix[name]).read_bytes() if name in fix else t.extractfile(m).read()
   if len(b)!=row['bytes'] or H(b)!=row['sha256']:raise ValueError('content '+name)
   target=out.joinpath(*name.split('/'));target.parent.mkdir(mode=0o700,parents=True,exist_ok=True)
   fd=os.open(target,os.O_CREAT|os.O_EXCL|os.O_WRONLY,mode)
   try:
    f=os.fdopen(fd,'wb')
   except BaseException:
    try: os.close(fd)
    except OSError as close_error:
     try: print(json.dumps({'event':'fd-close-uncertain','path':name,'error':type(close_error).__name__}),file=sys.stderr)
     except BaseException: pass
    raise
   try:
    f.write(b);f.flush();os.fchmod(f.fileno(),mode)
   except BaseException:
    try: f.close()
    except BaseException as close_error:
     try: print(json.dumps({'event':'file-close-uncertain','path':name,'error':type(close_error).__name__}),file=sys.stderr)
     except BaseException: pass
    raise
   else:
    f.close()
   st=target.lstat()
   if not stat.S_ISREG(st.st_mode) or st.st_nlink!=1 or stat.S_IMODE(st.st_mode)!=mode or H(target.read_bytes())!=row['sha256']:raise ValueError('written leaf '+name)
   seen.add(name)
 if not top or seen!=set(files) or seen_dirs!=dirs:raise ValueError('archive membership')
 actual_files=set();actual_dirs=set()
 for base,ds,fs in os.walk(out,followlinks=False):
  for n in ds:
   q=pathlib.Path(base)/n; s=q.lstat(); rel=q.relative_to(out).as_posix()
   if not stat.S_ISDIR(s.st_mode) or stat.S_IMODE(s.st_mode)!=0o700:raise ValueError('output directory '+rel)
   actual_dirs.add(rel)
  for n in fs:
   q=pathlib.Path(base)/n; rel=q.relative_to(out).as_posix();s=q.lstat()
   if rel not in files or not stat.S_ISREG(s.st_mode) or s.st_nlink!=1:raise ValueError('extra/type '+rel)
   row=files[rel]; mode=0o755 if row['mode']=='100755' else 0o644
   if stat.S_IMODE(s.st_mode)!=mode or s.st_size!=row['bytes'] or H(q.read_bytes())!=row['sha256']:raise ValueError('output mismatch '+rel)
   actual_files.add(rel)
 if actual_files!=set(files) or actual_dirs!=dirs:raise ValueError('output inventory')
 print(json.dumps({'event':'source-materialized','leafCount':len(seen),'directoryCount':len(dirs),'manifestSHA256':H(manifest.read_bytes()),'precondition':'exclusive trusted stable parent namespace','limitation':'initial inventory only; post-build/test seals and process closure required'}))
except BaseException:
 try: print(json.dumps({'event':'materialization-failed','partialRoot':str(out),'disposition':'quarantine; preserve for diagnosis'}),file=sys.stderr)
 except BaseException: pass
 raise
